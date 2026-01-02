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

/// Detect whether source code contains TypeScript-specific syntax.
pub fn detect_typescript(source: &str) -> bool {
    use oxc::{
        allocator::Allocator,
        ast_visit::Visit,
        parser::{Parser, ParserReturn},
    };

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true).with_module(true);
    let parser = Parser::new(&allocator, source, source_type);
    let ParserReturn { errors, program, .. } = parser.parse();

    if !errors.is_empty() {
        return true;
    }

    struct TypeScriptDetector {
        has_typescript: bool,
    }

    impl<'a> Visit<'a> for TypeScriptDetector {
        fn visit_ts_type(&mut self, _ty: &oxc::ast::ast::TSType<'a>) {
            self.has_typescript = true;
        }

        fn visit_ts_interface_declaration(
            &mut self,
            _decl: &oxc::ast::ast::TSInterfaceDeclaration<'a>,
        ) {
            self.has_typescript = true;
        }

        fn visit_ts_type_alias_declaration(
            &mut self,
            _decl: &oxc::ast::ast::TSTypeAliasDeclaration<'a>,
        ) {
            self.has_typescript = true;
        }

        fn visit_ts_enum_declaration(&mut self, _decl: &oxc::ast::ast::TSEnumDeclaration<'a>) {
            self.has_typescript = true;
        }

        fn visit_ts_module_declaration(&mut self, _decl: &oxc::ast::ast::TSModuleDeclaration<'a>) {
            self.has_typescript = true;
        }
    }

    let mut detector = TypeScriptDetector { has_typescript: false };
    detector.visit_program(&program);

    detector.has_typescript
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
