export type CliConfig = {
  apiUrl: string
  token?: string
}

export type DeploySourceMapMode = 'none' | 'inline' | 'external'

export type DeployProjectConfig = {
  entry?: string
  root?: string
  sourcemap?: DeploySourceMapMode
  minify?: boolean
  external?: string[]
}

export const DEFAULT_API_URL = 'http://localhost:3000/api'
