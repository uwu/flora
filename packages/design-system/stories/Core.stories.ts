import type { Meta, StoryObj } from '@storybook/vue3'
import {
  Alert,
  Avatar,
  Badge,
  Button,
  Card,
  EmptyState,
  IconButton,
  Progress,
  Separator,
  Skeleton,
  Spinner
} from '../src'

const meta = {
  title: 'Components/Core'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const ControlsAndFeedback: Story = {
  render: () => ({
    components: {
      Alert,
      Avatar,
      Badge,
      Button,
      Card,
      EmptyState,
      IconButton,
      Progress,
      Separator,
      Skeleton,
      Spinner
    },
    template: `
      <div class="grid max-w-220 gap-6">
        <section class="flex flex-wrap items-center gap-2">
          <Button>Deploy</Button>
          <Button variant="secondary">Preview</Button>
          <Button variant="outline">Rollback</Button>
          <Button variant="ghost">Cancel</Button>
          <Button variant="danger">Delete</Button>
          <Button loading>Deploying</Button>
          <IconButton label="Close panel">
            <span class="i-lucide-x size-4" aria-hidden="true" />
          </IconButton>
        </section>

        <section class="flex flex-wrap items-center gap-2">
          <Badge>Queued</Badge>
          <Badge tone="primary">Current</Badge>
          <Badge tone="success">Healthy</Badge>
          <Badge tone="warning">Delayed</Badge>
          <Badge tone="danger">Failed</Badge>
          <Badge tone="info">Build logs</Badge>
        </section>

        <Card class="grid gap-4">
          <div class="flex items-center gap-3">
            <Avatar fallback="FL" />
            <div class="min-w-0">
              <h3 class="m-0 text-base font-750">Deployment health</h3>
              <p class="m-0 text-sm text-[var(--fl-color-text-muted)]">Latest runtime checks and rollout progress.</p>
            </div>
            <Spinner class="ml-auto" />
          </div>
          <Progress :model-value="68" label="Deployment progress" />
          <Separator />
          <Alert tone="warning" title="Rollback available">
            Revision 8f2a19c4 can be restored if the current deployment fails checks.
          </Alert>
        </Card>

        <div class="grid gap-4 md:grid-cols-2">
          <EmptyState title="No logs yet" description="Logs will appear here after the next command dispatch.">
            <template #actions>
              <Button variant="outline" size="sm">Refresh</Button>
            </template>
          </EmptyState>
          <Card class="grid gap-3">
            <Skeleton />
            <Skeleton />
            <Skeleton shape="rect" />
          </Card>
        </div>
      </div>
    `
  })
}
