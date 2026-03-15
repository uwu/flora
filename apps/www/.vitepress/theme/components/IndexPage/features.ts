import type { IndexFeature } from './types'

export const features: IndexFeature[] = [
  {
    id: 'sdk',
    title: 'TypeScript SDK',
    desc: 'Write bots with a rich, typed SDK. Slash commands, prefix commands, embeds — all first-class.',
    bg: '/monet02.jpg',
    snippetHtml: `<span class="ts-key">const</span> <span class="ts-var">ping</span> <span class="ts-op">=</span> <span class="ts-fn">slash</span><span class="ts-punc">(</span><span class="ts-punc">{</span>
  <span class="ts-prop">name</span><span class="ts-punc">:</span> <span class="ts-str">'ping'</span><span class="ts-punc">,</span>
  <span class="ts-prop">description</span><span class="ts-punc">:</span> <span class="ts-str">'Pong'</span><span class="ts-punc">,</span>
  <span class="ts-prop">handler</span><span class="ts-punc">:</span> <span class="ts-key">async</span> <span class="ts-punc">(</span><span class="ts-var">i</span><span class="ts-punc">)</span> <span class="ts-op">=&gt;</span> <span class="ts-var">i</span><span class="ts-punc">.</span><span class="ts-fn">reply</span><span class="ts-punc">(</span><span class="ts-punc">{</span> <span class="ts-prop">content</span><span class="ts-punc">:</span> <span class="ts-str">'Pong!'</span> <span class="ts-punc">}</span><span class="ts-punc">)</span>
<span class="ts-punc">}</span><span class="ts-punc">)</span>

<span class="ts-key">const</span> <span class="ts-var">say</span> <span class="ts-op">=</span> <span class="ts-fn">prefix</span><span class="ts-punc">(</span><span class="ts-punc">{</span>
  <span class="ts-prop">name</span><span class="ts-punc">:</span> <span class="ts-str">'say'</span><span class="ts-punc">,</span>
  <span class="ts-prop">description</span><span class="ts-punc">:</span> <span class="ts-str">'Echo text'</span><span class="ts-punc">,</span>
  <span class="ts-prop">handler</span><span class="ts-punc">:</span> <span class="ts-key">async</span> <span class="ts-punc">(</span><span class="ts-var">msg</span><span class="ts-punc">,</span> <span class="ts-var">args</span><span class="ts-punc">)</span> <span class="ts-op">=&gt;</span> <span class="ts-var">msg</span><span class="ts-punc">.</span><span class="ts-fn">reply</span><span class="ts-punc">(</span><span class="ts-var">args</span><span class="ts-punc">.</span><span class="ts-fn">join</span><span class="ts-punc">(</span><span class="ts-str">' '</span><span class="ts-punc">)</span><span class="ts-punc">)</span>
<span class="ts-punc">}</span><span class="ts-punc">)</span>

<span class="ts-fn">createBot</span><span class="ts-punc">(</span><span class="ts-punc">{</span> <span class="ts-prop">prefix</span><span class="ts-punc">:</span> <span class="ts-str">'!'</span><span class="ts-punc">,</span> <span class="ts-prop">commands</span><span class="ts-punc">:</span> <span class="ts-punc">[</span><span class="ts-var">say</span><span class="ts-punc">]</span><span class="ts-punc">,</span> <span class="ts-prop">slashCommands</span><span class="ts-punc">:</span> <span class="ts-punc">[</span><span class="ts-var">ping</span><span class="ts-punc">]</span> <span class="ts-punc">}</span><span class="ts-punc">)</span>`
  },
  {
    id: 'cli',
    title: 'CLI Deploy',
    desc: 'One command to bundle and deploy your bot to any guild. No infra to manage, no containers to run.',
    bg: '/monet01.jpg'
  },
  {
    id: 'runtime',
    title: 'Batteries Included',
    desc: 'Key-value storage, secrets management, sandboxing, and more — all built in, with more to come.',
    bg: '/monet04.jpg',
    snippetHtml: `<span class="ts-cmt">// kv</span>
<span class="ts-key">const</span> <span class="ts-var">guildKv</span> <span class="ts-op">=</span> <span class="ts-var">storage</span><span class="ts-punc">.</span><span class="ts-fn">kv</span><span class="ts-punc">(</span><span class="ts-str">'guild:847291053618249801'</span><span class="ts-punc">)</span>
<span class="ts-key">await</span> <span class="ts-var">guildKv</span><span class="ts-punc">.</span><span class="ts-fn">set</span><span class="ts-punc">(</span><span class="ts-str">'prefix'</span><span class="ts-punc">,</span> <span class="ts-str">'!'</span><span class="ts-punc">)</span>
<span class="ts-key">const</span> <span class="ts-var">prefix</span> <span class="ts-op">=</span> <span class="ts-key">await</span> <span class="ts-var">guildKv</span><span class="ts-punc">.</span><span class="ts-fn">get</span><span class="ts-punc">(</span><span class="ts-str">'prefix'</span><span class="ts-punc">)</span>

<span class="ts-cmt">// cron</span>
<span class="ts-fn">cron</span><span class="ts-punc">(</span><span class="ts-str">'*/5 * * * *'</span><span class="ts-punc">,</span> <span class="ts-key">async</span> <span class="ts-punc">(</span><span class="ts-punc">)</span> <span class="ts-op">=&gt;</span> <span class="ts-punc">{</span>
  <span class="ts-key">await</span> <span class="ts-var">guildKv</span><span class="ts-punc">.</span><span class="ts-fn">set</span><span class="ts-punc">(</span><span class="ts-str">'lastTick'</span><span class="ts-punc">,</span> <span class="ts-var">Date</span><span class="ts-punc">.</span><span class="ts-fn">now</span><span class="ts-punc">(</span><span class="ts-punc">)</span><span class="ts-punc">)</span>
<span class="ts-punc">}</span><span class="ts-punc">)</span>

<span class="ts-cmt">// secrets</span>
<span class="ts-key">const</span> <span class="ts-var">secretValue</span> <span class="ts-op">=</span> <span class="ts-key">await</span> <span class="ts-var">secrets</span><span class="ts-punc">.</span><span class="ts-fn">get</span><span class="ts-punc">(</span><span class="ts-str">'OPENAI_API_KEY'</span><span class="ts-punc">)</span>
<span class="ts-var">console</span><span class="ts-punc">.</span><span class="ts-fn">log</span><span class="ts-punc">(</span><span class="ts-str">'secret:'</span><span class="ts-punc">,</span> <span class="ts-var">secretValue</span><span class="ts-punc">)</span> <span class="ts-cmt">// e.g. FLORA_SECRETS_REF_9f3a2c1d</span>`
  }
]
