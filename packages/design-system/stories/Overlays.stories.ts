import type { Meta, StoryObj } from '@storybook/vue3'
import {
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
  Button,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  PopoverContent,
  PopoverPortal,
  PopoverRoot,
  PopoverTrigger,
  TabsContent,
  TabsList,
  TabsRoot,
  TabsTrigger,
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger
} from '../src'

const meta = {
  title: 'Components/Overlays'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const InteractivePrimitives: Story = {
  render: () => ({
    components: {
      AccordionContent,
      AccordionHeader,
      AccordionItem,
      AccordionRoot,
      AccordionTrigger,
      Button,
      DialogClose,
      DialogContent,
      DialogDescription,
      DialogOverlay,
      DialogPortal,
      DialogRoot,
      DialogTitle,
      DialogTrigger,
      DropdownMenuContent,
      DropdownMenuItem,
      DropdownMenuLabel,
      DropdownMenuPortal,
      DropdownMenuRoot,
      DropdownMenuSeparator,
      DropdownMenuTrigger,
      PopoverContent,
      PopoverPortal,
      PopoverRoot,
      PopoverTrigger,
      TabsContent,
      TabsList,
      TabsRoot,
      TabsTrigger,
      TooltipContent,
      TooltipPortal,
      TooltipProvider,
      TooltipRoot,
      TooltipTrigger
    },
    template: `
      <TooltipProvider>
        <div class="grid max-w-220 gap-6">
          <div class="flex flex-wrap items-center gap-2">
            <DialogRoot>
              <DialogTrigger as-child>
                <Button>Open deployment dialog</Button>
              </DialogTrigger>
              <DialogPortal>
                <DialogOverlay />
                <DialogContent>
                  <div class="flex items-start justify-between gap-4">
                    <div class="grid gap-1">
                      <DialogTitle>Deploy revision</DialogTitle>
                      <DialogDescription>
                        Confirm that revision 8f2a19c4 should become the active guild bundle.
                      </DialogDescription>
                    </div>
                    <DialogClose as-child>
                      <Button variant="ghost" size="icon-sm" aria-label="Close dialog">
                        <span class="i-lucide-x size-4" aria-hidden="true" />
                      </Button>
                    </DialogClose>
                  </div>
                  <div class="flex justify-end gap-2">
                    <DialogClose as-child><Button variant="outline">Cancel</Button></DialogClose>
                    <DialogClose as-child><Button>Deploy</Button></DialogClose>
                  </div>
                </DialogContent>
              </DialogPortal>
            </DialogRoot>

            <DropdownMenuRoot>
              <DropdownMenuTrigger as-child>
                <Button variant="outline">Revision actions</Button>
              </DropdownMenuTrigger>
              <DropdownMenuPortal>
                <DropdownMenuContent>
                  <DropdownMenuLabel>Revision</DropdownMenuLabel>
                  <DropdownMenuItem>View source diff</DropdownMenuItem>
                  <DropdownMenuItem>Copy revision ID</DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem class="text-[var(--fl-color-danger-soft-text)]">Rollback</DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenuPortal>
            </DropdownMenuRoot>

            <PopoverRoot>
              <PopoverTrigger as-child>
                <Button variant="secondary">Build metadata</Button>
              </PopoverTrigger>
              <PopoverPortal>
                <PopoverContent>
                  <div class="grid gap-1">
                    <div class="font-700">Build service</div>
                    <p class="m-0 text-sm leading-6 text-[var(--fl-color-text-muted)]">
                      Worker pool is healthy. Last artifact was generated 2 minutes ago.
                    </p>
                  </div>
                </PopoverContent>
              </PopoverPortal>
            </PopoverRoot>

            <TooltipRoot>
              <TooltipTrigger as-child>
                <Button variant="ghost" size="icon">
                  <span class="i-lucide-info size-4" aria-hidden="true" />
                  <span class="sr-only">Deployment help</span>
                </Button>
              </TooltipTrigger>
              <TooltipPortal>
                <TooltipContent>Deployments are isolated per guild.</TooltipContent>
              </TooltipPortal>
            </TooltipRoot>
          </div>

          <TabsRoot default-value="logs">
            <TabsList>
              <TabsTrigger value="logs">Logs</TabsTrigger>
              <TabsTrigger value="files">Files</TabsTrigger>
              <TabsTrigger value="checks">Checks</TabsTrigger>
            </TabsList>
            <TabsContent value="logs" class="rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] p-4 shadow-[var(--fl-shadow-border)]">
              Runtime log stream is ready.
            </TabsContent>
            <TabsContent value="files" class="rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] p-4 shadow-[var(--fl-shadow-border)]">
              18 bundled files changed in this revision.
            </TabsContent>
            <TabsContent value="checks" class="rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] p-4 shadow-[var(--fl-shadow-border)]">
              Typecheck and bundle validation passed.
            </TabsContent>
          </TabsRoot>

          <AccordionRoot type="single" collapsible class="rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] p-2 shadow-[var(--fl-shadow-border)]">
            <AccordionItem value="deploy">
              <AccordionHeader>
                <AccordionTrigger>What happens during deploy?</AccordionTrigger>
              </AccordionHeader>
              <AccordionContent>
                Flora bundles the uploaded files, validates runtime limits, and promotes the revision if checks pass.
              </AccordionContent>
            </AccordionItem>
          </AccordionRoot>
        </div>
      </TooltipProvider>
    `
  })
}
