import { defineConfig } from 'rolldown'
import { globalTypes } from './src/build/plugins/global-types'

export default defineConfig([
  {
    input: './src/index.ts',
    platform: 'neutral',
    treeshake: false,
    plugins: [
      globalTypes({
        input: './src/index.ts',
        output: './global-types.d.ts'
      })
    ],
    output: {
      file: '../scripts/runtime_sdk_bundle.js',
      format: 'iife',
      name: 'flora',
      exports: 'named',
      footer: `
;(function (global) {
  if (!global.flora) return;
  global.createBot = global.flora.createBot;
  global.prefix = global.flora.prefix;
  global.slash = global.flora.slash;
  global.hasRole = global.flora.hasRole;
  global.getSubcommand = global.flora.getSubcommand;
  global.getSubcommandGroup = global.flora.getSubcommandGroup;
  global.kv = global.flora.kv;
  global.EmbedBuilder = global.flora.EmbedBuilder;
  global.embed = global.flora.embed;
})(globalThis);
`
    }
  }
])
