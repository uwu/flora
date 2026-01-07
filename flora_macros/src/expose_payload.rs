use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Lit, parse_macro_input};

/// Main entry point for the expose_payload attribute macro.
pub fn attr_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as MacroArgs);

    match attr_impl(input, args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Parsed macro arguments
struct MacroArgs {
    from_type: Option<String>,
}

impl syn::parse::Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut from_type = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident == "from" {
                let _: syn::Token![=] = input.parse()?;
                let lit: syn::LitStr = input.parse()?;
                from_type = Some(lit.value());
            }

            // Handle trailing comma
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(MacroArgs { from_type })
    }
}

fn attr_impl(input: DeriveInput, args: MacroArgs) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Collect any additional derives from the original input
    let mut extra_derives = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                let path = &meta.path;
                if !path.is_ident("Serialize") && !path.is_ident("TS") {
                    extra_derives.push(quote! { #path });
                }
                Ok(())
            });
        } else if !attr.path().is_ident("expose")
            && !attr.path().is_ident("serde")
            && !attr.path().is_ident("ts")
        {
            other_attrs.push(quote! { #attr });
        }
    }

    // Extract struct fields
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "expose_payload can only be applied to structs",
            ));
        }
    };

    // Process fields and extract expose attributes
    let processed_fields = process_fields(fields, args.from_type.is_some())?;

    // Generate the struct definition
    let struct_def = generate_struct_def(
        name,
        vis,
        &impl_generics,
        where_clause,
        &extra_derives,
        &other_attrs,
        fields,
        &processed_fields,
    )?;

    // Optionally generate From impl
    let from_impl = if let Some(from_type_str) = &args.from_type {
        let from_type: syn::Type = syn::parse_str(from_type_str).map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("invalid type in from = \"...\": {}", e),
            )
        })?;

        generate_from_impl(
            name,
            &impl_generics,
            &ty_generics,
            where_clause,
            &from_type,
            fields,
            &processed_fields,
        )?
    } else {
        quote! {}
    };

    Ok(quote! {
        #struct_def
        #from_impl
    })
}

/// Field-level attributes
struct FieldAttrs {
    expr: Option<String>,
    skip: bool,
    default: bool,
}

struct ProcessedField {
    attrs: FieldAttrs,
}

fn process_fields(fields: &Fields, has_from: bool) -> syn::Result<Vec<ProcessedField>> {
    let field_iter: Box<dyn Iterator<Item = &syn::Field>> = match fields {
        Fields::Named(f) => Box::new(f.named.iter()),
        Fields::Unnamed(f) => Box::new(f.unnamed.iter()),
        Fields::Unit => Box::new(std::iter::empty()),
    };

    field_iter.map(|f| process_field(f, has_from)).collect()
}

fn process_field(field: &syn::Field, has_from: bool) -> syn::Result<ProcessedField> {
    let mut attrs = FieldAttrs { expr: None, skip: false, default: false };

    for attr in &field.attrs {
        if attr.path().is_ident("expose") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("expr") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(lit) = value {
                        attrs.expr = Some(lit.value());
                    }
                } else if meta.path.is_ident("skip") {
                    attrs.skip = true;
                } else if meta.path.is_ident("default") {
                    attrs.default = true;
                }
                Ok(())
            })?;
        }
    }

    // Validate: if struct has `from` attribute, fields need either expr, skip, or default
    if has_from && attrs.expr.is_none() && !attrs.skip && !attrs.default {
        let field_name =
            field.ident.as_ref().map(|i| i.to_string()).unwrap_or_else(|| "unnamed".to_string());
        return Err(syn::Error::new_spanned(
            field,
            format!(
                "field `{}` requires #[expose(expr = \"...\")] when from = \"...\" is set\n\
                 help: add #[expose(expr = \"src.{}.clone()\")] or #[expose(skip)] or #[expose(default)]",
                field_name, field_name
            ),
        ));
    }

    Ok(ProcessedField { attrs })
}

fn generate_struct_def(
    name: &syn::Ident,
    vis: &syn::Visibility,
    impl_generics: &syn::ImplGenerics,
    where_clause: Option<&syn::WhereClause>,
    extra_derives: &[proc_macro2::TokenStream],
    other_attrs: &[proc_macro2::TokenStream],
    fields: &Fields,
    _processed_fields: &[ProcessedField],
) -> syn::Result<proc_macro2::TokenStream> {
    let extra_derives = if extra_derives.is_empty() {
        quote! {}
    } else {
        quote! { #(#extra_derives,)* }
    };

    match fields {
        Fields::Named(fields_named) => {
            let field_defs: Vec<_> = fields_named
                .named
                .iter()
                .map(|field| {
                    let field_name = &field.ident;
                    let field_ty = &field.ty;
                    let field_vis = &field.vis;

                    // Preserve non-expose attributes
                    let attrs: Vec<_> = field
                        .attrs
                        .iter()
                        .filter(|a| !a.path().is_ident("expose"))
                        .map(|a| quote! { #a })
                        .collect();

                    quote! {
                        #(#attrs)*
                        #field_vis #field_name: #field_ty
                    }
                })
                .collect();

            Ok(quote! {
                #(#other_attrs)*
                #[derive(Debug, serde::Serialize, ts_rs::TS, #extra_derives)]
                #[serde(rename_all = "camelCase")]
                #[ts(export, optional_fields)]
                #vis struct #name #impl_generics #where_clause {
                    #(#field_defs),*
                }
            })
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_defs: Vec<_> = fields_unnamed
                .unnamed
                .iter()
                .map(|field| {
                    let field_ty = &field.ty;
                    let field_vis = &field.vis;
                    let attrs: Vec<_> = field
                        .attrs
                        .iter()
                        .filter(|a| !a.path().is_ident("expose"))
                        .map(|a| quote! { #a })
                        .collect();

                    quote! {
                        #(#attrs)*
                        #field_vis #field_ty
                    }
                })
                .collect();

            Ok(quote! {
                #(#other_attrs)*
                #[derive(Debug, serde::Serialize, ts_rs::TS, #extra_derives)]
                #[serde(rename_all = "camelCase")]
                #[ts(export, optional_fields)]
                #vis struct #name #impl_generics #where_clause (
                    #(#field_defs),*
                );
            })
        }
        Fields::Unit => Ok(quote! {
            #(#other_attrs)*
            #[derive(Debug, serde::Serialize, ts_rs::TS, #extra_derives)]
            #[serde(rename_all = "camelCase")]
            #[ts(export, optional_fields)]
            #vis struct #name #impl_generics #where_clause;
        }),
    }
}

fn generate_from_impl(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    from_type: &syn::Type,
    fields: &Fields,
    processed_fields: &[ProcessedField],
) -> syn::Result<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(fields_named) => {
            let field_inits: Vec<_> = fields_named
                .named
                .iter()
                .zip(processed_fields.iter())
                .map(|(field, processed)| {
                    let field_name = &field.ident;

                    if processed.attrs.skip || processed.attrs.default {
                        quote! { #field_name: Default::default() }
                    } else if let Some(expr_str) = &processed.attrs.expr {
                        let expr: syn::Expr = syn::parse_str(expr_str)
                            .unwrap_or_else(|_| panic!("invalid expression: {}", expr_str));
                        quote! { #field_name: #expr }
                    } else {
                        quote! { #field_name: Default::default() }
                    }
                })
                .collect();

            Ok(quote! {
                impl #impl_generics From<&#from_type> for #name #ty_generics #where_clause {
                    fn from(src: &#from_type) -> Self {
                        Self {
                            #(#field_inits),*
                        }
                    }
                }
            })
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_inits: Vec<_> = fields_unnamed
                .unnamed
                .iter()
                .zip(processed_fields.iter())
                .map(|(_, processed)| {
                    if processed.attrs.skip || processed.attrs.default {
                        quote! { Default::default() }
                    } else if let Some(expr_str) = &processed.attrs.expr {
                        let expr: syn::Expr = syn::parse_str(expr_str)
                            .unwrap_or_else(|_| panic!("invalid expression: {}", expr_str));
                        quote! { #expr }
                    } else {
                        quote! { Default::default() }
                    }
                })
                .collect();

            Ok(quote! {
                impl #impl_generics From<&#from_type> for #name #ty_generics #where_clause {
                    fn from(src: &#from_type) -> Self {
                        Self(#(#field_inits),*)
                    }
                }
            })
        }
        Fields::Unit => Ok(quote! {
            impl #impl_generics From<&#from_type> for #name #ty_generics #where_clause {
                fn from(_src: &#from_type) -> Self {
                    Self
                }
            }
        }),
    }
}
