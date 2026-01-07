/**
 * KV Store API for Flora
 *
 * Provides a simple key-value store interface per guild with named stores.
 * Supports cursor-based pagination and optional metadata on keys.
 */

import type { KvKeyMetadata } from '../generated/KvKeyMetadata'
import type { ListKeysOptions } from '../generated/ListKeysOptions'
import type { ListKeysResult } from '../generated/ListKeysResult'
import type { JsonValue } from '../generated/serde_json/JsonValue'

export interface GetResult {
  value: string | null
  metadata?: Record<string, unknown>
}

// Declare Deno.core.ops for the runtime environment
declare const Deno: {
  core: {
    ops: {
      op_kv_get(storeName: string, key: string): Promise<string | null>
      op_kv_get_with_metadata(
        storeName: string,
        key: string
      ): Promise<
        | [string, KvKeyMetadata | null]
        | null
      >
      op_kv_set(
        storeName: string,
        key: string,
        value: string,
        options: KvKeyMetadata
      ): Promise<void>
      op_kv_update_metadata(
        storeName: string,
        key: string,
        metadata: JsonValue | undefined
      ): Promise<void>
      op_kv_delete(storeName: string, key: string): Promise<void>
      op_kv_list_keys(
        options: ListKeysOptions,
        storeName: string
      ): Promise<ListKeysResult>
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
    const result = await Deno.core.ops.op_kv_get_with_metadata(
      this.#storeName,
      key
    )
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
    options?: KvKeyMetadata
  ): Promise<void> {
    await Deno.core.ops.op_kv_set(this.#storeName, key, value, {
      expiration: options?.expiration ?? undefined,
      metadata: options?.metadata ?? undefined
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
    metadata: JsonValue | undefined
  ): Promise<void> {
    await Deno.core.ops.op_kv_update_metadata(
      this.#storeName,
      key,
      metadata ?? undefined
    )
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
        prefix: options?.prefix ?? undefined,
        limit: options?.limit ?? undefined,
        cursor: options?.cursor ?? undefined
      },
      this.#storeName
    )
  }
}

export function store(name: string): KvStore {
  return new KvStore(name)
}

export const kv = {
  store
}
