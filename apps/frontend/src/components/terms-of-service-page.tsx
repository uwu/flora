import { Seo } from '@/lib/seo'

export function TermsOfServicePage() {
  return (
    <main className='mx-auto min-h-svh w-full max-w-4xl px-6 py-12'>
      <Seo
        title='Terms of Service'
        description='Terms of Service for using the flora web application.'
        path='/terms-of-service'
      />
      <article className='prose prose-neutral dark:prose-invert max-w-none'>
        <h1>Terms of Service</h1>
        <p>Effective date: March 7, 2026</p>

        <h2>1. Service scope</h2>
        <p>
          flora is a guild-only Discord bot platform. You are responsible for your guild
          configuration, scripts, and all actions executed through your Discord account.
        </p>

        <h2>2. Account and access</h2>
        <p>
          You must authenticate with Discord and keep your account secure. You may only use flora
          for guilds where you are authorized.
        </p>

        <h2>3. Acceptable use</h2>
        <p>
          Do not abuse the service, attempt unauthorized access, or deploy malicious content.
          Excessive or abusive usage may be restricted.
        </p>

        <h2>4. Availability</h2>
        <p>
          The service may change, be interrupted, or be discontinued at any time. We may perform
          maintenance without prior notice.
        </p>

        <h2>5. Liability</h2>
        <p>
          flora is provided “as is” without warranties. To the maximum extent allowed by law, we are
          not liable for indirect or consequential damages.
        </p>

        <h2>6. Contact</h2>
        <p>Questions can be directed to the flora maintainers.</p>
      </article>
    </main>
  )
}
