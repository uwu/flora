import type { Meta, StoryObj } from '@storybook/vue3'
import {
  Button,
  Checkbox,
  Field,
  Input,
  RadioGroup,
  Select,
  Slider,
  Switch,
  Textarea
} from '../src'
import type { RadioOption, SelectOption } from '../src'

const meta = {
  title: 'Components/Forms'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const languageOptions: SelectOption[] = [
  { label: 'TypeScript', value: 'typescript' },
  { label: 'JavaScript', value: 'javascript' },
  { label: 'JSON', value: 'json' }
]

const deployOptions: RadioOption[] = [
  { label: 'Immediate', value: 'immediate', description: 'Deploy as soon as checks pass.' },
  { label: 'Manual approval', value: 'manual', description: 'Hold the deployment for review.' },
  { label: 'Scheduled', value: 'scheduled', description: 'Release during a maintenance window.' }
]

export const DeploymentForm: Story = {
  render: () => ({
    components: { Button, Checkbox, Field, Input, RadioGroup, Select, Slider, Switch, Textarea },
    setup() {
      return { deployOptions, languageOptions }
    },
    data() {
      return {
        command: '/deploy',
        notes: 'Ship the latest guild command handlers.',
        language: 'typescript',
        deployMode: 'immediate',
        concurrency: [4],
        notifications: true,
        requireReview: false
      }
    },
    template: `
      <form class="grid w-[min(680px,calc(100vw-48px))] gap-5 rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] p-5 shadow-[var(--fl-shadow-border)]">
        <div class="grid gap-1">
          <h2 class="m-0 text-lg font-750">Deployment settings</h2>
          <p class="m-0 text-sm text-[var(--fl-color-text-muted)]">Configure a guild deployment before handing it to the build service.</p>
        </div>

        <Field label="Command" description="The command used in release notes.">
          <template #default="{ id, describedBy }">
            <Input :id="id" v-model="command" :aria-describedby="describedBy" />
          </template>
        </Field>

        <Field label="Runtime language">
          <Select v-model="language" :options="languageOptions" />
        </Field>

        <Field label="Notes">
          <Textarea v-model="notes" />
        </Field>

        <Field label="Deploy mode">
          <RadioGroup v-model="deployMode" :options="deployOptions" />
        </Field>

        <Field label="Build concurrency" description="How many build workers may run at once.">
          <Slider v-model="concurrency" :min="1" :max="8" label="Build concurrency" />
        </Field>

        <div class="grid gap-2">
          <Switch v-model="notifications">Send deployment notifications</Switch>
          <Checkbox v-model="requireReview">Require reviewer approval</Checkbox>
        </div>

        <div class="flex justify-end gap-2">
          <Button variant="outline">Cancel</Button>
          <Button>Save settings</Button>
        </div>
      </form>
    `
  })
}
