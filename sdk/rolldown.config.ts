import { defineConfig } from 'rolldown'
import { globalTypes } from './src/build/plugins/global-types'

export default defineConfig([
  // SDK bundle
  {
    input: './src/index.ts',
    platform: 'neutral',
    treeshake: false,
    plugins: [
      globalTypes({
        sdkInput: './src/index.ts',
        runtimeInput: './src/runtime/index.ts',
        output: './global-types.d.ts'
      })
    ],
    output: {
      file: '../runtime-dist/runtime_sdk_bundle.js',
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
  global.MessageFlags = global.flora.MessageFlags;
})(globalThis);
`
    }
  },

  // Runtime prelude
  {
    input: './src/runtime/index.ts',
    platform: 'neutral',
    treeshake: false,
    output: {
      file: '../runtime-dist/runtime_prelude.js',
      format: 'esm'
    }
  }
])
