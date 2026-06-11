import { defineComponent, h } from 'vue'
import { cn } from '../utils/cn'

function tablePart(name: string, tag: string, className: string) {
  return defineComponent({
    name,
    inheritAttrs: false,
    setup(_, { attrs, slots }) {
      return () =>
        h(
          tag,
          {
            ...attrs,
            class: cn(className, attrs.class)
          },
          slots
        )
    }
  })
}

export const TableHeader = tablePart(
  'FlTableHeader',
  'thead',
  'bg-[var(--fl-color-bg-subtle)] text-[var(--fl-color-text-muted)]'
)

export const TableBody = tablePart(
  'FlTableBody',
  'tbody',
  'divide-y divide-[var(--fl-color-border)]'
)

export const TableFooter = tablePart(
  'FlTableFooter',
  'tfoot',
  'border-t border-[var(--fl-color-border)] bg-[var(--fl-color-bg-subtle)]'
)

export const TableRow = tablePart(
  'FlTableRow',
  'tr',
  'transition-[background-color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] hover:bg-[var(--fl-color-bg-subtle)]'
)

export const TableHead = tablePart(
  'FlTableHead',
  'th',
  'h-9 whitespace-nowrap px-3 text-left text-xs font-700 uppercase tracking-[0.04em]'
)

export const TableCell = tablePart(
  'FlTableCell',
  'td',
  'h-10 whitespace-nowrap px-3 text-[var(--fl-color-text)]'
)

export const TableCaption = tablePart(
  'FlTableCaption',
  'caption',
  'mt-3 text-sm text-[var(--fl-color-text-subtle)]'
)
