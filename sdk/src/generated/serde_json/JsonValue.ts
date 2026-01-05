// Type definition for serde_json::Value
// This represents arbitrary JSON values
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue }
