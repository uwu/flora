import { dirname, resolve } from 'node:path'
import type { Plugin } from 'rolldown'
import ts from 'typescript'

export function globalTypes(options: {
  sdkInput: string
  runtimeInput: string
  output: string
}): Plugin {
  return {
    name: 'global-types',
    async buildEnd() {
      const { sdkInput, runtimeInput, output } = options
      const sdkPath = resolve(process.cwd(), sdkInput)
      const runtimePath = resolve(process.cwd(), runtimeInput)

      const declarations = generateBundledDeclarations(sdkPath, runtimePath)

      const outputPath = resolve(process.cwd(), output)
      const { writeFileSync, mkdirSync } = await import('node:fs')
      mkdirSync(dirname(outputPath), { recursive: true })
      writeFileSync(outputPath, declarations)

      console.log(`Generated bundled global types: ${output}`)
    }
  }
}

function generateBundledDeclarations(sdkPath: string, runtimePath: string): string {
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
    noEmit: false,
    outDir: undefined,
    declarationDir: undefined
  }

  const host = ts.createCompilerHost(compilerOptions)
  const program = ts.createProgram([sdkPath, runtimePath], compilerOptions, host)
  const checker = program.getTypeChecker()

  const sdkDeclarations: string[] = []
  const runtimeDeclarations: string[] = []
  const processedSymbols = new Map<string, Set<string>>()

  function hasProcessed(name: string, kind: string): boolean {
    return processedSymbols.get(name)?.has(kind) ?? false
  }

  function markProcessed(name: string, kind: string): void {
    const entry = processedSymbols.get(name) ?? new Set<string>()
    entry.add(kind)
    processedSymbols.set(name, entry)
  }

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

  function processSymbol(symbol: ts.Symbol, target: string[]): void {
    const name = symbol.getName()

    const declarations = symbol.getDeclarations()
    if (!declarations || declarations.length === 0) return

    for (const decl of declarations) {
      if (!decl) continue

      // Type alias
      if (ts.isTypeAliasDeclaration(decl)) {
        if (hasProcessed(name, 'type')) continue
        markProcessed(name, 'type')
        const type = checker.getTypeAtLocation(decl)
        const typeParams = decl.typeParameters
          ? `<${decl.typeParameters.map((tp) => tp.getText()).join(', ')}>`
          : ''
        const typeString = getFullyQualifiedType(type, decl)
        target.push(`  type ${name}${typeParams} = ${typeString};`)
      } // Interface
      else if (ts.isInterfaceDeclaration(decl)) {
        if (hasProcessed(name, 'type')) continue
        markProcessed(name, 'type')
        const type = checker.getTypeAtLocation(decl)
        const props = type.getProperties()
        const typeParams = decl.typeParameters
          ? `<${decl.typeParameters.map((tp) => tp.getText()).join(', ')}>`
          : ''

        const members: string[] = []
        for (const prop of props) {
          const propDecl = prop.getDeclarations()?.[0]
          if (propDecl && ts.isPropertySignature(propDecl)) {
            const propType = checker.getTypeOfSymbolAtLocation(prop, propDecl)
            const optional = prop.flags & ts.SymbolFlags.Optional ? '?' : ''
            members.push(
              `    ${prop.getName()}${optional}: ${getFullyQualifiedType(propType, propDecl)};`
            )
          }
        }

        target.push(`  interface ${name}${typeParams} {\n${members.join('\n')}\n  }`)
      } // Function
      else if (ts.isFunctionDeclaration(decl)) {
        if (hasProcessed(name, 'value')) continue
        markProcessed(name, 'value')
        const signature = checker.getSignatureFromDeclaration(decl)
        if (signature) {
          const returnType = checker.getReturnTypeOfSignature(signature)
          const params = signature
            .getParameters()
            .map((p) => {
              const paramDecl = p.getDeclarations()?.[0]
              const paramType = paramDecl
                ? checker.getTypeOfSymbolAtLocation(p, paramDecl)
                : checker.getAnyType()
              const isOptional =
                p.flags & ts.SymbolFlags.Optional ||
                (paramDecl &&
                  ts.isParameter(paramDecl) &&
                  (paramDecl.questionToken !== undefined || paramDecl.initializer !== undefined))
              const optionalMarker = isOptional ? '?' : ''
              return `${p.getName()}${optionalMarker}: ${getFullyQualifiedType(paramType, paramDecl)}`
            })
            .join(', ')

          const typeParams = decl.typeParameters
            ? `<${decl.typeParameters.map((tp) => tp.getText()).join(', ')}>`
            : ''

          target.push(
            `  function ${name}${typeParams}(${params}): ${getFullyQualifiedType(
              returnType,
              decl
            )};`
          )
        }
      } // Variable/const
      else if (ts.isVariableDeclaration(decl)) {
        if (hasProcessed(name, 'value')) continue
        markProcessed(name, 'value')
        const type = checker.getTypeAtLocation(decl)
        const isLet =
          decl.parent &&
          ts.isVariableDeclarationList(decl.parent) &&
          (decl.parent.flags & ts.NodeFlags.Let) !== 0
        const keyword = isLet ? 'let' : 'const'
        target.push(`  ${keyword} ${name}: ${getFullyQualifiedType(type, decl)};`)
      } // Class
      else if (ts.isClassDeclaration(decl)) {
        if (hasProcessed(name, 'value')) continue
        markProcessed(name, 'value')
        const type = checker.getTypeAtLocation(decl)
        const typeParams = decl.typeParameters
          ? `<${decl.typeParameters.map((tp) => tp.getText()).join(', ')}>`
          : ''

        const constructSignatures = type.getConstructSignatures()
        const members: string[] = []

        for (const sig of constructSignatures) {
          const params = sig
            .getParameters()
            .map((p) => {
              const paramDecl = p.getDeclarations()?.[0]
              const paramType = paramDecl
                ? checker.getTypeOfSymbolAtLocation(p, paramDecl)
                : checker.getAnyType()
              return `${p.getName()}: ${getFullyQualifiedType(paramType, paramDecl)}`
            })
            .join(', ')
          members.push(`    constructor(${params});`)
        }

        const instanceType = checker.getDeclaredTypeOfSymbol(symbol)
        for (const prop of instanceType.getProperties()) {
          const propDecl = prop.getDeclarations()?.[0]
          if (!propDecl) continue

          // Skip private members
          const hasPrivateModifier =
            ts.canHaveModifiers(propDecl) &&
            ts
              .getModifiers(propDecl)
              ?.some((m: ts.Modifier) => m.kind === ts.SyntaxKind.PrivateKeyword)
          if (hasPrivateModifier) continue
          if (prop.getName().startsWith('#')) continue

          const propType = checker.getTypeOfSymbolAtLocation(prop, propDecl)

          if (ts.isMethodDeclaration(propDecl) || ts.isMethodSignature(propDecl)) {
            const sig = checker.getSignatureFromDeclaration(propDecl)
            if (sig) {
              const returnType = checker.getReturnTypeOfSignature(sig)
              const params = sig
                .getParameters()
                .map((p) => {
                  const pd = p.getDeclarations()?.[0]
                  const pt = pd ? checker.getTypeOfSymbolAtLocation(p, pd) : checker.getAnyType()
                  const isOptional =
                    p.flags & ts.SymbolFlags.Optional ||
                    (pd &&
                      ts.isParameter(pd) &&
                      (pd.questionToken !== undefined || pd.initializer !== undefined))
                  const optionalMarker = isOptional ? '?' : ''
                  return `${p.getName()}${optionalMarker}: ${getFullyQualifiedType(pt, pd)}`
                })
                .join(', ')
              members.push(
                `    ${prop.getName()}(${params}): ${getFullyQualifiedType(returnType, propDecl)};`
              )
            }
          } else {
            members.push(`    ${prop.getName()}: ${getFullyQualifiedType(propType, propDecl)};`)
          }
        }

        target.push(`  class ${name}${typeParams} {\n${members.join('\n')}\n  }`)
      }
    }
  }

  // Process SDK exports
  const sdkSourceFile = program.getSourceFile(sdkPath)
  if (sdkSourceFile) {
    const moduleSymbol = checker.getSymbolAtLocation(sdkSourceFile)
    if (moduleSymbol) {
      const exports = checker.getExportsOfModule(moduleSymbol)
      for (const exp of exports) {
        processSymbol(exp, sdkDeclarations)
      }
    }
  }

  // Process runtime exports
  const runtimeSourceFile = program.getSourceFile(runtimePath)
  if (runtimeSourceFile) {
    const moduleSymbol = checker.getSymbolAtLocation(runtimeSourceFile)
    if (moduleSymbol) {
      const exports = checker.getExportsOfModule(moduleSymbol)
      for (const exp of exports) {
        processSymbol(exp, runtimeDeclarations)
      }
    }

    // Extract declare global block contents
    ts.forEachChild(runtimeSourceFile, (node) => {
      if (ts.isModuleDeclaration(node) && node.name.getText() === 'global') {
        const body = node.body
        if (body && ts.isModuleBlock(body)) {
          for (const statement of body.statements) {
            if (ts.isVariableStatement(statement)) {
              for (const decl of statement.declarationList.declarations) {
                const name = decl.name.getText()
                if (hasProcessed(name, 'value')) continue
                markProcessed(name, 'value')
                const type = checker.getTypeAtLocation(decl)
                const keyword = statement.declarationList.flags & ts.NodeFlags.Let ? 'let' : 'var'
                runtimeDeclarations.push(
                  `  ${keyword} ${name}: ${getFullyQualifiedType(type, decl)};`
                )
              }
            } else if (ts.isFunctionDeclaration(statement) && statement.name) {
              const name = statement.name.getText()
              if (hasProcessed(name, 'value')) continue
              markProcessed(name, 'value')
              const signature = checker.getSignatureFromDeclaration(statement)
              if (signature) {
                const returnType = checker.getReturnTypeOfSignature(signature)
                const params = signature
                  .getParameters()
                  .map((p) => {
                    const paramDecl = p.getDeclarations()?.[0]
                    const paramType = paramDecl
                      ? checker.getTypeOfSymbolAtLocation(p, paramDecl)
                      : checker.getAnyType()
                    const isOptional =
                      p.flags & ts.SymbolFlags.Optional ||
                      (paramDecl &&
                        ts.isParameter(paramDecl) &&
                        (paramDecl.questionToken !== undefined ||
                          paramDecl.initializer !== undefined))
                    const optionalMarker = isOptional ? '?' : ''
                    return `${p.getName()}${optionalMarker}: ${getFullyQualifiedType(
                      paramType,
                      paramDecl
                    )}`
                  })
                  .join(', ')

                const typeParams = statement.typeParameters
                  ? `<${statement.typeParameters.map((tp) => tp.getText()).join(', ')}>`
                  : ''

                runtimeDeclarations.push(
                  `  function ${name}${typeParams}(${params}): ${getFullyQualifiedType(
                    returnType,
                    statement
                  )};`
                )
              }
            }
          }
        }
      }
    })
  }

  return `// Auto-generated global types for Flora SDK
// Do not edit manually - regenerate with \`bun run build\`

declare global {
  // Runtime exports (from runtime/index.ts)
${runtimeDeclarations.join('\n\n')}

  // SDK exports (functions, consts, classes, types)
${sdkDeclarations.join('\n\n')}

  const flora: typeof import('./src/index')
}

export {}
`
}
