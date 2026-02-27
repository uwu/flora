use crate::transpile::transpile_if_typescript;
use deno_core::ModuleName;
use flora_config::RuntimeConfig;
use oxc::{
    allocator::Allocator,
    ast::ast::{
        BindingPattern, BindingPatternKind, Declaration, ExportAllDeclaration,
        ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration,
        ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, Statement,
    },
    codegen::{Codegen, Context, Gen},
    parser::Parser,
    span::SourceType,
};
use oxc_sourcemap::SourceMapBuilder;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

const RUNTIME_MODULE_RESOLVER: &str =
    include_str!("../../../runtime-dist/runtime_module_resolution.js");
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DeploymentFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug)]
pub struct BundleOutput {
    pub code: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error("entry file not found: {0}")]
    EntryMissing(String),
    #[error("unsupported import: {0}")]
    UnsupportedImport(String),
    #[error("missing module: {0}")]
    MissingModule(String),
    #[error("failed to parse {path}: {message}")]
    ParseError { path: String, message: String },
    #[error("failed to transpile {path}: {message}")]
    TranspileError { path: String, message: String },
    #[error("invalid file path: {0}")]
    InvalidPath(String),
}

const MAX_FILES: usize = 200;
const MAX_TOTAL_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Copy)]
pub struct BundleLimits {
    pub max_files: usize,
    pub max_total_bytes: usize,
}

impl BundleLimits {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            max_files: config.max_bundle_files,
            max_total_bytes: config.max_bundle_total_bytes,
        }
    }
}

impl Default for BundleLimits {
    fn default() -> Self {
        Self {
            max_files: MAX_FILES,
            max_total_bytes: MAX_TOTAL_BYTES,
        }
    }
}

pub fn bundle_files(
    bundle_name: &str,
    entry: &str,
    files: &[DeploymentFile],
    limits: BundleLimits,
) -> Result<BundleOutput, BundleError> {
    if files.len() > limits.max_files {
        return Err(BundleError::InvalidPath(format!(
            "too many files (max {})",
            limits.max_files
        )));
    }

    let mut total_bytes = 0usize;
    let mut file_map = HashMap::new();
    for file in files {
        total_bytes = total_bytes.saturating_add(file.contents.len());
        if total_bytes > limits.max_total_bytes {
            return Err(BundleError::InvalidPath(format!(
                "bundle exceeds size limit (max {} bytes)",
                limits.max_total_bytes
            )));
        }
        let normalized = normalize_path(&file.path)?;
        if file_map
            .insert(normalized.clone(), file.contents.clone())
            .is_some()
        {
            return Err(BundleError::InvalidPath(format!(
                "duplicate file path: {normalized}"
            )));
        }
    }

    let entry = normalize_path(entry)?;
    if !file_map.contains_key(&entry) {
        return Err(BundleError::EntryMissing(entry));
    }

    let mut modules = HashMap::new();
    let mut visited = HashSet::new();
    collect_modules(&entry, &file_map, &mut visited, &mut modules)?;

    let mut output = String::new();
    output.push_str(RUNTIME_MODULE_RESOLVER);
    output.push('\n');

    let mut module_ids: Vec<String> = modules.keys().cloned().collect();
    module_ids.sort();
    for id in &module_ids {
        let module = modules.get(id).expect("module missing");
        output.push_str(&format!(
            "__define({}, (exports, module) => {{\n",
            quote(id)
        ));
        output.push_str(&module.code);
        if !module.code.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("});\n");
    }

    output.push_str(&format!("__require({});\n", quote(&entry)));

    let mut sourcemap = SourceMapBuilder::default();
    sourcemap.set_file(bundle_name);
    for id in module_ids {
        let module = modules.get(&id).expect("module missing");
        sourcemap.add_source_and_content(&id, &module.source);
    }
    let sm = sourcemap.into_sourcemap();
    output.push_str("//# sourceMappingURL=");
    output.push_str(&sm.to_data_url());
    output.push('\n');

    Ok(BundleOutput { code: output })
}

struct ModuleOutput {
    code: String,
    source: String,
}

fn collect_modules(
    entry: &str,
    file_map: &HashMap<String, String>,
    visited: &mut HashSet<String>,
    modules: &mut HashMap<String, ModuleOutput>,
) -> Result<(), BundleError> {
    if !visited.insert(entry.to_string()) {
        return Ok(());
    }

    let source = file_map
        .get(entry)
        .cloned()
        .ok_or_else(|| BundleError::MissingModule(entry.to_string()))?;
    let transpiled = transpile_source(entry, &source)?;
    let (code, deps) = transform_module(entry, &transpiled, file_map)?;

    modules.insert(entry.to_string(), ModuleOutput { code, source });

    for dep in deps {
        collect_modules(&dep, file_map, visited, modules)?;
    }

    Ok(())
}

fn transpile_source(path: &str, source: &str) -> Result<String, BundleError> {
    let module_name = ModuleName::from(path.to_string());
    match transpile_if_typescript(&module_name, source).map_err(|err| {
        BundleError::TranspileError {
            path: path.to_string(),
            message: err.to_string(),
        }
    })? {
        Some(output) => Ok(output.code.to_string()),
        None => Ok(source.to_string()),
    }
}

fn transform_module(
    path: &str,
    source: &str,
    file_map: &HashMap<String, String>,
) -> Result<(String, Vec<String>), BundleError> {
    let allocator = Allocator::default();
    let mut source_type = SourceType::from_path(path).map_err(|err| BundleError::ParseError {
        path: path.to_string(),
        message: err.to_string(),
    })?;
    source_type = source_type.with_module(true);
    let parser = Parser::new(&allocator, source, source_type);
    let parsed = parser.parse();
    if !parsed.errors.is_empty() {
        let message = parsed
            .errors
            .iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(BundleError::ParseError {
            path: path.to_string(),
            message,
        });
    }

    let mut output = String::new();
    let mut deps = Vec::new();
    let mut counter = 0usize;

    for stmt in parsed.program.body.iter() {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                if decl.import_kind == ImportOrExportKind::Type {
                    continue;
                }
                let spec = decl.source.value.as_str().to_string();
                let resolved = resolve_specifier(path, &spec, file_map)?;
                deps.push(resolved.clone());
                counter += 1;
                output.push_str(&render_import(decl, &resolved, counter));
            }
            Statement::ExportNamedDeclaration(decl) => {
                output.push_str(&render_export_named(
                    decl,
                    path,
                    file_map,
                    &mut deps,
                    &mut counter,
                )?);
            }
            Statement::ExportDefaultDeclaration(decl) => {
                output.push_str(&render_export_default(decl));
            }
            Statement::ExportAllDeclaration(decl) => {
                output.push_str(&render_export_all(
                    decl,
                    path,
                    file_map,
                    &mut deps,
                    &mut counter,
                )?);
            }
            _ => {
                output.push_str(&render_node(stmt));
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            }
        }
    }

    Ok((output, deps))
}

fn render_import(decl: &ImportDeclaration<'_>, spec: &str, counter: usize) -> String {
    let mut out = String::new();
    match &decl.specifiers {
        None => {
            out.push_str(&format!("__require({});\n", quote(spec)));
        }
        Some(specifiers) => {
            let mut default = None;
            let mut namespace = None;
            let mut named = Vec::new();

            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        default = Some(spec.local.name.as_str().to_string());
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        namespace = Some(spec.local.name.as_str().to_string());
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        if spec.import_kind == ImportOrExportKind::Type {
                            continue;
                        }
                        let imported = match &spec.imported {
                            oxc::ast::ast::ModuleExportName::IdentifierName(name) => {
                                name.name.as_str().to_string()
                            }
                            oxc::ast::ast::ModuleExportName::IdentifierReference(name) => {
                                name.name.as_str().to_string()
                            }
                            oxc::ast::ast::ModuleExportName::StringLiteral(lit) => {
                                lit.value.as_str().to_string()
                            }
                        };
                        let local = spec.local.name.as_str().to_string();
                        named.push((imported, local));
                    }
                }
            }

            let module_ident = format!("__mod{counter}");
            out.push_str(&format!(
                "const {module_ident} = __require({});\n",
                quote(spec)
            ));
            if let Some(name) = namespace {
                out.push_str(&format!("const {name} = {module_ident};\n"));
            }
            if let Some(name) = default {
                out.push_str(&format!("const {name} = {module_ident}.default;\n"));
            }
            if !named.is_empty() {
                out.push_str("const { ");
                for (idx, (imported, local)) in named.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    if imported == local {
                        out.push_str(imported);
                    } else {
                        out.push_str(imported);
                        out.push_str(": ");
                        out.push_str(local);
                    }
                }
                out.push_str(&format!(" }} = {module_ident};\n"));
            }
        }
    }
    out
}

fn render_export_named(
    decl: &ExportNamedDeclaration<'_>,
    path: &str,
    file_map: &HashMap<String, String>,
    deps: &mut Vec<String>,
    counter: &mut usize,
) -> Result<String, BundleError> {
    if decl.export_kind == ImportOrExportKind::Type {
        return Ok(String::new());
    }

    let mut out = String::new();
    if let Some(declaration) = &decl.declaration {
        out.push_str(&render_declaration(declaration));
        for name in collect_declaration_names(declaration) {
            out.push_str(&format!("exports.{name} = {name};\n"));
        }
        return Ok(out);
    }

    if let Some(source) = &decl.source {
        let spec = source.value.as_str().to_string();
        let resolved = resolve_specifier(path, &spec, file_map)?;
        deps.push(resolved.clone());
        *counter += 1;
        let module_ident = format!("__mod{counter}");
        out.push_str(&format!(
            "const {module_ident} = __require({});\n",
            quote(&resolved)
        ));
        for specifier in &decl.specifiers {
            if specifier.export_kind == ImportOrExportKind::Type {
                continue;
            }
            let local = module_export_name(&specifier.local);
            let exported = module_export_name(&specifier.exported);
            out.push_str(&format!("exports.{exported} = {module_ident}.{local};\n"));
        }
        return Ok(out);
    }

    for specifier in &decl.specifiers {
        if specifier.export_kind == ImportOrExportKind::Type {
            continue;
        }
        let local = module_export_name(&specifier.local);
        let exported = module_export_name(&specifier.exported);
        out.push_str(&format!("exports.{exported} = {local};\n"));
    }
    Ok(out)
}

fn render_export_default(decl: &ExportDefaultDeclaration<'_>) -> String {
    match &decl.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                let mut out = render_node(func.as_ref());
                let name = id.name.as_str();
                out.push_str(&format!("exports.default = {name};\n"));
                out
            } else {
                let expr = render_node(func.as_ref());
                format!("exports.default = {expr};\n")
            }
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                let mut out = render_node(class.as_ref());
                let name = id.name.as_str();
                out.push_str(&format!("exports.default = {name};\n"));
                out
            } else {
                let expr = render_node(class.as_ref());
                format!("exports.default = {expr};\n")
            }
        }
        _ => {
            let mut out = String::new();
            let expr = trim_trailing_semicolon(render_node(&decl.declaration));
            out.push_str(&format!("const __default = {expr};\n"));
            out.push_str("exports.default = __default;\n");
            out
        }
    }
}

fn render_export_all(
    decl: &ExportAllDeclaration<'_>,
    path: &str,
    file_map: &HashMap<String, String>,
    deps: &mut Vec<String>,
    counter: &mut usize,
) -> Result<String, BundleError> {
    let spec = decl.source.value.as_str().to_string();
    let resolved = resolve_specifier(path, &spec, file_map)?;
    deps.push(resolved.clone());
    *counter += 1;
    let module_ident = format!("__mod{counter}");
    let mut out = String::new();
    out.push_str(&format!(
        "const {module_ident} = __require({});\n",
        quote(&resolved)
    ));
    if let Some(exported) = &decl.exported {
        let exported = module_export_name(exported);
        out.push_str(&format!("exports.{exported} = {module_ident};\n"));
    } else {
        out.push_str(&format!(
            "for (const __k in {module_ident}) {{ if (__k !== \"default\") exports[__k] = {module_ident}[__k]; }}\n"
        ));
    }
    Ok(out)
}

fn render_declaration(declaration: &Declaration<'_>) -> String {
    match declaration {
        Declaration::VariableDeclaration(decl) => render_node(decl.as_ref()),
        Declaration::FunctionDeclaration(decl) => render_node(decl.as_ref()),
        Declaration::ClassDeclaration(decl) => render_node(decl.as_ref()),
        _ => String::new(),
    }
}

fn collect_declaration_names(declaration: &Declaration<'_>) -> Vec<String> {
    match declaration {
        Declaration::VariableDeclaration(decl) => {
            let mut names = Vec::new();
            for declarator in decl.declarations.iter() {
                collect_binding_names(&declarator.id, &mut names);
            }
            names
        }
        Declaration::FunctionDeclaration(decl) => decl
            .id
            .as_ref()
            .map(|id| vec![id.name.as_str().to_string()])
            .unwrap_or_default(),
        Declaration::ClassDeclaration(decl) => decl
            .id
            .as_ref()
            .map(|id| vec![id.name.as_str().to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn collect_binding_names(pattern: &BindingPattern<'_>, out: &mut Vec<String>) {
    match &pattern.kind {
        BindingPatternKind::BindingIdentifier(ident) => {
            out.push(ident.name.as_str().to_string());
        }
        BindingPatternKind::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                collect_binding_names(&prop.value, out);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names(&rest.argument, out);
            }
        }
        BindingPatternKind::ArrayPattern(arr) => {
            for pat in arr.elements.iter().flatten() {
                collect_binding_names(pat, out);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names(&rest.argument, out);
            }
        }
        BindingPatternKind::AssignmentPattern(assign) => {
            collect_binding_names(&assign.left, out);
        }
    }
}

fn render_node<T: Gen>(node: &T) -> String {
    let mut codegen = Codegen::new();
    node.r#gen(&mut codegen, Context::default());
    String::from(codegen)
}

fn trim_trailing_semicolon(value: String) -> String {
    let trimmed = value.trim_end();
    if let Some(stripped) = trimmed.strip_suffix(';') {
        stripped.trim_end().to_string()
    } else {
        trimmed.to_string()
    }
}

fn module_export_name(name: &oxc::ast::ast::ModuleExportName<'_>) -> String {
    match name {
        oxc::ast::ast::ModuleExportName::IdentifierName(ident) => ident.name.as_str().to_string(),
        oxc::ast::ast::ModuleExportName::IdentifierReference(ident) => {
            ident.name.as_str().to_string()
        }
        oxc::ast::ast::ModuleExportName::StringLiteral(lit) => lit.value.as_str().to_string(),
    }
}

fn quote(value: &str) -> String {
    format!("{:?}", value)
}

fn resolve_specifier(
    from: &str,
    specifier: &str,
    file_map: &HashMap<String, String>,
) -> Result<String, BundleError> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return Err(BundleError::UnsupportedImport(specifier.to_string()));
    }

    let from_dir = Path::new(from).parent().unwrap_or_else(|| Path::new(""));
    let candidate = normalize_path(&from_dir.join(specifier).to_string_lossy())?;

    let mut candidates = Vec::new();
    if Path::new(&candidate).extension().is_some() {
        candidates.push(candidate.clone());
    } else {
        for ext in [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cts"] {
            candidates.push(format!("{candidate}{ext}"));
        }
    }

    let mut final_candidates = Vec::new();
    for candidate in candidates {
        final_candidates.push(candidate.clone());
        final_candidates.push(format!("{candidate}/index.ts"));
        final_candidates.push(format!("{candidate}/index.tsx"));
        final_candidates.push(format!("{candidate}/index.js"));
        final_candidates.push(format!("{candidate}/index.jsx"));
        final_candidates.push(format!("{candidate}/index.mjs"));
        final_candidates.push(format!("{candidate}/index.cts"));
    }

    for candidate in final_candidates {
        if file_map.contains_key(&candidate) {
            return Ok(candidate);
        }
    }

    Err(BundleError::MissingModule(specifier.to_string()))
}

fn normalize_path(input: &str) -> Result<String, BundleError> {
    let path = Path::new(input);
    if path.is_absolute() {
        return Err(BundleError::InvalidPath(input.to_string()));
    }

    let mut parts: Vec<&str> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => {
                if let Some(part) = part.to_str() {
                    parts.push(part);
                } else {
                    return Err(BundleError::InvalidPath(input.to_string()));
                }
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(BundleError::InvalidPath(input.to_string()));
                }
            }
            _ => return Err(BundleError::InvalidPath(input.to_string())),
        }
    }

    let normalized = parts.join("/");
    if normalized.is_empty() {
        return Err(BundleError::InvalidPath(input.to_string()));
    }
    Ok(normalized)
}
