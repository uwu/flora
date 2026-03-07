import { Seo } from '@/lib/seo'

export function PrivacyPolicyPage() {
  return (
    <main className='mx-auto min-h-svh w-full max-w-4xl px-6 py-12'>
      <Seo
        title='Privacy Policy'
        description='Privacy Policy for the flora web application.'
        path='/privacy-policy'
      />
      <article className='prose prose-neutral dark:prose-invert max-w-none'>
        <h1>Privacy Policy</h1>
        <p>Effective date: March 7, 2026</p>

        <h2>1. Data we process</h2>
        <p>
          We process Discord account and guild metadata required to authenticate you and operate the
          flora dashboard.
        </p>

        <h2>2. How data is used</h2>
        <p>
          Data is used to provide core functionality such as login, deployment management, logs, and
          guild configuration.
        </p>

        <h2>3. Retention</h2>
        <p>
          Data is retained for as long as needed to provide the service and comply with operational
          or legal requirements.
        </p>

        <h2>4. Security</h2>
        <p>
          We apply reasonable technical and organizational safeguards, but no system can guarantee
          absolute security.
        </p>

        <h2>5. Your controls</h2>
        <p>
          You can revoke access by disconnecting the app in Discord and by discontinuing use of the
          service.
        </p>

        <h2>6. Contact</h2>
        <p>Privacy questions can be directed to the flora maintainers.</p>
      </article>
    </main>
  )
}
