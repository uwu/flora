let counter = 100000000000000000n

export function nextId(): string {
  counter += 1n
  return counter.toString()
}

export function resetIdCounter(): void {
  counter = 100000000000000000n
}
