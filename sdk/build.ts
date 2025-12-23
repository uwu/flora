import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { rolldown } from "rolldown";

const entry = resolve(import.meta.dirname, "src/index.ts");
const output = resolve(import.meta.dirname, "../dist/sdk-bundle.js");
mkdirSync(dirname(output), { recursive: true });

const bundle = await rolldown({
  input: entry,
  platform: "neutral",
  treeshake: false,
});

const { output: chunks } = await bundle.generate({
  format: "iife",
  name: "flora",
  exports: "named",
});

const chunk = chunks.find((item) => item.type === "chunk");
if (!chunk || chunk.type !== "chunk") {
  throw new Error("rolldown did not emit a chunk");
}

let code = chunk.code;
code += `
;(function (global) {
  if (!global.flora) return;
  global.createBot = global.flora.createBot;
  global.defineCommand = global.flora.defineCommand;
  global.defineSlashCommand = global.flora.defineSlashCommand;
})(globalThis);
`;

writeFileSync(output, code, "utf8");
console.log(`sdk bundle written to ${output}`);
