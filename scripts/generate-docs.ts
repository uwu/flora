#!/usr/bin/env bun
/**
 * Generate Markdown API documentation from rustdoc JSON output.
 *
 * Usage:
 *   1. Generate rustdoc JSON: RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo +nightly doc --no-deps
 *   2. Run this script: bun run scripts/generate-docs.ts
 *
 * Output: docs/api.md
 */

import { readFileSync, writeFileSync, mkdirSync } from 'fs'
import { dirname, join } from 'path'

const RUSTDOC_JSON = 'target/doc/flora.json'
const OUTPUT_FILE = 'docs/api.md'

interface RustdocJson {
  crate_version: string
  index: Record<string, Item>
  paths: Record<string, { path: string[] }>
}

interface Item {
  id: number
  name: string
  docs: string | null
  span: { filename: string; begin: [number, number]; end: [number, number] }
  inner: {
    struct?: { kind: { plain?: { fields: number[] } } }
    struct_field?: { resolved_path?: { path: string } } | { primitive?: string }
    function?: { sig: { inputs: Array<{ name: string; type: any }> } }
  }
}

function main() {
  console.log('Reading rustdoc JSON...')

  let doc: RustdocJson
  try {
    doc = JSON.parse(readFileSync(RUSTDOC_JSON, 'utf-8'))
  } catch (e) {
    console.error(`Error reading ${RUSTDOC_JSON}. Run this first:`)
    console.error('  RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo +nightly doc --no-deps')
    process.exit(1)
  }

  const lines: string[] = []
  lines.push('# Flora SDK API Reference')
  lines.push('')
  lines.push(`Generated from Flora v${doc.crate_version}`)
  lines.push('')

  // Find all exported structs/types we care about
  const payloadTypes = findItemsByPattern(doc, 'Payload', ['discord_handler'])
  const opInputTypes = findItemsByPattern(doc, 'Input', ['ops'])
  const opArgsTypes = findItemsByPattern(doc, 'Args', ['ops'])
  const kvTypes = findItemsByPattern(doc, ['KvKeyInfo', 'KvKeyMetadata', 'ListKeysResult', 'SetOptions', 'ListKeysOptions'], ['kv', 'ops'])

  // Document Event Payloads
  lines.push('## Event Payloads')
  lines.push('')
  lines.push('These types represent the data received in event handlers.')
  lines.push('')

  for (const item of payloadTypes) {
    documentStruct(doc, item, lines)
  }

  // Document Op Input Types
  lines.push('## Op Input Types')
  lines.push('')
  lines.push('These types represent the arguments passed to runtime operations.')
  lines.push('')

  for (const item of [...opInputTypes, ...opArgsTypes]) {
    documentStruct(doc, item, lines)
  }

  // Document KV Types
  lines.push('## KV Store Types')
  lines.push('')
  lines.push('Types for the key-value store API.')
  lines.push('')

  for (const item of kvTypes) {
    documentStruct(doc, item, lines)
  }

  // Write output
  mkdirSync(dirname(OUTPUT_FILE), { recursive: true })
  writeFileSync(OUTPUT_FILE, lines.join('\n'))
  console.log(`Documentation written to ${OUTPUT_FILE}`)
}

function findItemsByPattern(doc: RustdocJson, pattern: string | string[], modules: string[]): Item[] {
  const patterns = Array.isArray(pattern) ? pattern : [pattern]
  const items: Item[] = []

  for (const [id, item] of Object.entries(doc.index)) {
    if (!item.name) continue
    if (!item.inner?.struct) continue

    const matches = patterns.some(p => item.name.includes(p) || item.name === p)
    if (!matches) continue

    const inModule = modules.some(m => item.span?.filename?.includes(m))
    if (!inModule) continue

    items.push(item)
  }

  return items.sort((a, b) => a.name.localeCompare(b.name))
}

function documentStruct(doc: RustdocJson, item: Item, lines: string[]) {
  lines.push(`### \`${item.name}\``)
  lines.push('')

  if (item.docs) {
    lines.push(item.docs.trim())
    lines.push('')
  }

  const fields = item.inner?.struct?.kind?.plain?.fields
  if (!fields || fields.length === 0) {
    lines.push('_No fields_')
    lines.push('')
    return
  }

  lines.push('| Field | Type | Description |')
  lines.push('|-------|------|-------------|')

  for (const fieldId of fields) {
    const field = doc.index[String(fieldId)]
    if (!field) continue

    const fieldName = field.name
    const fieldType = getFieldType(field)
    const fieldDocs = field.docs?.trim().replace(/\n/g, ' ') || ''

    lines.push(`| \`${fieldName}\` | \`${fieldType}\` | ${fieldDocs} |`)
  }

  lines.push('')
}

function getFieldType(field: Item): string {
  const inner = field.inner?.struct_field
  if (!inner) return 'unknown'

  if ('resolved_path' in inner && inner.resolved_path) {
    return inner.resolved_path.path
  }
  if ('primitive' in inner && inner.primitive) {
    return inner.primitive
  }

  // Try to extract from other type representations
  return 'unknown'
}

main()
