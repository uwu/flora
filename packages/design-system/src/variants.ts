import { cva, type VariantProps } from 'class-variance-authority'

export const buttonVariants = cva(
  [
    'fl-focus-ring inline-flex shrink-0 items-center justify-center gap-1.5 whitespace-nowrap rounded-[var(--fl-radius-md)] border border-transparent bg-clip-padding text-sm font-600 leading-none outline-none select-none',
    'transition-[scale,background-color,color,border-color,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)]',
    'active:scale-[0.96] disabled:pointer-events-none disabled:scale-100 disabled:opacity-50',
    '[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=size-])]:size-4'
  ],
  {
    variants: {
      variant: {
        primary:
          'bg-[var(--fl-color-primary)] text-[var(--fl-color-primary-text)] hover:bg-[var(--fl-color-primary-hover)] active:bg-[var(--fl-color-primary-active)]',
        secondary:
          'bg-[var(--fl-color-bg-muted)] text-[var(--fl-color-text)] hover:bg-[var(--gray4)] active:bg-[var(--gray5)]',
        outline:
          'border-[var(--fl-color-border)] bg-[var(--fl-color-surface)] text-[var(--fl-color-text)] hover:border-[var(--fl-color-border-strong)] hover:bg-[var(--fl-color-bg-subtle)]',
        ghost:
          'bg-transparent text-[var(--fl-color-text-muted)] hover:bg-[var(--fl-color-bg-muted)] hover:text-[var(--fl-color-text)]',
        danger:
          'bg-[var(--fl-color-danger-soft)] text-[var(--fl-color-danger-soft-text)] hover:bg-[var(--redA4)]',
        link: 'h-auto rounded-[var(--fl-radius-sm)] bg-transparent px-0 text-[var(--fl-color-primary-soft-text)] underline-offset-4 hover:underline active:scale-100'
      },
      size: {
        xs: 'h-7 px-2 text-xs',
        sm: 'h-8 px-2.5 text-sm',
        md: 'h-9 px-3 text-sm',
        lg: 'h-10 px-4 text-sm',
        icon: 'size-9 p-0',
        'icon-sm': 'size-8 p-0',
        'icon-lg': 'size-10 p-0'
      },
      loading: {
        true: 'cursor-wait',
        false: ''
      },
      static: {
        true: 'active:scale-100',
        false: ''
      }
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
      loading: false,
      static: false
    }
  }
)

export type ButtonVariants = VariantProps<typeof buttonVariants>

export const badgeVariants = cva(
  'inline-flex min-h-5 items-center gap-1 rounded-[var(--fl-radius-pill)] px-2 py-0.5 text-xs font-650 leading-none',
  {
    variants: {
      tone: {
        neutral: 'bg-[var(--grayA3)] text-[var(--fl-color-text-muted)]',
        primary: 'bg-[var(--fl-color-primary-soft)] text-[var(--fl-color-primary-soft-text)]',
        success: 'bg-[var(--fl-color-success-soft)] text-[var(--fl-color-success-soft-text)]',
        warning: 'bg-[var(--fl-color-warning-soft)] text-[var(--fl-color-warning-soft-text)]',
        danger: 'bg-[var(--fl-color-danger-soft)] text-[var(--fl-color-danger-soft-text)]',
        info: 'bg-[var(--fl-color-info-soft)] text-[var(--fl-color-info-soft-text)]'
      },
      variant: {
        soft: '',
        outline: 'bg-transparent shadow-[inset_0_0_0_1px_var(--fl-color-border)]'
      }
    },
    defaultVariants: {
      tone: 'neutral',
      variant: 'soft'
    }
  }
)

export type BadgeVariants = VariantProps<typeof badgeVariants>

export const inputVariants = cva(
  [
    'fl-focus-ring w-full min-w-0 rounded-[var(--fl-radius-md)] border border-[var(--fl-color-border)] bg-[var(--fl-color-surface)] text-[var(--fl-color-text)] outline-none',
    'placeholder:text-[var(--fl-color-placeholder)] disabled:cursor-not-allowed disabled:opacity-55',
    'transition-[background-color,border-color,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)]',
    'aria-invalid:border-[var(--fl-color-danger)] aria-invalid:shadow-[0_0_0_3px_var(--redA4)]'
  ],
  {
    variants: {
      size: {
        sm: 'h-8 px-2.5 text-sm',
        md: 'h-9 px-3 text-sm',
        lg: 'h-10 px-3.5 text-base'
      }
    },
    defaultVariants: {
      size: 'md'
    }
  }
)

export type InputVariants = VariantProps<typeof inputVariants>

export const surfaceVariants = cva(
  'rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] text-[var(--fl-color-text)]',
  {
    variants: {
      variant: {
        plain: '',
        bordered: 'shadow-[var(--fl-shadow-border)]',
        raised: 'shadow-[var(--fl-shadow-overlay)]'
      },
      padding: {
        none: '',
        sm: 'p-3',
        md: 'p-4',
        lg: 'p-6'
      }
    },
    defaultVariants: {
      variant: 'bordered',
      padding: 'md'
    }
  }
)

export type SurfaceVariants = VariantProps<typeof surfaceVariants>
