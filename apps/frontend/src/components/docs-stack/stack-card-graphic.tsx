import { useEffect, useRef } from 'react'
import type { GraphicType } from './stack-data'

function rng(seed: number) {
  let state = seed >>> 0
  return () => {
    state = (1664525 * state + 1013904223) >>> 0
    return state / 0x100000000
  }
}

function withAlpha(hex: string, alpha: number) {
  const base = hex.replace('#', '')
  const pairs =
    base.length === 3
      ? base.split('').map((v) => v + v)
      : [base.slice(0, 2), base.slice(2, 4), base.slice(4, 6)]
  return `rgba(${parseInt(pairs[0], 16)}, ${parseInt(pairs[1], 16)}, ${parseInt(
    pairs[2],
    16
  )}, ${alpha})`
}

export function StackCardGraphic({
  type,
  seed,
  foreground
}: {
  type: GraphicType
  seed: number
  foreground: string
}) {
  const ref = useRef<HTMLCanvasElement | null>(null)

  useEffect(() => {
    const canvas = ref.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const width = 312
    const height = 192
    const random = rng(seed)
    canvas.width = width
    canvas.height = height
    ctx.clearRect(0, 0, width, height)

    if (type === 'waveformBars') {
      ctx.fillStyle = withAlpha(foreground, 0.5)
      for (let x = 0; x < width; x += 6) {
        const h = 8 + random() * (height - 16)
        ctx.fillRect(x, 0, 2, h)
      }
      return
    }

    if (type === 'gridBlocks') {
      const size = 15
      for (let y = 0; y < height; y += size + 2) {
        for (let x = 0; x < width; x += size + 2) {
          ctx.fillStyle = withAlpha(foreground, 0.1 + random() * 0.35)
          ctx.fillRect(x, y, size, size)
        }
      }
      return
    }

    if (type === 'noiseLines') {
      for (let y = 2; y < height; y += 4) {
        ctx.strokeStyle = withAlpha(foreground, 0.18 + random() * 0.45)
        ctx.lineWidth = 0.5 + random() * 3
        ctx.beginPath()
        ctx.moveTo(0, y)
        ctx.lineTo(width, y + (random() - 0.5) * 8)
        ctx.stroke()
      }
      return
    }

    if (type === 'fluidGrid') {
      const cols = 40
      const cellW = width / cols
      for (let col = 0; col < cols; col++) {
        const barW = Math.max(1.5, random() * 11)
        const x = col * cellW + (cellW - barW) / 2
        const barH = 20
        const y = (col % 2) * 4 + random() * (height - barH - 6)
        ctx.fillStyle = withAlpha(foreground, 0.2 + random() * 0.45)
        ctx.fillRect(x, y, barW, barH)
      }
      return
    }

    ctx.strokeStyle = withAlpha(foreground, 0.2)
    ctx.lineWidth = 1
    for (let x = 0; x <= width; x += 12) {
      ctx.beginPath()
      ctx.moveTo(x, 0)
      ctx.lineTo(x, height)
      ctx.stroke()
    }
    for (let y = 0; y <= height; y += 12) {
      ctx.beginPath()
      ctx.moveTo(0, y)
      ctx.lineTo(width, y)
      ctx.stroke()
    }
    for (let i = 0; i < 6; i++) {
      const x = 8 + random() * (width - 70)
      const y = 8 + random() * (height - 30)
      const w = 30 + random() * 50
      const h = 10 + random() * 20
      ctx.strokeStyle = withAlpha(foreground, 0.38)
      ctx.strokeRect(x, y, w, h)
    }
  }, [foreground, seed, type])

  return <canvas ref={ref} className='h-full w-full' />
}
