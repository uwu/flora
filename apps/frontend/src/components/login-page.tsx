import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { FieldDescription, FieldGroup } from '@/components/ui/field'
import { useLoginRedirect } from '@/hooks/use-login-redirect'
import { cn } from '@/lib/utils'

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<'div'>) {
  const redirectToLogin = useLoginRedirect()

  return (
    <div className={cn('flex flex-col gap-6', className)} {...props}>
      <Card className='overflow-hidden p-0'>
        <CardContent className='grid p-0 md:grid-cols-2'>
          <form className='p-6 md:p-8'>
            <FieldGroup>
              <div className='flex flex-col items-center gap-2 text-center'>
                <h1 className='text-2xl font-bold'>Welcome back</h1>
                <p className='text-balance text-muted-foreground'>
                  Login to your Discord account
                </p>
              </div>
              <div className='flex justify-center pt-4'>
                <Button
                  type='button'
                  variant='outline'
                  className='w-full max-w-xs'
                  onClick={redirectToLogin}
                >
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
              </div>
            </FieldGroup>
          </form>
          <div className='hidden bg-muted md:block'>
            <img
              src='/login.jpg'
              alt='Login artwork'
              className='block h-auto w-full'
            />
          </div>
        </CardContent>
      </Card>
      <FieldDescription className='px-6 text-center'>
        By clicking continue, you agree to our <a href='/terms-of-service'>Terms of Service</a> and
        {' '}
        <a href='/privacy-policy'>Privacy Policy</a>.
      </FieldDescription>
    </div>
  )
}
