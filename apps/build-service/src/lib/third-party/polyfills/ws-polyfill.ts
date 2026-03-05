import { EventEmitter } from 'node:events'

type FloraOn = (
  event: string,
  handler: (ctx: { msg: unknown }) => void | Promise<void>
) => void

type ReadyPayload = {
  user?: {
    id?: string
    username?: string
    discriminator?: number | null
    bot?: boolean
  }
  guildIds?: string[]
}

type MessagePayload = {
  id?: string
  channelId?: string
  guildId?: string
  content?: string
  author?: {
    id?: string
    username?: string
    discriminator?: number | null
    bot?: boolean
  }
  member?: {
    nick?: string | null
    roles?: string[]
    joinedAt?: string | null
    premiumSince?: string | null
    pending?: boolean
    communicationDisabledUntil?: string | null
  }
}

const TOKEN_MARKER = '__FLORA_THIRDPARTY_DISCORD_TOKEN__'

type EventHandler<E> = ((event: E) => void) | null

class WS extends EventEmitter {
  private _readyState = 0
  private _closed = false
  private _identified = false
  private _seq = 0
  private _readyPayload: ReadyPayload | null = null

  binaryType: string = 'blob'

  private _onopen: EventHandler<{ target: WS }> = null
  private _onclose: EventHandler<{ code: number; reason: string; target: WS }> = null
  private _onerror: EventHandler<{ error: Error; message: string; target: WS }> = null
  private _onmessage: EventHandler<{
    data: unknown
    type: string
    target: WS
  }> = null

  get onopen() {
    return this._onopen
  }
  set onopen(fn: EventHandler<{ target: WS }>) {
    if (this._onopen) this.removeListener('open', this._onopen as any)
    this._onopen = fn
    if (fn) this.on('open', fn as any)
  }

  get onclose() {
    return this._onclose
  }
  set onclose(
    fn: EventHandler<{ code: number; reason: string; target: WS }>
  ) {
    if (this._onclose) this.removeListener('close', this._onclose as any)
    this._onclose = fn
    if (fn) this.on('close', fn as any)
  }

  get onerror() {
    return this._onerror
  }
  set onerror(
    fn: EventHandler<{ error: Error; message: string; target: WS }>
  ) {
    if (this._onerror) this.removeListener('error', this._onerror as any)
    this._onerror = fn
    if (fn) this.on('error', fn as any)
  }

  get onmessage() {
    return this._onmessage
  }
  set onmessage(
    fn: EventHandler<{ data: unknown; type: string; target: WS }>
  ) {
    if (this._onmessage) this.removeListener('message', this._onmessage as any)
    this._onmessage = fn
    if (fn) this.on('message', fn as any)
  }

  constructor(_url: string | URL, _protocols?: string | string[], _opts?: unknown) {
    super()

    const floraOn = (globalThis as { on?: FloraOn }).on
    if (!floraOn) {
      queueMicrotask(() => {
        this._emitError(new Error('flora event bridge unavailable'))
      })
      return
    }

    floraOn('ready', ({ msg }) => {
      this._readyPayload = (msg as ReadyPayload) ?? null
      if (this._identified) {
        this._emitDispatch('READY', this._buildReadyDispatch())
      }
    })

    floraOn('messageCreate', ({ msg }) => {
      if (!this._identified) return
      this._emitDispatch(
        'MESSAGE_CREATE',
        this._buildMessageDispatch(msg as MessagePayload)
      )
    })

    queueMicrotask(() => {
      this._readyState = WS.OPEN
      this.emit('open', { target: this })
      this._emitRaw({ op: 10, d: { heartbeat_interval: 45000 } })
    })
  }

  send(data: unknown) {
    if (this._readyState !== WS.OPEN) {
      throw new Error('WebSocket is not open')
    }

    let frame: { op?: number; d?: unknown }
    try {
      frame = JSON.parse(String(data)) as { op?: number; d?: unknown }
    } catch {
      return
    }

    if (frame.op === 1) {
      this._emitRaw({ op: 11, d: null })
      return
    }

    if (frame.op === 2) {
      const token = (frame.d as { token?: string } | undefined)?.token
      if (token !== TOKEN_MARKER) {
        this._emitError(
          new Error(
            'invalid token marker for third-party discord transport'
          )
        )
        this.close(4004, 'authentication failed')
        return
      }

      this._identified = true
      this._emitDispatch('READY', this._buildReadyDispatch())
    }
  }

  close(code?: number, reason?: string) {
    if (this._closed) return

    this._closed = true
    this._readyState = WS.CLOSED
    this.emit('close', {
      code: code ?? 1000,
      reason: reason ?? '',
      target: this
    })
  }

  terminate() {
    this.close()
  }

  cleanup() {
    this.close()
  }

  get readyState() {
    return this._readyState
  }

  private _emitError(err: Error) {
    this.emit('error', { error: err, message: err.message, target: this })
  }

  private _emitDispatch(eventType: string, data: unknown) {
    this._seq += 1
    this._emitRaw({ op: 0, t: eventType, s: this._seq, d: data })
  }

  private _emitRaw(payload: unknown) {
    const str = JSON.stringify(payload)
    this.emit('message', { data: str, type: 'message', target: this })
  }

  private _buildReadyDispatch() {
    const user = this._readyPayload?.user
    const userId = user?.id ?? '0'
    const guildIds = this._readyPayload?.guildIds ?? []

    return {
      v: 10,
      user: {
        id: userId,
        username: user?.username ?? 'flora',
        discriminator: String(user?.discriminator ?? 0),
        bot: user?.bot ?? true,
        avatar: null,
        global_name: user?.username ?? 'flora'
      },
      guilds: guildIds.map((id) => ({ id, unavailable: true })),
      session_id: 'flora-thirdparty-session',
      resume_gateway_url: 'wss://gateway.discord.gg',
      application: {
        id: userId,
        flags: 0
      }
    }
  }

  private _buildMessageDispatch(msg: MessagePayload) {
    const author = msg.author ?? {}
    const member = msg.member

    return {
      id: msg.id ?? '0',
      channel_id: msg.channelId ?? '0',
      guild_id: msg.guildId ?? null,
      type: 0,
      content: msg.content ?? '',
      timestamp: new Date().toISOString(),
      edited_timestamp: null,
      tts: false,
      mention_everyone: false,
      mentions: [],
      mention_roles: [],
      attachments: [],
      embeds: [],
      pinned: false,
      flags: 0,
      author: {
        id: author.id ?? '0',
        username: author.username ?? 'unknown',
        discriminator: String(author.discriminator ?? 0),
        bot: author.bot ?? false,
        avatar: null,
        global_name: author.username ?? 'unknown'
      },
      member: member
        ? {
          nick: member.nick ?? null,
          roles: member.roles ?? [],
          joined_at: member.joinedAt ?? null,
          premium_since: member.premiumSince ?? null,
          pending: member.pending ?? false,
          communication_disabled_until: member.communicationDisabledUntil ?? null
        }
        : undefined
    }
  }

  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3
}

export { WS as WebSocket }
export default WS
