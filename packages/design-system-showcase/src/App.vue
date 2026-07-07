<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue'
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  GridLines,
  Input,
  Separator,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger
} from '@flora-internal/design-system'

const isDark = ref(false)
const projectName = ref('flora-web')
const themeLabel = computed(() => (isDark.value ? 'Dark' : 'Light'))

watchEffect(() => {
  document.documentElement.classList.toggle('dark', isDark.value)
})
</script>

<template>
  <TooltipProvider>
    <main class="min-h-svh bg-background text-foreground font-sans antialiased">
      <section class="relative mx-auto w-full max-w-6xl space-y-8 px-6 py-10 sm:px-8 lg:px-10">
        <GridLines mode="lines-with-dots" />

        <header class="flex flex-col gap-6 sm:flex-row sm:items-start sm:justify-between">
          <div class="max-w-3xl space-y-4">
            <Badge variant="outline" class="gap-1.5 border-primary/30 bg-primary/10 text-primary">
              <span class="i-lucide-sparkles size-4" aria-hidden="true" />
              Flora design system
            </Badge>
            <div class="space-y-3">
              <h1 class="text-4xl font-semibold tracking-tight text-balance sm:text-6xl">
                Components, tokens, and layout helpers
              </h1>
              <p class="max-w-2xl text-lg text-muted-foreground text-pretty">
                This package exports shadcn-vue components, the presetFlora UnoCSS preset, and a
                small grid-line helper for app frames. The examples below are intentionally boring.
              </p>
            </div>
          </div>

          <Card class="w-full sm:w-72">
            <CardHeader>
              <CardTitle class="text-base">Theme</CardTitle>
              <CardDescription>Class-based dark mode.</CardDescription>
            </CardHeader>
            <CardContent class="flex items-center justify-between text-sm text-muted-foreground">
              <span class="flex items-center gap-2">
                <span :class="isDark ? 'i-lucide-moon' : 'i-lucide-sun'" aria-hidden="true" />
                {{ themeLabel }} mode
              </span>
              <Switch v-model="isDark" aria-label="Toggle dark mode" />
            </CardContent>
          </Card>
        </header>

        <section class="grid gap-4 md:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle class="text-base">Install</CardTitle>
              <CardDescription>Use the workspace package from an app.</CardDescription>
            </CardHeader>
            <CardContent class="space-y-3 text-sm text-muted-foreground">
              <p>Add the package as a dependency and import the CSS once in the app entry.</p>
              <pre
                class="overflow-x-auto rounded-lg border bg-muted p-3 text-xs text-foreground"
              ><code>import '@flora-internal/design-system/style.css'
import 'virtual:uno.css'</code></pre>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle class="text-base">UnoCSS</CardTitle>
              <CardDescription>Use presetFlora with the standard presets.</CardDescription>
            </CardHeader>
            <CardContent class="space-y-3 text-sm text-muted-foreground">
              <p>Configure presetWind4, presetIcons, presetWebFonts, and transformerDirectives.</p>
              <pre
                class="overflow-x-auto rounded-lg border bg-muted p-3 text-xs text-foreground"
              ><code>presets: [presetWind4(), presetIcons(), presetWebFonts(), presetFlora()]</code></pre>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle class="text-base">Tokens</CardTitle>
              <CardDescription>Default semantic hues.</CardDescription>
            </CardHeader>
            <CardContent class="grid grid-cols-2 gap-2 text-sm">
              <div class="rounded-lg bg-primary p-3 text-primary-foreground">primary</div>
              <div class="rounded-lg bg-info-9 p-3 text-info-fg">info</div>
              <div class="rounded-lg bg-tip-9 p-3 text-tip-fg">tip</div>
              <div class="rounded-lg bg-warning-9 p-3 text-warning-fg">warning</div>
              <div class="rounded-lg bg-danger-9 p-3 text-danger-fg">danger</div>
              <div class="rounded-lg border border-neutral-7 bg-neutral-3 p-3 text-foreground">
                neutral
              </div>
            </CardContent>
          </Card>
        </section>

        <Card>
          <CardHeader>
            <CardTitle class="text-base">Components</CardTitle>
            <CardDescription>Common states for generated Vue components.</CardDescription>
          </CardHeader>
          <CardContent>
            <Tabs default-value="buttons" class="w-full">
              <TabsList class="grid w-full grid-cols-3">
                <TabsTrigger value="buttons">Buttons</TabsTrigger>
                <TabsTrigger value="form">Form</TabsTrigger>
                <TabsTrigger value="feedback">Feedback</TabsTrigger>
              </TabsList>
              <TabsContent value="buttons" class="mt-4 flex flex-wrap gap-3">
                <Button>Default</Button>
                <Button variant="secondary">Secondary</Button>
                <Button variant="outline">Outline</Button>
                <Button variant="ghost">Ghost</Button>
                <Button variant="destructive">Destructive</Button>
                <Button size="icon" aria-label="Icon button">
                  <span class="i-lucide-check-circle-2 size-4" aria-hidden="true" />
                </Button>
              </TabsContent>
              <TabsContent value="form" class="mt-4 space-y-3">
                <label class="grid gap-2 text-sm font-medium">
                  Project name
                  <Input v-model="projectName" aria-label="Project name" />
                </label>
                <div class="flex items-center gap-3 rounded-lg border p-3">
                  <Switch :model-value="true" aria-label="Enable previews" />
                  <div>
                    <p class="text-sm font-medium">Preview deployments</p>
                    <p class="text-sm text-muted-foreground">
                      Switch state styles come from data attributes.
                    </p>
                  </div>
                </div>
              </TabsContent>
              <TabsContent value="feedback" class="mt-4 space-y-3 text-sm text-muted-foreground">
                <div class="flex flex-wrap gap-2">
                  <Badge>Default</Badge>
                  <Badge variant="secondary">Secondary</Badge>
                  <Badge variant="outline">Outline</Badge>
                  <Badge variant="destructive">Destructive</Badge>
                </div>
                <Separator />
                <Tooltip>
                  <TooltipTrigger as-child>
                    <Button variant="outline">
                      <span class="i-lucide-book-open size-4" aria-hidden="true" />
                      Tooltip target
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent
                    >Tooltip content uses the shared animation utilities.</TooltipContent
                  >
                </Tooltip>
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>

        <Card class="relative overflow-hidden">
          <GridLines mode="lines-with-dots" variant="above-bottom-dots" />
          <CardHeader>
            <CardTitle class="flex items-center gap-2 text-base">
              <span class="i-lucide-grid-2x2 size-4 text-primary" aria-hidden="true" />
              Grid lines
            </CardTitle>
            <CardDescription>Decorative lines for framed layouts.</CardDescription>
          </CardHeader>
          <CardContent>
            <div class="relative rounded-lg border bg-muted/40 p-4 [--grid-line-offset:16px]">
              <GridLines mode="dashed" />
              <p class="max-w-2xl text-sm text-muted-foreground">
                Place GridLines inside a relative container. Use CSS variables such as
                <code class="rounded bg-background px-1.5 py-0.5 font-mono text-xs text-foreground">
                  --grid-line-offset
                </code>
                and
                <code class="rounded bg-background px-1.5 py-0.5 font-mono text-xs text-foreground">
                  --grid-max-width
                </code>
                to align the frame.
              </p>
            </div>
          </CardContent>
        </Card>
      </section>
    </main>
  </TooltipProvider>
</template>
