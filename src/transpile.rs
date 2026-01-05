use std::path::Path;

use deno_core::{ModuleCodeString, ModuleName, SourceMapData};
use deno_error::JsErrorBox;
use oxc::{
    CompilerInterface,
    codegen::{CodegenOptions, CodegenReturn},
    diagnostics::OxcDiagnostic,
    span::SourceType,
    transformer::TransformOptions,
};

pub struct TranspileOutput {
    pub code: ModuleCodeString,
    pub source_map: Option<SourceMapData>,
}

/// Transpile TypeScript/TSX sources to JavaScript using oxc.
///
/// Returns `Ok(None)` when the specifier is not a TypeScript-like path.
pub fn transpile_if_typescript(
    specifier: &ModuleName,
    source: &str,
) -> Result<Option<TranspileOutput>, JsErrorBox> {
    if !is_typescript_specifier(specifier.as_str()) {
        return Ok(None);
    }

    let source_type = SourceType::from_path(specifier.as_str())
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    if !source_type.is_typescript() {
        return Ok(None);
    }

    let mut compiler = TsCompiler::default();
    compiler.compile(source, source_type, Path::new(specifier.as_str()));
    if !compiler.errors.is_empty() {
        let message = compiler.describe_errors(specifier.as_str(), source);
        return Err(JsErrorBox::generic(message));
    }

    let source_map = compiler.source_map.map(|map| map.to_json_string().into_bytes().into());

    Ok(Some(TranspileOutput { code: ModuleCodeString::from(compiler.output), source_map }))
}

fn is_typescript_specifier(specifier: &str) -> bool {
    Path::new(specifier)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|ext| matches!(ext, "ts" | "mts" | "cts" | "tsx"))
}

#[derive(Default)]
struct TsCompiler {
    output: String,
    source_map: Option<oxc_sourcemap::SourceMap>,
    errors: Vec<OxcDiagnostic>,
    options: TransformOptions,
}

impl TsCompiler {
    fn describe_errors(&self, specifier: &str, source: &str) -> String {
        let rendered = self
            .errors
            .iter()
            .map(|error| {
                let with_source = error.clone().with_source_code(source.to_string());
                format!("{with_source:?}")
            })
            .collect::<Vec<_>>();
        format!("Failed to transpile TypeScript for {specifier}: {}", rendered.join("\n"))
    }
}

impl CompilerInterface for TsCompiler {
    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn enable_sourcemap(&self) -> bool {
        true
    }

    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.options)
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(CodegenOptions::default())
    }

    fn after_codegen(&mut self, ret: CodegenReturn) {
        self.output = ret.code;
        self.source_map = ret.map;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_typescript_with_interface() {
        let code = r#"
interface User {
  name: string;
  age: number;
}
        "#;
        assert!(detect_typescript(code));
    }

    #[test]
    fn test_detect_typescript_with_type_annotation() {
        let code = r#"
function greet(user: User): string {
  return `Hello, ${user.name}!`;
}
        "#;
        assert!(detect_typescript(code));
    }

    #[test]
    fn test_detect_typescript_with_enum() {
        let code = r#"
enum Color {
  Red,
  Green,
  Blue
}
        "#;
        assert!(detect_typescript(code));
    }

    #[test]
    fn test_detect_javascript() {
        let code = r#"
function greet(user) {
  return `Hello, ${user.name}!`;
}
        "#;
        assert!(!detect_typescript(code));
    }

    #[test]
    fn test_detect_javascript_with_comments() {
        let code = r#"
// This is a User type
function greet(user) {
  return `Hello, ${user.name}!`;
}
        "#;
        assert!(!detect_typescript(code));
    }
}
