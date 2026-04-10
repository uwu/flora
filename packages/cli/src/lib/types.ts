export type CliConfig = {
  apiUrl: string
  token?: string
}

export type DeployProjectConfig = {
  entry?: string
  root?: string
}

export type DeploySourceMapMode = 'none' | 'inline' | 'external'

export const DEFAULT_API_URL = 'https://app.flora.uwu.network/api'
