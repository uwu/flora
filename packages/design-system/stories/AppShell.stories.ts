import type { Meta, StoryObj } from '@storybook/vue3'
import {
  AppShell,
  Avatar,
  Badge,
  Button,
  Card,
  Sidebar,
  SidebarItem,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '../src'

const meta = {
  title: 'Patterns/App Shell',
  parameters: {
    layout: 'fullscreen'
  }
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Dashboard: Story = {
  render: () => ({
    components: {
      AppShell,
      Avatar,
      Badge,
      Button,
      Card,
      Sidebar,
      SidebarItem,
      Table,
      TableBody,
      TableCell,
      TableHead,
      TableHeader,
      TableRow
    },
    template: `
      <AppShell class="min-h-[720px]">
        <template #sidebar>
          <Sidebar title="flora">
            <template #default="{ open }">
              <div class="grid gap-1">
                <SidebarItem href="#" active>
                  <span class="i-lucide-layout-dashboard size-4" aria-hidden="true" />
                  <span v-if="open">Overview</span>
                </SidebarItem>
                <SidebarItem href="#">
                  <span class="i-lucide-package size-4" aria-hidden="true" />
                  <span v-if="open">Deployments</span>
                </SidebarItem>
                <SidebarItem href="#">
                  <span class="i-lucide-database size-4" aria-hidden="true" />
                  <span v-if="open">KV stores</span>
                </SidebarItem>
              </div>
            </template>
            <template #footer="{ open }">
              <div class="flex items-center gap-2">
                <Avatar fallback="TA" size="sm" />
                <div v-if="open" class="min-w-0 text-sm">
                  <div class="truncate font-650">taskylizard</div>
                  <div class="truncate text-xs text-[var(--fl-color-text-subtle)]">Owner</div>
                </div>
              </div>
            </template>
          </Sidebar>
        </template>

        <section class="grid gap-6 p-6">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="grid gap-1">
              <h1 class="m-0 text-2xl font-750">Guild overview</h1>
              <p class="m-0 text-sm text-[var(--fl-color-text-muted)]">Manage active deployments and runtime state.</p>
            </div>
            <Button>New deployment</Button>
          </div>

          <div class="grid gap-4 md:grid-cols-3">
            <Card>
              <div class="text-sm text-[var(--fl-color-text-muted)]">Active revision</div>
              <div class="mt-1 text-2xl font-750 fl-tabular-nums">8f2a19c4</div>
            </Card>
            <Card>
              <div class="text-sm text-[var(--fl-color-text-muted)]">Commands dispatched</div>
              <div class="mt-1 text-2xl font-750 fl-tabular-nums">12,480</div>
            </Card>
            <Card>
              <div class="text-sm text-[var(--fl-color-text-muted)]">Runtime status</div>
              <div class="mt-2"><Badge tone="success">Healthy</Badge></div>
            </Card>
          </div>

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Revision</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Actor</TableHead>
                <TableHead>Deployed</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell class="font-mono fl-tabular-nums">8f2a19c4</TableCell>
                <TableCell><Badge tone="success">Current</Badge></TableCell>
                <TableCell>@taskylizard</TableCell>
                <TableCell>2m ago</TableCell>
              </TableRow>
              <TableRow>
                <TableCell class="font-mono fl-tabular-nums">5d8be001</TableCell>
                <TableCell><Badge>Previous</Badge></TableCell>
                <TableCell>@taskylizard</TableCell>
                <TableCell>1h ago</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </section>
      </AppShell>
    `
  })
}
