import { Helmet } from 'react-helmet-async'

type SeoProps = {
  title?: string
  description?: string
  path?: string
  imagePath?: string
  noindex?: boolean
}

const DEFAULT_ORIGIN = 'https://app.flora.uwu.network'
const SITE_NAME = 'flora'
const DEFAULT_DESCRIPTION =
  'flora is a fast runtime that lets you write Discord bots for your servers with a rich TypeScript SDK, without worrying about running infrastructure.'

function getSiteOrigin() {
  const configuredOrigin = import.meta.env.VITE_PUBLIC_SITE_URL?.trim()
  if (!configuredOrigin) return DEFAULT_ORIGIN
  return configuredOrigin.endsWith('/') ? configuredOrigin.slice(0, -1) : configuredOrigin
}

function toCanonicalUrl(path?: string) {
  const origin = getSiteOrigin()

  if (path?.startsWith('http://') || path?.startsWith('https://')) {
    return path
  }

  const cleanPath = (path ?? '/').split('?')[0].split('#')[0]
  const pathname = cleanPath.startsWith('/') ? cleanPath : `/${cleanPath}`
  return `${origin}${pathname}`
}

function toAbsoluteImageUrl(imagePath?: string) {
  if (!imagePath) return `${getSiteOrigin()}/logo.png`
  if (imagePath.startsWith('http://') || imagePath.startsWith('https://')) return imagePath
  const path = imagePath.startsWith('/') ? imagePath : `/${imagePath}`
  return `${getSiteOrigin()}${path}`
}

export function Seo(
  { title, description = DEFAULT_DESCRIPTION, path, imagePath, noindex = false }: SeoProps
) {
  const pageTitle = title ? `${title} | ${SITE_NAME}` : SITE_NAME
  const canonical = toCanonicalUrl(path)
  const imageUrl = toAbsoluteImageUrl(imagePath)
  const robots = noindex ? 'noindex,nofollow' : 'index,follow'

  return (
    <Helmet>
      <title>{pageTitle}</title>
      <meta name='description' content={description} />
      <link rel='canonical' href={canonical} />
      <meta name='robots' content={robots} />

      <meta property='og:type' content='website' />
      <meta property='og:site_name' content={SITE_NAME} />
      <meta property='og:title' content={pageTitle} />
      <meta property='og:description' content={description} />
      <meta property='og:url' content={canonical} />
      <meta property='og:image' content={imageUrl} />

      <meta name='twitter:card' content='summary_large_image' />
      <meta name='twitter:title' content={pageTitle} />
      <meta name='twitter:description' content={description} />
      <meta name='twitter:image' content={imageUrl} />
    </Helmet>
  )
}
