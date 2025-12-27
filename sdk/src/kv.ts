/**
 * KV Store API for Flora
 * 
 * Provides a simple key-value store interface per guild with named stores.
 */

/**
 * A KV store instance for a specific store name.
 * 
 * All operations are scoped to the current guild context.
 */
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
    // @ts-ignore - Deno op
    return await Deno.core.ops.op_kv_get(this.#storeName, key)
  }

  /**
   * Set a value in the store.
   * 
   * The value size is limited to 1MB.
   * 
   * @param key - The key to set
   * @param value - The value to store (max 1MB)
   */
  async set(key: string, value: string): Promise<void> {
    // @ts-ignore - Deno op
    await Deno.core.ops.op_kv_set(this.#storeName, key, value)
  }

  /**
   * Delete a key from the store.
   * 
   * @param key - The key to delete
   */
  async delete(key: string): Promise<void> {
    // @ts-ignore - Deno op
    await Deno.core.ops.op_kv_delete(this.#storeName, key)
  }

  /**
   * List all keys in the store, optionally filtered by prefix.
   * 
   * WARNING: This method is not paginated. It may be slow for stores with millions of keys.
   * Consider using a prefix filter to narrow results.
   * 
   * @param prefix - Optional prefix to filter keys
   * @returns Array of keys
   */
  async listKeys(prefix?: string): Promise<string[]> {
    // @ts-ignore - Deno op
    return await Deno.core.ops.op_kv_list_keys(this.#storeName, prefix ?? null)
  }
}

/**
 * Get a named KV store instance.
 * 
 * The store must be created via the backend API before use.
 * 
 * @param name - The name of the store
 * @returns A KvStore instance
 * 
 * @example
 * ```ts
 * const users = kv.store('users')
 * await users.set('alice', JSON.stringify({ name: 'Alice', score: 100 }))
 * const data = await users.get('alice')
 * ```
 */
export function store(name: string): KvStore {
  return new KvStore(name)
}

export const kv = {
  store,
}
