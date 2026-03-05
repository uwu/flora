type PackageShimOptions = {
  package: string
  path: string
}

export function packageShimPlugin(options: PackageShimOptions) {
  const { package: packageName, path: shimPath } = options

  if (!packageName || !shimPath) {
    throw new Error(
      'packageShimPlugin requires both "package" and "path" options'
    )
  }

  return {
    name: 'package-shim-plugin',

    resolveId(source: string) {
      if (source === packageName) {
        return {
          id: shimPath,
          external: false
        }
      }

      if (source.startsWith(`${packageName}/`)) {
        const subPath = source.slice(packageName.length)
        return {
          id: shimPath + subPath,
          external: false
        }
      }

      return null
    }
  }
}
