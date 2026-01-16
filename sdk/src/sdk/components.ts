import { ButtonStyle, ComponentType, InputTextStyle } from './types'
import type { JsonValue } from './types'

export type ComponentJson = {
  type: number
  [key: string]: JsonValue
}

export type ComponentLike = ComponentBuilder | ComponentJson

export type ComponentBuilder = {
  toJSON: () => ComponentJson
}

type EmojiLike = JsonValue
type SelectDefaultKind = 'user' | 'role' | 'channel'

type SelectDefaultValue = {
  id: string
  type: SelectDefaultKind
}

type SelectOption = {
  label: string
  value: string
  description?: string
  emoji?: EmojiLike
  default?: boolean
}

type MediaItem = {
  url: string
}

type MediaItemEntry = {
  media: MediaItem
  description?: string
  spoiler?: boolean
}

const isBuilder = (value: ComponentLike): value is ComponentBuilder =>
  typeof (value as ComponentBuilder)?.toJSON === 'function'

const resolveComponent = (value: ComponentLike): ComponentJson =>
  (isBuilder(value) ? value.toJSON() : value) as ComponentJson

export class ActionRowBuilder implements ComponentBuilder {
  #components: ComponentLike[] = []

  addComponents(...components: ComponentLike[]) {
    this.#components.push(...components)
    return this
  }

  setComponents(components: ComponentLike[]) {
    this.#components = [...components]
    return this
  }

  toJSON(): ComponentJson {
    return {
      type: ComponentType.ActionRow,
      components: this.#components.map(resolveComponent)
    }
  }
}

export class ButtonBuilder implements ComponentBuilder {
  #data: ComponentJson = { type: ComponentType.Button }

  setStyle(style: number) {
    this.#data.style = style
    return this
  }

  setCustomId(customId: string) {
    this.#data.custom_id = customId
    return this
  }

  setUrl(url: string) {
    this.#data.style = 5
    this.#data.url = url
    return this
  }

  setSkuId(skuId: string) {
    this.#data.style = 6
    this.#data.sku_id = skuId
    return this
  }

  setLabel(label: string) {
    this.#data.label = label
    return this
  }

  setEmoji(emoji: EmojiLike) {
    this.#data.emoji = emoji
    return this
  }

  setDisabled(disabled = true) {
    this.#data.disabled = disabled
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export class SelectMenuBuilderBase implements ComponentBuilder {
  protected data: ComponentJson

  constructor(type: number, customId: string) {
    this.data = { type, custom_id: customId }
  }

  setCustomId(customId: string) {
    this.data.custom_id = customId
    return this
  }

  setPlaceholder(placeholder: string) {
    this.data.placeholder = placeholder
    return this
  }

  setMinValues(min: number) {
    this.data.min_values = min
    return this
  }

  setMaxValues(max: number) {
    this.data.max_values = max
    return this
  }

  setRequired(required = true) {
    this.data.required = required
    return this
  }

  setDisabled(disabled = true) {
    this.data.disabled = disabled
    return this
  }

  setChannelTypes(types: number[]) {
    this.data.channel_types = [...types]
    return this
  }

  setDefaultValues(values: SelectDefaultValue[]) {
    this.data.default_values = values.map(value => ({ ...value }))
    return this
  }

  setDefaultUsers(ids: string[]) {
    this.data.default_users = [...ids]
    return this
  }

  setDefaultRoles(ids: string[]) {
    this.data.default_roles = [...ids]
    return this
  }

  setDefaultChannels(ids: string[]) {
    this.data.default_channels = [...ids]
    return this
  }

  addDefaultUser(id: string) {
    const current = (this.data.default_users ?? []) as string[]
    this.data.default_users = [...current, id]
    return this
  }

  addDefaultRole(id: string) {
    const current = (this.data.default_roles ?? []) as string[]
    this.data.default_roles = [...current, id]
    return this
  }

  addDefaultChannel(id: string) {
    const current = (this.data.default_channels ?? []) as string[]
    this.data.default_channels = [...current, id]
    return this
  }

  addDefaultValue(id: string, type: SelectDefaultKind) {
    const current = (this.data.default_values ?? []) as SelectDefaultValue[]
    this.data.default_values = [...current, { id, type }]
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.data }
  }
}

export class StringSelectMenuBuilder extends SelectMenuBuilderBase {
  constructor(customId: string) {
    super(ComponentType.StringSelect, customId)
  }

  setOptions(options: SelectOption[]) {
    this.data.options = options.map(option => ({ ...option }))
    return this
  }

  addOptions(...options: SelectOption[]) {
    const current = (this.data.options ?? []) as SelectOption[]
    this.data.options = [...current, ...options.map(option => ({ ...option }))]
    return this
  }
}

export class UserSelectMenuBuilder extends SelectMenuBuilderBase {
  constructor(customId: string) {
    super(ComponentType.UserSelect, customId)
  }
}

export class RoleSelectMenuBuilder extends SelectMenuBuilderBase {
  constructor(customId: string) {
    super(ComponentType.RoleSelect, customId)
  }
}

export class MentionableSelectMenuBuilder extends SelectMenuBuilderBase {
  constructor(customId: string) {
    super(ComponentType.MentionableSelect, customId)
  }
}

export class ChannelSelectMenuBuilder extends SelectMenuBuilderBase {
  constructor(customId: string) {
    super(ComponentType.ChannelSelect, customId)
  }
}

export class InputTextBuilder implements ComponentBuilder {
  #data: ComponentJson

  constructor(customId: string) {
    this.#data = { type: ComponentType.InputText, custom_id: customId }
  }

  setCustomId(customId: string) {
    this.#data.custom_id = customId
    return this
  }

  setStyle(style: number) {
    this.#data.style = style
    return this
  }

  setMinLength(min: number) {
    this.#data.min_length = min
    return this
  }

  setMaxLength(max: number) {
    this.#data.max_length = max
    return this
  }

  setRequired(required = true) {
    this.#data.required = required
    return this
  }

  setValue(value: string) {
    this.#data.value = value
    return this
  }

  setPlaceholder(placeholder: string) {
    this.#data.placeholder = placeholder
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export class TextDisplayBuilder implements ComponentBuilder {
  #data: ComponentJson

  constructor(content: string) {
    this.#data = { type: ComponentType.TextDisplay, content }
  }

  setContent(content: string) {
    this.#data.content = content
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export class ThumbnailBuilder implements ComponentBuilder {
  #data: ComponentJson

  constructor(url: string) {
    this.#data = { type: ComponentType.Thumbnail, media: { url } }
  }

  setUrl(url: string) {
    this.#data.media = { url }
    return this
  }

  setDescription(description: string) {
    this.#data.description = description
    return this
  }

  setSpoiler(spoiler = true) {
    this.#data.spoiler = spoiler
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export class SectionBuilder implements ComponentBuilder {
  #components: ComponentLike[] = []
  #accessory?: ComponentLike

  addComponents(...components: ComponentLike[]) {
    this.#components.push(...components)
    return this
  }

  setComponents(components: ComponentLike[]) {
    this.#components = [...components]
    return this
  }

  setAccessory(accessory: ComponentLike) {
    this.#accessory = accessory
    return this
  }

  toJSON(): ComponentJson {
    return {
      type: ComponentType.Section,
      components: this.#components.map(resolveComponent),
      accessory: this.#accessory ? resolveComponent(this.#accessory) : null
    }
  }
}

export class MediaGalleryBuilder implements ComponentBuilder {
  #items: MediaItemEntry[] = []

  addItem(url: string, options?: { description?: string; spoiler?: boolean }) {
    const item: MediaItemEntry = {
      media: { url },
      description: options?.description,
      spoiler: options?.spoiler
    }
    this.#items.push(item)
    return this
  }

  setItems(items: MediaItemEntry[]) {
    this.#items = items.map(item => ({ ...item, media: { ...item.media } }))
    return this
  }

  toJSON(): ComponentJson {
    return {
      type: ComponentType.MediaGallery,
      items: this.#items.map(item => ({ ...item, media: { ...item.media } }))
    }
  }
}

export class FileBuilder implements ComponentBuilder {
  #data: ComponentJson

  constructor(url: string) {
    this.#data = { type: ComponentType.File, file: { url } }
  }

  setUrl(url: string) {
    this.#data.file = { url }
    return this
  }

  setSpoiler(spoiler = true) {
    this.#data.spoiler = spoiler
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export class SeparatorBuilder implements ComponentBuilder {
  #data: ComponentJson

  constructor(divider = true) {
    this.#data = { type: ComponentType.Separator, divider }
  }

  setDivider(divider = true) {
    this.#data.divider = divider
    return this
  }

  setSpacing(spacing: number | 'small' | 'large') {
    this.#data.spacing = spacing
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export class ContainerBuilder implements ComponentBuilder {
  #components: ComponentLike[] = []
  #accentColor?: number
  #spoiler?: boolean

  addComponents(...components: ComponentLike[]) {
    this.#components.push(...components)
    return this
  }

  setComponents(components: ComponentLike[]) {
    this.#components = [...components]
    return this
  }

  setAccentColor(color: number) {
    this.#accentColor = color
    return this
  }

  setSpoiler(spoiler = true) {
    this.#spoiler = spoiler
    return this
  }

  toJSON(): ComponentJson {
    const data: ComponentJson = {
      type: ComponentType.Container,
      components: this.#components.map(resolveComponent)
    }
    if (this.#accentColor !== undefined) data.accent_color = this.#accentColor
    if (this.#spoiler !== undefined) data.spoiler = this.#spoiler
    return data
  }
}

export class LabelBuilder implements ComponentBuilder {
  #label: string
  #description?: string
  #component?: ComponentLike

  constructor(label: string) {
    this.#label = label
  }

  setLabel(label: string) {
    this.#label = label
    return this
  }

  setDescription(description: string) {
    this.#description = description
    return this
  }

  setComponent(component: ComponentLike) {
    this.#component = component
    return this
  }

  toJSON(): ComponentJson {
    const data: ComponentJson = {
      type: ComponentType.Label,
      label: this.#label,
      component: this.#component ? resolveComponent(this.#component) : null
    }
    if (this.#description !== undefined) data.description = this.#description
    return data
  }
}

export class FileUploadBuilder implements ComponentBuilder {
  #data: ComponentJson

  constructor(customId: string) {
    this.#data = { type: ComponentType.FileUpload, custom_id: customId }
  }

  setCustomId(customId: string) {
    this.#data.custom_id = customId
    return this
  }

  setMinValues(min: number) {
    this.#data.min_values = min
    return this
  }

  setMaxValues(max: number) {
    this.#data.max_values = max
    return this
  }

  setRequired(required = true) {
    this.#data.required = required
    return this
  }

  toJSON(): ComponentJson {
    return { ...this.#data }
  }
}

export const actionRow = () => new ActionRowBuilder()
export const button = () => new ButtonBuilder()
export const stringSelect = (customId: string) => new StringSelectMenuBuilder(customId)
export const userSelect = (customId: string) => new UserSelectMenuBuilder(customId)
export const roleSelect = (customId: string) => new RoleSelectMenuBuilder(customId)
export const mentionableSelect = (customId: string) => new MentionableSelectMenuBuilder(customId)
export const channelSelect = (customId: string) => new ChannelSelectMenuBuilder(customId)
export const inputText = (customId: string) => new InputTextBuilder(customId)
export const textDisplay = (content: string) => new TextDisplayBuilder(content)
export const thumbnail = (url: string) => new ThumbnailBuilder(url)
export const section = () => new SectionBuilder()
export const mediaGallery = () => new MediaGalleryBuilder()
export const file = (url: string) => new FileBuilder(url)
export const separator = (divider = true) => new SeparatorBuilder(divider)
export const container = () => new ContainerBuilder()
export const label = (labelText: string) => new LabelBuilder(labelText)
export const fileUpload = (customId: string) => new FileUploadBuilder(customId)

export const ButtonStyles = ButtonStyle
export const InputTextStyles = InputTextStyle
