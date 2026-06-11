import {
  AccordionContent as RekaAccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger as RekaAccordionTrigger,
  CollapsibleContent as RekaCollapsibleContent,
  CollapsibleRoot,
  CollapsibleTrigger,
  DialogClose,
  DialogContent as RekaDialogContent,
  DialogDescription as RekaDialogDescription,
  DialogOverlay as RekaDialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle as RekaDialogTitle,
  DialogTrigger,
  DropdownMenuCheckboxItem as RekaDropdownMenuCheckboxItem,
  DropdownMenuContent as RekaDropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem as RekaDropdownMenuItem,
  DropdownMenuItemIndicator,
  DropdownMenuLabel as RekaDropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem as RekaDropdownMenuRadioItem,
  DropdownMenuRoot,
  DropdownMenuSeparator as RekaDropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent as RekaDropdownMenuSubContent,
  DropdownMenuSubTrigger as RekaDropdownMenuSubTrigger,
  DropdownMenuTrigger,
  PopoverClose,
  PopoverContent as RekaPopoverContent,
  PopoverPortal,
  PopoverRoot,
  PopoverTrigger,
  TabsContent as RekaTabsContent,
  TabsList as RekaTabsList,
  TabsRoot,
  TabsTrigger as RekaTabsTrigger,
  TooltipContent as RekaTooltipContent,
  TooltipPortal,
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger
} from 'reka-ui'
import { defineComponent, h, type Component } from 'vue'
import { cn } from '../utils/cn'

function styled(name: string, component: Component, className: string) {
  return defineComponent({
    name,
    inheritAttrs: false,
    setup(_, { attrs, slots }) {
      return () =>
        h(
          component,
          {
            ...attrs,
            class: cn(className, attrs.class)
          },
          slots
        )
    }
  })
}

export { AccordionHeader, AccordionItem, AccordionRoot }
export { CollapsibleRoot, CollapsibleTrigger }
export { DialogClose, DialogPortal, DialogRoot, DialogTrigger }
export { DropdownMenuGroup, DropdownMenuItemIndicator, DropdownMenuPortal, DropdownMenuRadioGroup }
export { DropdownMenuRoot, DropdownMenuSub, DropdownMenuTrigger }
export { PopoverClose, PopoverPortal, PopoverRoot, PopoverTrigger }
export { TabsRoot }
export { TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger }

export const TooltipContent = styled(
  'FlTooltipContent',
  RekaTooltipContent,
  'fl-popover-enter z-[var(--fl-z-dropdown)] max-w-72 rounded-[var(--fl-radius-md)] bg-[var(--fl-color-text)] px-2.5 py-1.5 text-xs font-600 leading-5 text-[var(--fl-color-bg)] shadow-[var(--fl-shadow-overlay)]'
)

export const PopoverContent = styled(
  'FlPopoverContent',
  RekaPopoverContent,
  'fl-popover-enter z-[var(--fl-z-dropdown)] min-w-56 rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface-overlay)] p-3 text-sm text-[var(--fl-color-text)] shadow-[var(--fl-shadow-overlay)] outline-none'
)

export const DialogOverlay = styled(
  'FlDialogOverlay',
  RekaDialogOverlay,
  'fl-overlay-enter fixed inset-0 z-[var(--fl-z-overlay)] bg-black/45'
)

export const DialogContent = styled(
  'FlDialogContent',
  RekaDialogContent,
  'fl-dialog-enter fixed left-1/2 top-1/2 z-[calc(var(--fl-z-overlay)+1)] grid max-h-[min(720px,calc(100vh-32px))] w-[min(520px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 gap-4 overflow-auto rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface-overlay)] p-5 text-[var(--fl-color-text)] shadow-[var(--fl-shadow-overlay)] outline-none'
)

export const DialogTitle = styled(
  'FlDialogTitle',
  RekaDialogTitle,
  'm-0 text-base font-750 leading-6 text-[var(--fl-color-text)]'
)

export const DialogDescription = styled(
  'FlDialogDescription',
  RekaDialogDescription,
  'm-0 text-sm leading-6 text-[var(--fl-color-text-muted)]'
)

export const SheetOverlay = DialogOverlay

export const SheetContent = styled(
  'FlSheetContent',
  RekaDialogContent,
  'fl-popover-enter fixed inset-y-0 right-0 z-[calc(var(--fl-z-overlay)+1)] grid h-dvh w-[min(420px,calc(100vw-24px))] gap-4 overflow-auto bg-[var(--fl-color-surface-overlay)] p-5 text-[var(--fl-color-text)] shadow-[var(--fl-shadow-overlay)] outline-none'
)

export const DropdownMenuContent = styled(
  'FlDropdownMenuContent',
  RekaDropdownMenuContent,
  'fl-popover-enter z-[var(--fl-z-dropdown)] min-w-48 overflow-hidden rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface-overlay)] p-1 text-sm text-[var(--fl-color-text)] shadow-[var(--fl-shadow-overlay)] outline-none'
)

export const DropdownMenuSubContent = styled(
  'FlDropdownMenuSubContent',
  RekaDropdownMenuSubContent,
  'fl-popover-enter z-[var(--fl-z-dropdown)] min-w-44 overflow-hidden rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface-overlay)] p-1 text-sm text-[var(--fl-color-text)] shadow-[var(--fl-shadow-overlay)] outline-none'
)

export const DropdownMenuItem = styled(
  'FlDropdownMenuItem',
  RekaDropdownMenuItem,
  'relative flex min-h-8 cursor-default select-none items-center gap-2 rounded-[var(--fl-radius-md)] px-2.5 py-1.5 text-sm outline-none transition-[background-color,color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] data-[highlighted]:bg-[var(--fl-color-bg-muted)] data-[highlighted]:text-[var(--fl-color-text)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50'
)

export const DropdownMenuCheckboxItem = styled(
  'FlDropdownMenuCheckboxItem',
  RekaDropdownMenuCheckboxItem,
  'relative flex min-h-8 cursor-default select-none items-center gap-2 rounded-[var(--fl-radius-md)] py-1.5 pl-8 pr-2.5 text-sm outline-none transition-[background-color,color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] data-[highlighted]:bg-[var(--fl-color-bg-muted)] data-[highlighted]:text-[var(--fl-color-text)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50'
)

export const DropdownMenuRadioItem = styled(
  'FlDropdownMenuRadioItem',
  RekaDropdownMenuRadioItem,
  'relative flex min-h-8 cursor-default select-none items-center gap-2 rounded-[var(--fl-radius-md)] py-1.5 pl-8 pr-2.5 text-sm outline-none transition-[background-color,color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] data-[highlighted]:bg-[var(--fl-color-bg-muted)] data-[highlighted]:text-[var(--fl-color-text)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50'
)

export const DropdownMenuSubTrigger = styled(
  'FlDropdownMenuSubTrigger',
  RekaDropdownMenuSubTrigger,
  'relative flex min-h-8 cursor-default select-none items-center gap-2 rounded-[var(--fl-radius-md)] px-2.5 py-1.5 text-sm outline-none transition-[background-color,color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] data-[highlighted]:bg-[var(--fl-color-bg-muted)] data-[highlighted]:text-[var(--fl-color-text)] data-[state=open]:bg-[var(--fl-color-bg-muted)]'
)

export const DropdownMenuLabel = styled(
  'FlDropdownMenuLabel',
  RekaDropdownMenuLabel,
  'px-2.5 py-1.5 text-xs font-700 uppercase tracking-[0.04em] text-[var(--fl-color-text-subtle)]'
)

export const DropdownMenuSeparator = styled(
  'FlDropdownMenuSeparator',
  RekaDropdownMenuSeparator,
  '-mx-1 my-1 h-px bg-[var(--fl-color-border)]'
)

export const TabsList = styled(
  'FlTabsList',
  RekaTabsList,
  'inline-flex min-h-9 items-center gap-1 rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-bg-subtle)] p-1'
)

export const TabsTrigger = styled(
  'FlTabsTrigger',
  RekaTabsTrigger,
  'fl-focus-ring inline-flex h-7 items-center justify-center rounded-[var(--fl-radius-md)] px-3 text-sm font-650 text-[var(--fl-color-text-muted)] outline-none transition-[background-color,color,box-shadow] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] data-[state=active]:bg-[var(--fl-color-surface)] data-[state=active]:text-[var(--fl-color-text)] data-[state=active]:shadow-[var(--fl-shadow-border)] disabled:pointer-events-none disabled:opacity-50'
)

export const TabsContent = styled('FlTabsContent', RekaTabsContent, 'mt-3 outline-none')

export const AccordionTrigger = styled(
  'FlAccordionTrigger',
  RekaAccordionTrigger,
  'fl-focus-ring flex min-h-10 w-full items-center justify-between gap-3 rounded-[var(--fl-radius-md)] px-2 text-left text-sm font-650 text-[var(--fl-color-text)] outline-none transition-[background-color,color,box-shadow] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] hover:bg-[var(--fl-color-bg-subtle)]'
)

export const AccordionContent = styled(
  'FlAccordionContent',
  RekaAccordionContent,
  'overflow-hidden px-2 pb-3 text-sm leading-6 text-[var(--fl-color-text-muted)] data-[state=closed]:animate-[fl-fade-out_var(--fl-duration-fast)_ease-in] data-[state=open]:animate-[fl-fade-in_var(--fl-duration-base)_var(--fl-ease-standard)]'
)

export const CollapsibleContent = styled(
  'FlCollapsibleContent',
  RekaCollapsibleContent,
  'overflow-hidden data-[state=closed]:animate-[fl-fade-out_var(--fl-duration-fast)_ease-in] data-[state=open]:animate-[fl-fade-in_var(--fl-duration-base)_var(--fl-ease-standard)]'
)
