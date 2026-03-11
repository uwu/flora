export type DeploymentChangeSummary = {
  added_files: number
  removed_files: number
  modified_files: number
}

export type DeploymentRevision = {
  id: string
  guild_id: string
  entry: string
  status: string
  deployed_at: string
  deploy_source: string
  actor: {
    user_id?: string | null
    username?: string | null
    actor_type: string
  }
  error_message?: string | null
  build_id?: string | null
  base_revision_id?: string | null
  change_summary?: DeploymentChangeSummary | null
  files?: Array<{ path: string; contents: string }> | null
}
