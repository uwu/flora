import type {
  JsonValue,
  RawKvKeyMetadata,
  RawKvListKeysOptions,
  RawKvListKeysResult,
  RawKvSetOptions
} from '../generated'

type KvEntry = {
  value: string
  expiration?: bigint
  metadata?: JsonValue
}

export class KvMock {
  stores = new Map<string, Map<string, KvEntry>>()

  private getStore(name: string): Map<string, KvEntry> {
    let store = this.stores.get(name)
    if (!store) {
      store = new Map()
      this.stores.set(name, store)
    }
    return store
  }

  get(storeName: string, key: string): string | null {
    const entry = this.getStore(storeName).get(key)
    return entry?.value ?? null
  }

  getWithMetadata(storeName: string, key: string): [string, RawKvKeyMetadata | null] | null {
    const entry = this.getStore(storeName).get(key)
    if (!entry) return null
    const meta: RawKvKeyMetadata | null =
      entry.expiration !== undefined || entry.metadata !== undefined
        ? { expiration: entry.expiration, metadata: entry.metadata }
        : null
    return [entry.value, meta]
  }

  set(storeName: string, key: string, value: string, options: RawKvSetOptions): void {
    this.getStore(storeName).set(key, {
      value,
      expiration: options.expiration ?? undefined,
      metadata: options.metadata ?? undefined
    })
  }

  updateMetadata(storeName: string, key: string, metadata: JsonValue | undefined): void {
    const store = this.getStore(storeName)
    const entry = store.get(key)
    if (!entry) return
    entry.metadata = metadata
  }

  delete(storeName: string, key: string): void {
    this.getStore(storeName).delete(key)
  }

  listKeys(options: RawKvListKeysOptions, storeName: string): RawKvListKeysResult {
    const store = this.getStore(storeName)
    let keys = Array.from(store.entries())
      .map(([name, entry]) => ({
        name,
        expiration: entry.expiration,
        metadata: entry.metadata
      }))
      .sort((a, b) => a.name.localeCompare(b.name))

    if (options.prefix) {
      keys = keys.filter((k) => k.name.startsWith(options.prefix!))
    }

    let startIndex = 0
    if (options.cursor) {
      startIndex = keys.findIndex((k) => k.name > options.cursor!)
      if (startIndex === -1) {
        return { keys: [], listComplete: true, cursor: undefined }
      }
    }

    const limit = Number(options.limit ?? 100n)
    const slice = keys.slice(startIndex, startIndex + limit)
    const listComplete = startIndex + limit >= keys.length

    return {
      keys: slice,
      listComplete,
      cursor: listComplete ? undefined : slice[slice.length - 1]!.name
    }
  }

  clear(): void {
    this.stores.clear()
  }
}
