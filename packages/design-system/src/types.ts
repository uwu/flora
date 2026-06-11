import type { AcceptableValue } from 'reka-ui'

export type RadioOption = {
  label: string
  value: AcceptableValue
  description?: string
  disabled?: boolean
}

export type SelectOption = {
  label: string
  value: AcceptableValue
  disabled?: boolean
}
