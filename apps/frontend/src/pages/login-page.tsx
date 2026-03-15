import { Button } from '@/components/ui/button'
import { FieldDescription } from '@/components/ui/field'
import { useApp } from '@/contexts/AppContext'
import { useLoginRedirect } from '@/hooks/use-login-redirect'
import { Seo } from '@/lib/seo'
import { cn } from '@/lib/utils'
import { useEffect } from 'react'
import { useLocation, useSearch } from 'wouter'

function toSafeNext(next: string | null) {
  if (!next) return '/'
  if (!next.startsWith('/')) return '/'
  if (next.startsWith('//')) return '/'
  return next
}

export function LoginPage() {
  const { session, sessionLoading } = useApp()
  const [, setLocation] = useLocation()
  const search = useSearch()
  const next = toSafeNext(new URLSearchParams(search).get('next'))

  useEffect(() => {
    if (sessionLoading || !session) return
    setLocation(next, { replace: true })
  }, [next, session, sessionLoading, setLocation])

  if (sessionLoading || session) {
    return (
      <div className='flex min-h-dvh items-center justify-center p-6 text-sm text-muted-foreground'>
        Loading…
      </div>
    )
  }

  return (
    <>
      <Seo
        title='Login'
        description='Sign in with Discord to access the flora guild dashboard.'
        path='/login'
        noindex
      />
      <div className='flex min-h-dvh items-center justify-center'>
        <LoginForm className='w-full' />
      </div>
    </>
  )
}

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<'div'>) {
  const redirectToLogin = useLoginRedirect()

  return (
    <div
      className={cn('flex w-full flex-1 flex-col justify-center px-4 py-10 lg:px-6', className)}
      {...props}
    >
      <div className='mx-auto w-full max-w-sm'>
        <h1 className='text-balance text-center text-lg font-semibold text-foreground'>
          Welcome back
        </h1>
        <p className='text-pretty text-center text-sm text-muted-foreground'>
          Sign in with Discord to access the flora guild dashboard.
        </p>
        <form
          className='mt-6 space-y-4'
          onSubmit={(event) => {
            event.preventDefault()
            redirectToLogin()
          }}
        >
          <Button type='submit' className='w-full py-2 font-medium'>
            <svg
              aria-hidden='true'
              viewBox='0 0 127.14 96.36'
              className='size-4'
              fill='currentColor'
            >
              <path d='M107.7 8.07A105.15 105.15 0 0 0 81.47 0a72.06 72.06 0 0 0-3.36 6.83A97.68 97.68 0 0 0 49 6.83 72.37 72.37 0 0 0 45.64 0a105.89 105.89 0 0 0-26.24 8.1C2.79 32.65-1.71 56.6.54 80.21h.02A105.73 105.73 0 0 0 32.71 96a77.7 77.7 0 0 0 6.89-11.18 68.42 68.42 0 0 1-10.84-5.18c.91-.66 1.8-1.36 2.66-2.09a75.57 75.57 0 0 0 64.3 0c.87.73 1.76 1.43 2.66 2.09a68.68 68.68 0 0 1-10.86 5.19A77.3 77.3 0 0 0 94.43 96a105.25 105.25 0 0 0 32.17-15.74c2.64-27.37-4.5-51.1-18.9-72.2ZM42.45 65.69c-6.27 0-11.42-5.76-11.42-12.85S36.03 40 42.45 40c6.46 0 11.5 5.81 11.42 12.85 0 7.09-5.03 12.85-11.42 12.85Zm42.24 0c-6.27 0-11.42-5.76-11.42-12.85S78.27 40 84.69 40c6.46 0 11.5 5.81 11.42 12.85 0 7.09-5.03 12.85-11.42 12.85Z' />
            </svg>
            Continue with Discord
          </Button>
        </form>
        <div className='mt-6 space-y-3'>
          <FieldDescription className='text-pretty text-center text-xs text-muted-foreground'>
            flora is currently in{' '}
            <strong>alpha</strong>. Things are subject to change. Please report any bugs, issues, or
            security vulnerabilities to <a href='https://uwu.network/~tasky'>tasky</a>.
          </FieldDescription>
          <FieldDescription className='text-pretty text-center text-xs text-muted-foreground'>
            By clicking continue, you agree to our <a href='/terms-of-service'>Terms of Service</a>
            {' '}
            and <a href='/privacy-policy'>Privacy Policy</a>.
          </FieldDescription>
        </div>
      </div>
    </div>
  )
}
