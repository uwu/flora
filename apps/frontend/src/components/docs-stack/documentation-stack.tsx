import { ArrowUpRight } from 'lucide-react'
import { domAnimation, LazyMotion, m } from 'motion/react'
import { useEffect, useMemo, useState } from 'react'

import { StackCardGraphic } from './stack-card-graphic'
import { CARD_SIZES, CLUSTER_LAYOUT, STACK_ANIMATION, STACK_CARDS } from './stack-data'

const COLLAPSED_ROW = { offsetX: 0, offsetY: 220, spacing: 70 }

function useClusterScale(viewportWidth: number) {
  return useMemo(() => {
    const clamped = Math.max(400, Math.min(1080, viewportWidth))
    const normalized = (clamped - 400) / 680
    return {
      offsetMultiplier: 0.4 + 0.6 * normalized,
      clusterScale: 0.8 + 0.2 * normalized
    }
  }, [viewportWidth])
}

export function DocumentationStack() {
  const [selectedCardId, setSelectedCardId] = useState<string | null>(null)
  const [viewportWidth, setViewportWidth] = useState(1080)
  const [animationsReady, setAnimationsReady] = useState(false)
  const descriptionFontSize = viewportWidth <= 768 ? 17.5 : 16
  const { offsetMultiplier, clusterScale } = useClusterScale(viewportWidth)

  useEffect(() => {
    const onResize = () => setViewportWidth(window.innerWidth)
    onResize()
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  useEffect(() => {
    const timer = window.setTimeout(() => setAnimationsReady(true), 1100)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <div
      className='relative w-full flex items-center justify-center'
      style={{ minHeight: 600 }}
      onClick={() => setSelectedCardId(null)}
    >
      <LazyMotion features={domAnimation}>
        <m.div
          className='relative'
          style={{ width: 900, height: 550 }}
          initial={{ filter: 'blur(8px)', scale: 0.8 * clusterScale, opacity: 0, y: 120 }}
          animate={{ filter: 'blur(0px)', scale: clusterScale, opacity: 1, y: 0 }}
          transition={{ duration: 0.4, ease: 'easeOut', delay: 0.5 }}
        >
          {STACK_CARDS.map((card, index) => {
            // Utter garbage.
            const isSelected = selectedCardId === card.id
            const compact = CARD_SIZES.compact
            const expanded = CARD_SIZES.expanded
            const metrics = isSelected ? expanded : compact
            const cluster = CLUSTER_LAYOUT.expanded[card.id]
            const collapsed = CLUSTER_LAYOUT.collapsed[card.id]

            let x = 0
            let y = 0
            let rotate = 0
            let scale = 1

            if (isSelected) {
              x = 0
              y = -40
              rotate = 0
              scale = 1
            } else if (selectedCardId === null) {
              x = cluster.offsetX * offsetMultiplier
              y = cluster.offsetY
              rotate = cluster.rotation
              scale = 1
            } else {
              const rowWidth = (STACK_CARDS.length - 1) * COLLAPSED_ROW.spacing
              x = COLLAPSED_ROW.offsetX + index * COLLAPSED_ROW.spacing - rowWidth / 2 +
                collapsed.offsetX
              y = COLLAPSED_ROW.offsetY + collapsed.offsetY
              rotate = collapsed.rotation
              scale = 0.7 * collapsed.scale
            }

            const titleTop = isSelected
              ? metrics.padding + metrics.graphicH + 16
              : metrics.cardH - metrics.padding - metrics.titleLineH * card.titleLines
            const introY = y - metrics.cardH / 2 + Math.max(25, 50 * Math.abs(index - 2)) +
              48 * index
            const delay = 0.5 + 0.016 * index
            const descriptionTop = expanded.padding + expanded.graphicH + 16 +
              expanded.titleLineH * card.titleLines + 12
            const descriptionWidth = expanded.cardW - 2 * expanded.padding

            return (
              <m.div
                key={card.id}
                onClick={(event) => {
                  event.stopPropagation()
                  setSelectedCardId((prev) => (prev === card.id ? null : card.id))
                }}
                className='absolute cursor-pointer origin-center select-none'
                initial={{ x: x - metrics.cardW / 2 + (2 - index) * 50, y: introY, scale, rotate }}
                style={{ left: '50%', top: '50%', zIndex: index + 1 }}
                animate={{ x: x - metrics.cardW / 2, y: y - metrics.cardH / 2, scale, rotate }}
                whileHover={selectedCardId
                  ? {}
                  : { scale: 1.03 * scale, y: y - metrics.cardH / 2 - 8 }}
                transition={{
                  ...STACK_ANIMATION.card,
                  ...(animationsReady ? {} : { delay }),
                  x: {
                    type: 'spring',
                    visualDuration: 0.5,
                    bounce: 0.3,
                    ...(animationsReady ? {} : { delay })
                  },
                  y: {
                    type: 'spring',
                    visualDuration: 0.5,
                    bounce: 0.3,
                    ...(animationsReady ? {} : { delay })
                  }
                }}
              >
                <m.div
                  className='relative overflow-hidden rounded-2xl'
                  style={{ backgroundColor: card.background }}
                  animate={{ width: metrics.cardW, height: metrics.cardH }}
                  transition={STACK_ANIMATION.card}
                >
                  <m.div
                    className='absolute overflow-hidden'
                    animate={{
                      left: metrics.padding,
                      top: metrics.padding,
                      width: metrics.graphicW,
                      height: metrics.graphicH
                    }}
                    transition={STACK_ANIMATION.card}
                  >
                    <m.div
                      style={{ width: 312, height: 192, transformOrigin: 'top left' }}
                      animate={{ scale: metrics.graphicW / 312 }}
                      transition={STACK_ANIMATION.card}
                    >
                      <StackCardGraphic
                        type={card.graphicType}
                        seed={card.seed}
                        foreground={card.foreground}
                      />
                    </m.div>
                  </m.div>

                  <m.h3
                    className='absolute whitespace-pre-line font-serif'
                    style={{
                      color: card.foreground,
                      fontSize: metrics.titleSize,
                      lineHeight: `${metrics.titleLineH}px`,
                      transition:
                        'font-size 0.4s cubic-bezier(0.34, 1.56, 0.64, 1), line-height 0.4s cubic-bezier(0.34, 1.56, 0.64, 1)'
                    }}
                    animate={{ top: titleTop, left: metrics.padding }}
                    transition={STACK_ANIMATION.card}
                  >
                    {card.title}
                  </m.h3>

                  <m.div
                    className='absolute'
                    style={{ left: expanded.padding, top: descriptionTop, width: descriptionWidth }}
                    animate={{
                      opacity: Number(isSelected),
                      filter: isSelected ? 'blur(0px)' : 'blur(4px)'
                    }}
                    transition={STACK_ANIMATION.description}
                  >
                    <p
                      style={{
                        color: card.bodyColor,
                        fontSize: descriptionFontSize,
                        lineHeight: '24px'
                      }}
                    >
                      {card.description}
                    </p>
                  </m.div>

                  <m.a
                    href={card.href}
                    target='_blank'
                    rel='noopener noreferrer'
                    onClick={(event) => event.stopPropagation()}
                    className='absolute right-4 bottom-4 inline-flex items-center justify-center'
                    animate={{ opacity: isSelected ? 1 : 0.85 }}
                    transition={STACK_ANIMATION.description}
                  >
                    <ArrowUpRight className='h-4 w-4' style={{ color: card.foreground }} />
                  </m.a>
                </m.div>
              </m.div>
            )
          })}
        </m.div>
      </LazyMotion>
    </div>
  )
}
