import { defineConfig } from 'rolldown'
import { globalTypes } from './src/build/plugins/global-types'
import { buildRuntimeGlobals } from './src/build/runtime-globals'

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
      footer: buildRuntimeGlobals('./src/index.ts')
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
