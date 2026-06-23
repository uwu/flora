import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { mount } from '@vue/test-utils'
import { axe } from 'vitest-axe'
import { afterEach, describe, expect, it } from 'vite-plus/test'
import {
  Button,
  Checkbox,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  Field,
  Input,
  Switch,
  TabsContent,
  TabsList,
  TabsRoot,
  TabsTrigger,
  buttonVariants
} from '../src'

const styleCss = readFileSync(join(import.meta.dirname, '../src/style.css'), 'utf8')

afterEach(() => {
  document.documentElement.className = ''
  document.documentElement.removeAttribute('data-theme')
  document.body.className = ''
  document.body.removeAttribute('data-theme')
})

describe('design system primitives', () => {
  it('paints document surfaces from semantic theme variables', () => {
    expect(styleCss).toMatch(/html\s*{[^}]*background: var\(--fl-color-bg\);/s)
    expect(styleCss).toMatch(/html\s*{[^}]*color: var\(--fl-color-text\);/s)
    expect(styleCss).toMatch(/body\s*{[^}]*background: var\(--fl-color-bg\);/s)
    expect(styleCss).toMatch(/body\s*{[^}]*color: var\(--fl-color-text\);/s)
  })

  it('builds stable button variant classes', () => {
    expect(buttonVariants({ variant: 'primary', size: 'md' })).toContain(
      'bg-[var(--fl-color-primary)]'
    )
    expect(buttonVariants({ variant: 'outline', size: 'icon-sm' })).toContain('size-8')
  })

  it('renders an accessible button without axe violations', async () => {
    const wrapper = mount(Button, {
      slots: {
        default: 'Deploy'
      }
    })

    expect(wrapper.get('[data-slot="button"]').text()).toBe('Deploy')
    expect((await axe(wrapper.element)).violations).toEqual([])
  })

  it('emits checkbox and switch model updates', async () => {
    const checkbox = mount(Checkbox, {
      props: {
        modelValue: false,
        'onUpdate:modelValue': (value: boolean | 'indeterminate') => {
          checkbox.setProps({ modelValue: value })
        }
      },
      slots: {
        default: 'Require review'
      }
    })

    await checkbox.get('[role="checkbox"]').trigger('click')
    expect(checkbox.emitted('update:modelValue')?.at(0)).toEqual([true])

    const toggle = mount(Switch, {
      props: {
        modelValue: false,
        'onUpdate:modelValue': (value: boolean) => {
          toggle.setProps({ modelValue: value })
        }
      },
      slots: {
        default: 'Notify team'
      }
    })

    await toggle.get('[role="switch"]').trigger('click')
    expect(toggle.emitted('update:modelValue')?.at(0)).toEqual([true])
  })

  it('provides field ids and descriptions to controls', () => {
    const wrapper = mount({
      components: { Field, Input },
      template: `
        <Field label="Command" description="Used in release notes.">
          <template #default="{ id, describedBy }">
            <Input :id="id" :aria-describedby="describedBy" />
          </template>
        </Field>
      `
    })

    const input = wrapper.get('input')
    expect(input.attributes('id')).toMatch(/^fl-field-/)
    expect(input.attributes('aria-describedby')).toBe(`${input.attributes('id')}-description`)
  })

  it('renders tabs without axe violations', async () => {
    const wrapper = mount({
      components: { TabsContent, TabsList, TabsRoot, TabsTrigger },
      template: `
        <main>
          <TabsRoot default-value="logs">
            <TabsList aria-label="Deployment views">
              <TabsTrigger value="logs">Logs</TabsTrigger>
              <TabsTrigger value="files">Files</TabsTrigger>
            </TabsList>
            <TabsContent value="logs">Runtime logs</TabsContent>
            <TabsContent value="files">Bundled files</TabsContent>
          </TabsRoot>
        </main>
      `
    })

    expect((await axe(wrapper.element)).violations).toEqual([])
  })

  it('renders an open dialog without axe violations', async () => {
    const wrapper = mount({
      components: {
        DialogContent,
        DialogDescription,
        DialogOverlay,
        DialogPortal,
        DialogRoot,
        DialogTitle
      },
      template: `
        <DialogRoot :open="true">
          <DialogPortal>
            <DialogOverlay />
            <DialogContent>
              <DialogTitle>Deploy revision</DialogTitle>
              <DialogDescription>Confirm deployment for the selected guild.</DialogDescription>
            </DialogContent>
          </DialogPortal>
        </DialogRoot>
      `,
      attachTo: document.body
    })

    expect((await axe(document.body)).violations).toEqual([])
    wrapper.unmount()
  })
})
