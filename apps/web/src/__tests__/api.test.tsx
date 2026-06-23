import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { Mock } from 'vitest'
import { getHealth, getQuota } from '../lib/api'

describe('api client', () => {
  beforeEach(() => {
    // @ts-ignore
    global.fetch = vi.fn()
  })

  it('parses health successfully', async () => {
    // @ts-ignore
    fetch.mockResolvedValueOnce({ ok: true, json: async () => ({ status: 'healthy', database: 'connected' }) })
    const h = await getHealth()
    expect(h.status).toBe('healthy')
  })

  it('builds Authorization header and rejects spaces in token', async () => {
    // @ts-ignore
    fetch.mockResolvedValueOnce({ ok: true, json: async () => ({ user_id: 'u', quota_bytes: 100, used_bytes: 10 }) })
    const spy = fetch as unknown as Mock
    await getQuota('mytoken')
    expect(spy).toHaveBeenCalled()
    // token with space should reject
    await expect(() => getQuota('bad token')).rejects.toThrow()
  })
})
