import { $api } from '@/lib/openapi-client'

export const authSessionQueryOptions = () => $api.queryOptions('get', '/auth/me', {})
