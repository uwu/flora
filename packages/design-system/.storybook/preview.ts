import type { Preview } from '@storybook/vue3'
import { useEffect, useGlobals } from 'storybook/preview-api'
import '../src/style.css'
import 'virtual:uno.css'

function resolveTheme(value: unknown) {
  return value === 'dark' ? 'dark' : 'light'
}

function applyTheme(theme: 'light' | 'dark') {
  document.documentElement.classList.toggle('dark', theme === 'dark')
  document.documentElement.classList.toggle('light', theme === 'light')
  document.documentElement.dataset.theme = theme
  document.body.classList.toggle('dark', theme === 'dark')
  document.body.classList.toggle('light', theme === 'light')
  document.body.dataset.theme = theme
}

const preview: Preview = {
  initialGlobals: {
    theme: 'light'
  },
  parameters: {
    a11y: {
      test: 'todo'
    },
    backgrounds: {
      default: 'surface',
      values: [
        { name: 'surface', value: 'var(--fl-color-bg)' },
        { name: 'subtle', value: 'var(--fl-color-bg-subtle)' }
      ]
    },
    controls: {
      expanded: true
    },
    layout: 'centered'
  },
  globalTypes: {
    theme: {
      description: 'Theme',
      defaultValue: 'light',
      toolbar: {
        icon: 'circlehollow',
        items: ['light', 'dark']
      }
    }
  },
  decorators: [
    (story) => {
      const [globals] = useGlobals()
      const theme = resolveTheme(globals.theme)

      useEffect(() => {
        applyTheme(theme)
      }, [theme])

      applyTheme(theme)

      return {
        components: { Story: story() },
        setup() {
          return { theme }
        },
        template: '<div class="fl-story-root" :class="theme"><Story /></div>'
      }
    }
  ]
}

export default preview
