import type { StorybookConfig } from '@storybook/vue3-vite'
import UnoCSS from 'unocss/vite'

const config: StorybookConfig = {
  stories: ['../stories/**/*.stories.@(ts|tsx|mdx)'],
  addons: ['@storybook/addon-a11y'],
  framework: {
    name: '@storybook/vue3-vite',
    options: {}
  },
  viteFinal(config) {
    config.plugins = [...(config.plugins ?? []), UnoCSS()]
    return config
  }
}

export default config
