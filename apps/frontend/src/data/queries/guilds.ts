import { $api } from '@/lib/openapi-client'

export const guildsQueryOptions = () => $api.queryOptions('get', '/guilds/', {})
