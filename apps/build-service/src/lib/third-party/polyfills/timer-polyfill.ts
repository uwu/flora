const nativeSetTimeout = globalThis.setTimeout
const nativeClearTimeout = globalThis.clearTimeout
const nativeClearInterval = globalThis.clearInterval

type TimerRef = {
  unref(): TimerRef
  ref(): TimerRef
  [Symbol.toPrimitive](): number
  valueOf(): number
  clear(): void
}

type TimerCallback = (...args: unknown[]) => void

export function setTimeout(
  callback: TimerCallback,
  delay?: number,
  ...args: unknown[]
): TimerRef {
  const id = nativeSetTimeout(callback, delay, ...args)

  return {
    unref() {
      return this
    },
    ref() {
      return this
    },
    [Symbol.toPrimitive]() {
      return Number(id)
    },
    valueOf() {
      return Number(id)
    },
    clear() {
      nativeClearTimeout(id)
    }
  }
}

export function setInterval(
  _callback: TimerCallback,
  _delay?: number,
  ..._args: unknown[]
): TimerRef {
  return {
    unref() {
      return this
    },
    ref() {
      return this
    },
    [Symbol.toPrimitive]() {
      return 0
    },
    valueOf() {
      return 0
    },
    clear() {}
  }
}

export function clearTimeout(timeoutObj: TimerRef | number | undefined) {
  if (timeoutObj && typeof timeoutObj === 'object' && 'clear' in timeoutObj) {
    timeoutObj.clear()
    return
  }

  nativeClearTimeout(timeoutObj as number | undefined)
}

export function clearInterval(intervalObj: TimerRef | number | undefined) {
  if (
    intervalObj &&
    typeof intervalObj === 'object' &&
    'clear' in intervalObj
  ) {
    intervalObj.clear()
    return
  }

  nativeClearInterval(intervalObj as number | undefined)
}
