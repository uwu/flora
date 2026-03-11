import { $api } from '@/lib/openapi-client'

export const tokensQueryOptions = () => $api.queryOptions('get', '/tokens/', {})
