import { dirname, resolve } from 'node:path'
import ts from 'typescript'

export function buildRuntimeGlobals(sdkInput: string): string {
  const sdkPath = resolve(process.cwd(), sdkInput)
  const configPath = ts.findConfigFile(dirname(sdkPath), ts.sys.fileExists, 'tsconfig.json')
  const configFile = configPath ? ts.readConfigFile(configPath, ts.sys.readFile) : { config: {} }
  const parsedConfig = ts.parseJsonConfigFileContent(
    configFile.config,
    ts.sys,
    dirname(configPath || sdkPath)
  )

  const compilerOptions: ts.CompilerOptions = {
    ...parsedConfig.options,
    declaration: true,
    emitDeclarationOnly: true,
    noEmit: true
  }

  const host = ts.createCompilerHost(compilerOptions)
  const program = ts.createProgram([sdkPath], compilerOptions, host)
  const checker = program.getTypeChecker()
  const sourceFile = program.getSourceFile(sdkPath)
  if (!sourceFile) return wrapGlobals([])

  const moduleSymbol = checker.getSymbolAtLocation(sourceFile)
  if (!moduleSymbol) return wrapGlobals([])

  const exports = checker.getExportsOfModule(moduleSymbol)
  const names = exports
    .filter(symbolHasValueDeclaration)
    .map((symbol) => symbol.getName())
    .filter((name) => name && name !== 'default')
    .sort((a, b) => a.localeCompare(b))

  return wrapGlobals(names)
}

function symbolHasValueDeclaration(symbol: ts.Symbol): boolean {
  const declarations = symbol.getDeclarations()
  if (!declarations) return false
  return declarations.some((decl) => isValueDeclaration(decl))
}

function isValueDeclaration(decl: ts.Declaration): boolean {
  return (
    ts.isFunctionDeclaration(decl) ||
    ts.isVariableDeclaration(decl) ||
    ts.isClassDeclaration(decl) ||
    ts.isEnumDeclaration(decl)
  )
}

function wrapGlobals(names: string[]): string {
  const lines = names.map((name) => `  global.${name} = global.flora.${name};`)
  return `
;(function (global) {
  if (!global.flora) return;
${lines.join('\n')}
})(globalThis);
`
}
