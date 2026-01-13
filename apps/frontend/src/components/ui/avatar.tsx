import { cn } from '@/lib/utils'
import { useState } from 'react'

function getInitials(name?: string | null) {
  if (!name) return '?'
  const parts = name.split(' ').filter(Boolean)
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
  return (parts[0][0] + parts[1][0]).toUpperCase()
}

function getDiscordAvatarUrl(userId: string, avatarHash: string) {
  const ext = avatarHash.startsWith('a_') ? 'gif' : 'png'
  return `https://cdn.discordapp.com/avatars/${userId}/${avatarHash}.${ext}`
}

function getDiscordGuildIconUrl(guildId: string, iconHash: string) {
  const ext = iconHash.startsWith('a_') ? 'gif' : 'png'
  return `https://cdn.discordapp.com/icons/${guildId}/${iconHash}.${ext}`
}

export function Avatar({
  name,
  className,
  src,
  userId,
  avatarHash,
  guildId,
  iconHash
}: {
  name?: string | null
  className?: string
  src?: string | null
  userId?: string
  avatarHash?: string | null
  guildId?: string
  iconHash?: string | null
}) {
  const [error, setError] = useState(false)
  const imageUrl = src ||
    (userId && avatarHash ? getDiscordAvatarUrl(userId, avatarHash) : null) ||
    (guildId && iconHash ? getDiscordGuildIconUrl(guildId, iconHash) : null)

  if (imageUrl && !error) {
    return (
      <div
        className={cn(
          'flex h-9 w-9 items-center justify-center rounded-full overflow-hidden bg-muted',
          className
        )}
      >
        <img
          src={imageUrl}
          alt={name || 'Avatar'}
          className='h-full w-full object-cover'
          onError={() => setError(true)}
        />
      </div>
    )
  }

  return (
    <div
      className={cn(
        'flex h-9 w-9 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary',
        className
      )}
    >
      {getInitials(name)}
    </div>
  )
}
