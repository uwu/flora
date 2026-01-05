/**
 * KV Store API for Flora
 *
 * Provides a simple key-value store interface per guild with named stores.
 * Supports cursor-based pagination and optional metadata on keys.
 */

// SDK-friendly types (use number instead of bigint for convenience)
export interface ListKeysOptions {
  prefix?: string
  limit?: number
  cursor?: string
}

export interface KvKeyInfo {
  name: string
  expiration?: number
  metadata?: Record<string, unknown>
}

export interface ListKeysResult {
  keys: KvKeyInfo[]
  list_complete: boolean
  cursor: string | null
}

export interface GetResult {
  value: string | null
  metadata?: Record<string, unknown>
}

// Declare Deno.core.ops for the runtime environment
declare const Deno: {
  core: {
    ops: {
      op_kv_get(storeName: string, key: string): Promise<string | null>
      op_kv_get_with_metadata(storeName: string, key: string): Promise<[string, { expiration?: number | null; metadata?: unknown } | null] | null>
      op_kv_set(storeName: string, key: string, value: string, options: { expiration: number | null; metadata: unknown }): Promise<void>
      op_kv_update_metadata(storeName: string, key: string, metadata: unknown): Promise<void>
      op_kv_delete(storeName: string, key: string): Promise<void>
      op_kv_list_keys(options: { prefix: string | null; limit: number | null; cursor: string | null }, storeName: string): Promise<ListKeysResult>
    }
  }
}

export class KvStore {
  #storeName: string

  constructor(storeName: string) {
    this.#storeName = storeName
  }

  /**
   * Get a value from the store.
   *
   * @param key - The key to retrieve
   * @returns The value, or null if not found
   */
  async get(key: string): Promise<string | null> {
    return await Deno.core.ops.op_kv_get(this.#storeName, key)
  }

  /**
   * Get a value from the store along with its metadata.
   *
   * @param key - The key to retrieve
   * @returns Object with value and optional metadata
   */
  async getWithMetadata(key: string): Promise<GetResult> {
    const result = await Deno.core.ops.op_kv_get_with_metadata(this.#storeName, key)
    if (result === null) {
      return { value: null }
    }
    const [value, metadata] = result
    return { 
      value, 
      metadata: metadata?.metadata as Record<string, unknown> | undefined 
    }
  }

  /**
   * Set a value in the store.
   *
   * The value size is limited to 1MB.
   *
   * @param key - The key to set
   * @param value - The value to store (max 1MB)
   * @param options - Optional expiration (Unix timestamp) and metadata
   */
  async set(
    key: string,
    value: string,
    options?: { expiration?: number; metadata?: Record<string, unknown> }
  ): Promise<void> {
    await Deno.core.ops.op_kv_set(this.#storeName, key, value, {
      expiration: options?.expiration ?? null,
      metadata: options?.metadata ?? null,
    })
  }

  /**
   * Update just the metadata for a key without changing the value.
   *
   * @param key - The key to update
   * @param metadata - The metadata to set, or null to remove metadata
   */
  async updateMetadata(
    key: string,
    metadata: Record<string, unknown> | null
  ): Promise<void> {
    await Deno.core.ops.op_kv_update_metadata(this.#storeName, key, metadata ?? null)
  }

  /**
   * Delete a key from the store.
   *
   * @param key - The key to delete
   */
  async delete(key: string): Promise<void> {
    await Deno.core.ops.op_kv_delete(this.#storeName, key)
  }

  /**
   * List all keys in the store with cursor-based pagination.
   *
   * @param options - Optional prefix filter, limit (default 100, max 1000), and cursor for pagination
   * @returns Paginated result with keys, list_complete flag, and cursor for next page
   */
  async list(options?: ListKeysOptions): Promise<ListKeysResult> {
    return await Deno.core.ops.op_kv_list_keys(
      {
        prefix: options?.prefix ?? null,
        limit: options?.limit ?? null,
        cursor: options?.cursor ?? null,
      },
      this.#storeName
    )
  }
}

export function store(name: string): KvStore {
  return new KvStore(name)
}

export const kv = {
  store,
}
