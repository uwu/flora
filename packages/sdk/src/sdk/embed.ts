import type { RawEmbed as Embed, RawEmbedField as EmbedField } from './types'

export class EmbedBuilder {
  #embed: Embed

  constructor(initial: Embed = {}) {
    this.#embed = { ...initial }
  }

  setTitle(title: string) {
    this.#embed.title = title
    return this
  }

  setDescription(description: string) {
    this.#embed.description = description
    return this
  }

  setUrl(url: string) {
    this.#embed.url = url
    return this
  }

  setColor(color: number) {
    this.#embed.color = color
    return this
  }

  setTimestamp(timestamp: string) {
    this.#embed.timestamp = timestamp
    return this
  }

  setFooter(text: string, iconUrl?: string) {
    this.#embed.footer = { text, iconUrl }
    return this
  }

  setImage(url: string) {
    this.#embed.image = { url }
    return this
  }

  setThumbnail(url: string) {
    this.#embed.thumbnail = { url }
    return this
  }

  setAuthor(name?: string, options?: { url?: string; iconUrl?: string }) {
    this.#embed.author = { name, ...options }
    return this
  }

  addField(name: string, value: string, inline = false) {
    const field: EmbedField = { name, value, inline }
    this.#embed.fields = [...(this.#embed.fields ?? []), field]
    return this
  }

  addFields(fields: EmbedField[]) {
    this.#embed.fields = [...(this.#embed.fields ?? []), ...fields]
    return this
  }

  setFields(fields: EmbedField[]) {
    this.#embed.fields = [...fields]
    return this
  }

  toJSON(): Embed {
    return { ...this.#embed }
  }
}

export function embed(initial?: Embed) {
  return new EmbedBuilder(initial)
}
