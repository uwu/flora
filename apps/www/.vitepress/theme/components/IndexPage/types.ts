export type FeatureId = 'sdk' | 'cli' | 'runtime'

export interface IndexFeature {
  id: FeatureId
  title: string
  desc: string
  href: string
  linkLabel: string
}
