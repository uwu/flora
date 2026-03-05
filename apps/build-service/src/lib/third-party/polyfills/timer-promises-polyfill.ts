export function setTimeout<T = void>(_delay?: number, value?: T): Promise<T> {
  return Promise.resolve(value as T)
}

export function setImmediate<T = void>(value?: T): Promise<T> {
  return Promise.resolve(value as T)
}

export function setInterval<T = void>(_delay?: number, value?: T) {
  return {
    promise: Promise.resolve(value as T),
    clear() {}
  }
}

export default { setTimeout, setImmediate, setInterval }
