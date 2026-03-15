import { Avatar as AvatarPrimitive } from '@base-ui/react/avatar'
import * as React from 'react'

import { cn } from '@/lib/utils'

type AvatarImageStatus = 'idle' | 'loading' | 'loaded' | 'error'

type AvatarContextValue = {
  status: AvatarImageStatus
  setStatus: (status: AvatarImageStatus) => void
}

const AvatarContext = React.createContext<AvatarContextValue | null>(null)
const loadedAvatarImages = new Set<string>()

function useAvatarContext() {
  const context = React.useContext(AvatarContext)
  if (!context) {
    throw new Error('Avatar components must be wrapped in <Avatar>.')
  }
  return context
}

function Avatar({
  className,
  size = 'default',
  ...props
}: AvatarPrimitive.Root.Props & {
  size?: 'default' | 'sm' | 'lg'
}) {
  const [status, setStatus] = React.useState<AvatarImageStatus>('idle')
  const value = React.useMemo(() => ({ status, setStatus }), [status])

  return (
    <AvatarContext.Provider value={value}>
      <AvatarPrimitive.Root
        data-slot='avatar'
        data-size={size}
        className={cn(
          'group/avatar relative flex size-8 shrink-0 rounded-full select-none after:absolute after:inset-0 after:rounded-full after:border after:border-border after:mix-blend-darken data-[size=lg]:size-10 data-[size=sm]:size-6 dark:after:mix-blend-lighten',
          className
        )}
        {...props}
      />
    </AvatarContext.Provider>
  )
}

type AvatarImageProps = React.ComponentProps<'img'>

function AvatarImage({
  className,
  src,
  onLoad,
  onError,
  loading,
  decoding,
  ...props
}: AvatarImageProps) {
  const { setStatus } = useAvatarContext()
  const cached = Boolean(src && loadedAvatarImages.has(src))
  const [isLoaded, setIsLoaded] = React.useState(cached)
  const [hasError, setHasError] = React.useState(false)

  React.useLayoutEffect(() => {
    if (!src) {
      setIsLoaded(false)
      setHasError(false)
      setStatus('idle')
      return
    }

    const isCached = loadedAvatarImages.has(src)
    setIsLoaded(isCached)
    setHasError(false)
    setStatus(isCached ? 'loaded' : 'loading')
  }, [setStatus, src])

  const handleLoad = (event: React.SyntheticEvent<HTMLImageElement>) => {
    if (src) {
      loadedAvatarImages.add(src)
    }
    setIsLoaded(true)
    setHasError(false)
    setStatus('loaded')
    onLoad?.(event)
  }

  const handleError = (event: React.SyntheticEvent<HTMLImageElement>) => {
    setIsLoaded(false)
    setHasError(true)
    setStatus('error')
    onError?.(event)
  }

  if (!src || hasError) {
    return null
  }

  return (
    <img
      data-slot='avatar-image'
      data-loaded={isLoaded}
      className={cn(
        'absolute inset-0 aspect-square size-full rounded-full object-cover transition-opacity duration-150',
        isLoaded ? 'opacity-100' : 'opacity-0',
        className
      )}
      src={src}
      loading={loading ?? 'eager'}
      decoding={decoding ?? 'async'}
      onLoad={handleLoad}
      onError={handleError}
      {...props}
    />
  )
}

type AvatarFallbackProps = React.ComponentProps<'span'> & {
  delay?: number
}

function AvatarFallback({ className, delay = 150, ...props }: AvatarFallbackProps) {
  const { status } = useAvatarContext()
  const [delayPassed, setDelayPassed] = React.useState(delay === undefined)

  React.useEffect(() => {
    if (status === 'loaded') {
      setDelayPassed(false)
      return
    }

    if (delay === undefined) {
      setDelayPassed(true)
      return
    }

    setDelayPassed(false)
    const timeout = window.setTimeout(() => setDelayPassed(true), delay)
    return () => window.clearTimeout(timeout)
  }, [delay, status])

  if (status === 'loaded' || !delayPassed) {
    return null
  }

  return (
    <span
      data-slot='avatar-fallback'
      className={cn(
        'absolute inset-0 flex size-full items-center justify-center rounded-full bg-muted text-sm text-muted-foreground group-data-[size=sm]/avatar:text-xs',
        className
      )}
      {...props}
    />
  )
}

function AvatarBadge({ className, ...props }: React.ComponentProps<'span'>) {
  return (
    <span
      data-slot='avatar-badge'
      className={cn(
        'absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full bg-primary text-primary-foreground bg-blend-color ring-2 ring-background select-none',
        'group-data-[size=sm]/avatar:size-2 group-data-[size=sm]/avatar:[&>svg]:hidden',
        'group-data-[size=default]/avatar:size-2.5 group-data-[size=default]/avatar:[&>svg]:size-2',
        'group-data-[size=lg]/avatar:size-3 group-data-[size=lg]/avatar:[&>svg]:size-2',
        className
      )}
      {...props}
    />
  )
}

function AvatarGroup({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='avatar-group'
      className={cn(
        'group/avatar-group flex -space-x-2 *:data-[slot=avatar]:ring-2 *:data-[slot=avatar]:ring-background',
        className
      )}
      {...props}
    />
  )
}

function AvatarGroupCount({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='avatar-group-count'
      className={cn(
        'relative flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-sm text-muted-foreground ring-2 ring-background group-has-data-[size=lg]/avatar-group:size-10 group-has-data-[size=sm]/avatar-group:size-6 [&>svg]:size-4 group-has-data-[size=lg]/avatar-group:[&>svg]:size-5 group-has-data-[size=sm]/avatar-group:[&>svg]:size-3',
        className
      )}
      {...props}
    />
  )
}

export { Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage }
