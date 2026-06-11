<script setup lang="ts">
import FeatureShowcase from './FeatureShowcase.vue'
import FeatureShowcaseSymbols from './FeatureShowcaseSymbols.vue'
</script>

<template>
  <section id="features" class="features-section" aria-labelledby="features-title">
    <FeatureShowcaseSymbols />

    <div class="w-full">
      <h2 id="features-title" class="text-4xl sm:text-center mb-2.5">
        <span>Features</span>
        <span class="inline-block text-2xl align-middle ml-1.5">
          <!-- This weird alignment is intentional. -->
          <span class="i-radix-icons-radiobutton text-red-11 inline-block align-top" />
          <span class="i-radix-icons-vercel-logo text-blue-11 inline-block align-middle" />
          <span class="i-radix-icons-image text-green-11 inline-block align-top" />
        </span>
      </h2>
      <p class="text-gray-11 sm:text-center">
        The Discord bot runtime with the hosting, deploys, and state primitives already wired in.
      </p>
      <div
        class="w-[calc(100%+2.5rem)] sm:w-full border-0 sm:border-1 -ml-5 sm:ml-0
        grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-px bg-gray-6 border-gray-6 mt-10"
      >
        <div class="p-5 bg-gray-1 flex flex-col justify-between">
          <div class="feature-showcase w-full h-60 overflow-hidden select-none">
            <FeatureShowcase section="deployments" />
          </div>

          <div>
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-radiobutton inline-block align-middle mr-1.5" />
              <span>Typed SDK</span>
            </h3>
            <p class="text-xl tracking-tight">
              Slash commands, prefix commands, embeds, and Discord helpers with TypeScript types
              from the start.
            </p>
          </div>
        </div>

        <div class="p-5 bg-gray-1 flex flex-col sm:flex-col-reverse justify-between">
          <div class="feature-showcase w-full h-60 overflow-hidden select-none sm:pt-10">
            <FeatureShowcase section="cli" />
          </div>

          <div>
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-vercel-logo inline-block align-middle mr-1.5" />
              <span>Guild Deploys</span>
            </h3>
            <p class="text-xl tracking-tight">
              Bundle and deploy a bot to a guild with one CLI command. No containers, hosts, or
              release scripts to babysit.
            </p>
          </div>
        </div>

        <div class="p-5 bg-gray-1 flex flex-col justify-between">
          <div class="feature-showcase w-full h-60 overflow-hidden select-none">
            <FeatureShowcase section="files" />
          </div>

          <div>
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-image inline-block align-middle mr-1.5" />
              <span>Runtime Batteries</span>
            </h3>
            <p class="text-xl tracking-tight">
              KV storage, secrets, sandboxing, and runtime primitives are built in so bot code can
              stay focused.
            </p>
          </div>
        </div>

        <div class="p-5 bg-gray-1 flex flex-col justify-between">
          <div class="feature-showcase w-full h-40 overflow-hidden select-none">
            <FeatureShowcase section="checks" />
          </div>

          <div>
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-check-circled inline-block align-middle mr-1.5" />
              <span>Live Logs</span>
            </h3>
            <p class="text-xl tracking-tight">
              Follow bot output and runtime errors as events happen.
            </p>
          </div>
        </div>

        <div class="bg-gray-1 flex flex-col justify-between divide-y divide-gray-6">
          <div class="p-5 basis-1/2 flex flex-col justify-end">
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-avatar inline-block align-middle mr-1.5" />
              <span>API-Token Auth</span>
            </h3>
            <p class="text-xl tracking-tight">
              Log in once with a dashboard token and keep deploy commands tied to your account.
            </p>
          </div>
          <div class="p-5 basis-1/2 flex flex-col justify-end">
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-clock inline-block align-middle mr-1.5" />
              <span>KV Storage</span>
            </h3>
            <p class="text-xl tracking-tight">
              Store per-guild state, settings, and small records without wiring up a database.
            </p>
          </div>
        </div>

        <div class="bg-gray-1 flex flex-col justify-between divide-y divide-gray-6">
          <div class="p-5 basis-1/2 flex flex-col justify-end">
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-button inline-block align-middle mr-1.5" />
              <span>Secrets</span>
            </h3>
            <p class="text-xl tracking-tight">
              Keep bot tokens and config outside source with runtime-managed secret values.
            </p>
          </div>
          <div class="p-5 basis-1/2 flex flex-col justify-end">
            <h3 class="mb-2 text-gray-11">
              <span class="i-radix-icons-mix inline-block align-middle mr-1.5" />
              <span>More</span>
            </h3>
            <p class="text-xl tracking-tight">
              Cron events, sandboxed execution, examples, and Discord command helpers are built in.
            </p>
          </div>
        </div>
      </div>
      <div class="mx-0 sm:mx-2.5 mt-5 text-gray-11">
        <p>
          <span class="inline-block align-middle i-radix-icons-info-circled mr-1.5" />
          <span>Some runtime behavior is guild-scoped while flora is in active development.</span>
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.features-section {
  --uno: 'w-full px-5 pt-10 pb-10 text-xl sm:text-lg';
}

.features-section :is(h2, h3, p) {
  margin: 0;
}

.features-section :is(h2, h3) {
  font-weight: inherit;
}

.features-section h3 {
  font-size: inherit;
}

.feature-showcase {
  mask-image: linear-gradient(to bottom, var(--gray1) 50%, transparent 100%);
}

.features-section :deep(.showcase-svg) {
  --black: var(--gray1);
  --white: var(--gray12);
  --red: var(--red11);
  --green: var(--green11, var(--iris11));
  --blue: var(--blue11, var(--iris11));
  --yellow: var(--yellow11);
  --gray: var(--gray10);
  --text: var(--gray11);
  --icon: var(--gray11);
  --borders: var(--gray6);
  --hovers: var(--gray4);
  --editor: var(--gray2);
  --popups: var(--gray3);
  font-family:
    'Geist Mono',
    ui-monospace,
    'SF Mono',
    'Cascadia Code',
    'Source Code Pro',
    Menlo,
    Consolas,
    'DejaVu Sans Mono',
    monospace;
  font-size: 13px;
  letter-spacing: 0;
  white-space: pre;
}

</style>
