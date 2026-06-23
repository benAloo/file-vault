
import { useEffect, useMemo, useState } from 'react'
import { getHealth, getQuota } from './lib/api'

function App() {
  const [health, setHealth] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [token, setToken] = useState<string>((import.meta.env.VITE_DEV_API_TOKEN as string) || '')
  const [quota, setQuota] = useState<string | null>(null)
  const [copyStatus, setCopyStatus] = useState<string | null>(null)

  const isDev = useMemo(() => import.meta.env.DEV, [])

  useEffect(() => {
    let mounted = true
    getHealth()
      .then((h) => mounted && setHealth(`${h.status} (${h.database})`))
      .catch((e) => mounted && setError(String(e)))
    return () => {
      mounted = false
    }
  }, [])

  async function fetchQuota() {
    setQuota(null)
    setError(null)
    setCopyStatus(null)
    try {
      const q = await getQuota(token)
      setQuota(`used ${q.used_bytes} / ${q.quota_bytes}`)
    } catch (e: any) {
      setError(e.message || String(e))
    }
  }

  async function copyToken() {
    setError(null)
    setCopyStatus(null)

    if (!token) {
      setError('No token available to copy')
      return
    }

    try {
      await navigator.clipboard.writeText(token)
      setCopyStatus('Token copied to clipboard')
    } catch {
      setError('Clipboard access failed in this browser')
    }
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <h1 className="text-4xl font-bold mb-4">File Vault</h1>
      <div className="mb-4">Health: {health ?? 'loading...'}</div>
      <div className="mb-4">Error: {error ?? 'none'}</div>
      {copyStatus && <div className="mb-4 text-emerald-400">{copyStatus}</div>}

      {isDev && (
        <div className="mb-4 rounded border border-white/10 bg-white/5 p-4 text-sm text-slate-300">
          <p className="font-medium text-white">Dev token helper</p>
          <p className="mt-1">
            If you already have a JWT in browser storage, paste it here or copy it from your
            browser devtools and reuse it for the quota request.
          </p>
          <button
            onClick={copyToken}
            className="mt-3 rounded bg-slate-200 px-3 py-2 font-medium text-slate-900 transition hover:bg-white"
            type="button"
          >
            Copy token
          </button>
        </div>
      )}

      <div className="mb-4">
        <label className="block text-sm font-medium">Dev JWT Token (optional)</label>
        <input
          value={token}
          onChange={(e) => setToken(e.target.value)}
          className="border px-2 py-1 w-full"
          placeholder="Paste dev token here"
        />
      </div>

      <button onClick={fetchQuota} className="bg-blue-600 text-white px-4 py-2 rounded">
        Get My Quota
      </button>

      {quota && <div className="mt-4">Quota: {quota}</div>}
    </div>
  )
}

export default App
