import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import Monaco from '@uwu/monaco-react'
import { CheckCircle2, Clock, Play, Terminal, XCircle } from 'lucide-react'
import { useState } from 'react'
import { useEffect } from 'react'
// TODO: load this from example/bot.ts; inlined to avoid ESM import issues in Vite dev.
const defaultCode = `const ping = defineCommand({
  name: 'ping',
  description: 'Respond with pong',
  async run(ctx) {
    const embed = { title: 'Pong', fields: [{ name: 'Args', value: ctx.args.join(',') }] }
    await ctx.reply({ embeds: [embed] })
  }
})

createBot({
  prefix: '.',
  commands: [ping]
})
`

type Props = {
  initialCode?: string
  onSave: (code: string) => Promise<void> | void
  guildId: string
}

export function CodeEditor({ initialCode, onSave, guildId }: Props) {
  const [code, setCode] = useState<string>(initialCode || defaultCode)
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle')
  const [saveError, setSaveError] = useState<string | null>(null)
  const [isCodeLoading, setIsCodeLoading] = useState(false)

  useEffect(() => {
    if (initialCode !== undefined) {
      setCode(initialCode || defaultCode)
    }
  }, [initialCode])

  const handleSave = async () => {
    setSaveStatus('saving')
    setSaveError(null)
    try {
      await onSave(code)
      setSaveStatus('saved')
      setTimeout(() => setSaveStatus('idle'), 2000)
    } catch (err: any) {
      setSaveStatus('error')
      setSaveError(err.message || 'Failed to save')
    }
  }

  return (
    <div className='relative flex h-full w-full flex-col bg-zinc-950'>
      <div className='flex items-center gap-3 border-b border-border px-4 py-2 bg-muted/30'>
        <Terminal className='h-4 w-4 text-muted-foreground' />
        <Badge variant='secondary' className='font-mono text-xs'>
          typescript
        </Badge>
        <div className='ml-auto flex items-center gap-2'>
          {saveStatus === 'error' && saveError && (
            <span className='text-xs text-destructive flex items-center gap-1 animate-in fade-in'>
              <XCircle className='h-3 w-3' /> {saveError}
            </span>
          )}
          {saveStatus === 'saved' && (
            <span className='text-xs text-green-600 flex items-center gap-1 animate-in fade-in'>
              <CheckCircle2 className='h-3 w-3' /> Saved
            </span>
          )}
          <Button
            size='sm'
            onClick={handleSave}
            disabled={saveStatus === 'saving' || isCodeLoading}
            className={cn(
              'transition-all',
              saveStatus === 'saved' ? 'bg-green-600 hover:bg-green-700' : ''
            )}
          >
            {saveStatus === 'saving'
              ? <Clock className='mr-2 h-3 w-3 animate-spin' />
              : <Play className='mr-2 h-3 w-3 fill-current' />}
            {saveStatus === 'saved' ? 'Deployed' : 'Deploy'}
          </Button>
        </div>
      </div>
      <div className='relative flex-1 min-h-0'>
        {isCodeLoading && (
          <div className='absolute inset-0 z-10 flex items-center justify-center bg-background/50 backdrop-blur-sm'>
            <Clock className='h-8 w-8 animate-spin text-primary' />
          </div>
        )}
        <div className='h-full w-full'>
          <Monaco
            value={code}
            valOut={setCode}
            lang='typescript'
            theme='vs-dark'
            height='100%'
            filename={`${guildId}.ts`}
            otherCfg={{
              automaticLayout: true,
              fontSize: 14,
              minimap: { enabled: false }
            }}
          />
        </div>
      </div>
    </div>
  )
}
