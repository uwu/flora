import { defineConfig } from "rolldown";
import { dts } from "rolldown-plugin-dts";

export default defineConfig([
  {
    input: "./src/index.ts",
    platform: "neutral",
    treeshake: false,
    output: {
      file: "../scripts/runtime_sdk_bundle.js",
      format: "iife",
      name: "flora",
      exports: "named",
      footer: `
;(function (global) {
  if (!global.flora) return;
  global.createBot = global.flora.createBot;
  global.defineCommand = global.flora.defineCommand;
  global.defineSlashCommand = global.flora.defineSlashCommand;
  global.kv = global.flora.kv;
})(globalThis);
`,
    },
  },

  {
    input: "./src/index.ts",
    plugins: [dts({ emitDtsOnly: true })],
  },
]);
