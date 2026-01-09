import type { Plugin } from 'rolldown'
import { resolve, dirname } from 'node:path'
import ts from 'typescript'

export function globalTypes(options: {
  input: string
  output: string
}): Plugin {
  return {
    name: 'global-types',
    async buildEnd() {
      const { input, output } = options
      const inputPath = resolve(process.cwd(), input)

      const declarations = generateBundledDeclarations(inputPath)

      const outputPath = resolve(process.cwd(), output)
      const { writeFileSync, mkdirSync } = await import('node:fs')
      mkdirSync(dirname(outputPath), { recursive: true })
      writeFileSync(outputPath, declarations)

      console.log(`Generated bundled global types: ${output}`)
    }
  }
}

function generateBundledDeclarations(entryPath: string): string {
  const configPath = ts.findConfigFile(dirname(entryPath), ts.sys.fileExists, 'tsconfig.json')
  const configFile = configPath ? ts.readConfigFile(configPath, ts.sys.readFile) : { config: {} }
  const parsedConfig = ts.parseJsonConfigFileContent(
    configFile.config,
    ts.sys,
    dirname(configPath || entryPath)
  )

  const compilerOptions: ts.CompilerOptions = {
    ...parsedConfig.options,
    declaration: true,
    emitDeclarationOnly: true,
    noEmit: false,
    outDir: undefined,
    declarationDir: undefined
  }

  const host = ts.createCompilerHost(compilerOptions)
  const program = ts.createProgram([entryPath], compilerOptions, host)
  const checker = program.getTypeChecker()

  const collectedDeclarations: string[] = []
  const processedSymbols = new Set<string>()
  const exportedNames: string[] = []

  function getFullyQualifiedType(type: ts.Type, node?: ts.Node): string {
    let result = checker.typeToString(
      type,
      node,
      ts.TypeFormatFlags.NoTruncation |
      ts.TypeFormatFlags.WriteArrayAsGenericType |
      ts.TypeFormatFlags.InTypeAlias
    )
    result = result.replace(/import\([^)]+\)\./g, '')
    return result
  }

  function processSymbol(symbol: ts.Symbol) {
    const name = symbol.getName()
    if (processedSymbols.has(name)) return
    processedSymbols.add(name)

    const declarations = symbol.getDeclarations()
    if (!declarations || declarations.length === 0) return

    const decl = declarations[0]
    if (!decl) return

    // Type alias
    if (ts.isTypeAliasDeclaration(decl)) {
      const type = checker.getTypeAtLocation(decl)
      const typeParams = decl.typeParameters
        ? `<${decl.typeParameters.map(tp => tp.getText()).join(', ')}>`
        : ''
      const typeString = getFullyQualifiedType(type, decl)
      collectedDeclarations.push(`  type ${name}${typeParams} = ${typeString};`)
    }
    // Interface
    else if (ts.isInterfaceDeclaration(decl)) {
      const type = checker.getTypeAtLocation(decl)
      const props = type.getProperties()
      const typeParams = decl.typeParameters
        ? `<${decl.typeParameters.map(tp => tp.getText()).join(', ')}>`
        : ''

      const members: string[] = []
      for (const prop of props) {
        const propDecl = prop.getDeclarations()?.[0]
        if (propDecl && ts.isPropertySignature(propDecl)) {
          const propType = checker.getTypeOfSymbolAtLocation(prop, propDecl)
          const optional = prop.flags & ts.SymbolFlags.Optional ? '?' : ''
          members.push(`    ${prop.getName()}${optional}: ${getFullyQualifiedType(propType, propDecl)};`)
        }
      }

      collectedDeclarations.push(`  interface ${name}${typeParams} {\n${members.join('\n')}\n  }`)
    }
    // Function
    else if (ts.isFunctionDeclaration(decl)) {
      const signature = checker.getSignatureFromDeclaration(decl)
      if (signature) {
        const returnType = checker.getReturnTypeOfSignature(signature)
        const params = signature.getParameters().map(p => {
          const paramDecl = p.getDeclarations()?.[0]
          const paramType = paramDecl ? checker.getTypeOfSymbolAtLocation(p, paramDecl) : checker.getAnyType()
          const isOptional = p.flags & ts.SymbolFlags.Optional ||
            (paramDecl && ts.isParameter(paramDecl) && (paramDecl.questionToken !== undefined || paramDecl.initializer !== undefined))
          const optionalMarker = isOptional ? '?' : ''
          return `${p.getName()}${optionalMarker}: ${getFullyQualifiedType(paramType, paramDecl)}`
        }).join(', ')

        const typeParams = decl.typeParameters
          ? `<${decl.typeParameters.map(tp => tp.getText()).join(', ')}>`
          : ''

        collectedDeclarations.push(`  function ${name}${typeParams}(${params}): ${getFullyQualifiedType(returnType, decl)};`)
        exportedNames.push(name)
      }
    }
    // Variable/const
    else if (ts.isVariableDeclaration(decl)) {
      const type = checker.getTypeAtLocation(decl)
      collectedDeclarations.push(`  const ${name}: ${getFullyQualifiedType(type, decl)};`)
      exportedNames.push(name)
    }
    // Class
    else if (ts.isClassDeclaration(decl)) {
      const type = checker.getTypeAtLocation(decl)
      const typeParams = decl.typeParameters
        ? `<${decl.typeParameters.map(tp => tp.getText()).join(', ')}>`
        : ''

      const constructSignatures = type.getConstructSignatures()
      const members: string[] = []

      for (const sig of constructSignatures) {
        const params = sig.getParameters().map(p => {
          const paramDecl = p.getDeclarations()?.[0]
          const paramType = paramDecl ? checker.getTypeOfSymbolAtLocation(p, paramDecl) : checker.getAnyType()
          return `${p.getName()}: ${getFullyQualifiedType(paramType, paramDecl)}`
        }).join(', ')
        members.push(`    constructor(${params});`)
      }

      const instanceType = checker.getDeclaredTypeOfSymbol(symbol)
      for (const prop of instanceType.getProperties()) {
        const propDecl = prop.getDeclarations()?.[0]
        if (!propDecl) continue

        // Skip private members
        const hasPrivateModifier = ts.canHaveModifiers(propDecl) &&
          ts.getModifiers(propDecl)?.some((m: ts.Modifier) => m.kind === ts.SyntaxKind.PrivateKeyword)
        if (hasPrivateModifier) continue
        if (prop.getName().startsWith('#')) continue

        const propType = checker.getTypeOfSymbolAtLocation(prop, propDecl)

        if (ts.isMethodDeclaration(propDecl) || ts.isMethodSignature(propDecl)) {
          const sig = checker.getSignatureFromDeclaration(propDecl)
          if (sig) {
            const returnType = checker.getReturnTypeOfSignature(sig)
            const params = sig.getParameters().map(p => {
              const pd = p.getDeclarations()?.[0]
              const pt = pd ? checker.getTypeOfSymbolAtLocation(p, pd) : checker.getAnyType()
              const isOptional = p.flags & ts.SymbolFlags.Optional ||
                (pd && ts.isParameter(pd) && (pd.questionToken !== undefined || pd.initializer !== undefined))
              const optionalMarker = isOptional ? '?' : ''
              return `${p.getName()}${optionalMarker}: ${getFullyQualifiedType(pt, pd)}`
            }).join(', ')
            members.push(`    ${prop.getName()}(${params}): ${getFullyQualifiedType(returnType, propDecl)};`)
          }
        } else {
          members.push(`    ${prop.getName()}: ${getFullyQualifiedType(propType, propDecl)};`)
        }
      }

      collectedDeclarations.push(`  class ${name}${typeParams} {\n${members.join('\n')}\n  }`)
      exportedNames.push(name)
    }
  }

  // Process exports from entry file
  const sourceFile = program.getSourceFile(entryPath)
  if (sourceFile) {
    const moduleSymbol = checker.getSymbolAtLocation(sourceFile)
    if (moduleSymbol) {
      const exports = checker.getExportsOfModule(moduleSymbol)
      for (const exp of exports) {
        processSymbol(exp)
      }
    }
  }

  const runtimeGlobals = `
  // Runtime globals (from runtime_prelude.js)
  interface FloraEventMap {
    ready: BaseContext<EventReady>
    messageCreate: MessageContext
    messageUpdate: MessageUpdateContext
    messageDelete: MessageDeleteContext
    messageDeleteBulk: MessageDeleteBulkContext
    interactionCreate: InteractionContext
  }

  function on<E extends keyof FloraEventMap>(event: E, handler: (ctx: FloraEventMap[E]) => void | Promise<void>): void
  function registerSlashCommands(commands: FlattenedSlashCommand[]): void

  const __floraHandlers: Record<string, Function[]>
  const __floraGuildId: string | undefined
  function __floraDispatch(event: string, payload: unknown): Promise<void>

  const flora: typeof import('./src/index')
`

  return `// Auto-generated global types for Flora SDK
// Do not edit manually - regenerate with \`bun run build\`

declare global {
${runtimeGlobals}

  // SDK exports (functions, consts, classes, types)
${collectedDeclarations.join('\n\n')}
}

export {}
`
}
