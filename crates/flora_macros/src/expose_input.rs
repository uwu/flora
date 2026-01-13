use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

/// Main entry point for the expose_input attribute macro.
pub fn attr_macro(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match attr_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn attr_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    // Collect any additional derives from the original input (like Default)
    let mut extra_derives = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("derive") {
            // Parse the derive attribute to extract derive names
            let _ = attr.parse_nested_meta(|meta| {
                let path = &meta.path;
                // Skip derives we're adding ourselves
                if !path.is_ident("Debug") && !path.is_ident("Deserialize") && !path.is_ident("TS")
                {
                    extra_derives.push(quote! { #path });
                }
                Ok(())
            });
        } else if !attr.path().is_ident("expose")
            && !attr.path().is_ident("serde")
            && !attr.path().is_ident("ts")
        {
            // Preserve other attributes like #[doc]
            other_attrs.push(quote! { #attr });
        }
    }

    // Extract struct fields
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "expose_input can only be applied to structs",
            ));
        }
    };

    // Generate the output struct with derives
    let output = match fields {
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

            let extra_derives = if extra_derives.is_empty() {
                quote! {}
            } else {
                quote! { #(#extra_derives,)* }
            };

            quote! {
                #(#other_attrs)*
                #[derive(Debug, serde::Deserialize, T0x, #extra_derives)]
                #[serde(rename_all = "camelCase")]
                #vis struct #name #impl_generics #where_clause {
                    #(#field_defs),*
                }
            }
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

            let extra_derives = if extra_derives.is_empty() {
                quote! {}
            } else {
                quote! { #(#extra_derives,)* }
            };

            quote! {
                #(#other_attrs)*
                #[derive(Debug, serde::Deserialize, T0x, #extra_derives)]
                #[serde(rename_all = "camelCase")]
                #vis struct #name #impl_generics #where_clause (
                    #(#field_defs),*
                );
            }
        }
        Fields::Unit => {
            let extra_derives = if extra_derives.is_empty() {
                quote! {}
            } else {
                quote! { #(#extra_derives,)* }
            };

            quote! {
                #(#other_attrs)*
                #[derive(Debug, serde::Deserialize, T0x, #extra_derives)]
                #[serde(rename_all = "camelCase")]
                #vis struct #name #impl_generics #where_clause;
            }
        }
    };

    Ok(output)
}
