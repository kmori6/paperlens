import { useState } from 'react'
import './App.css'

const initialSummary = {
  background: '',
  issue: '',
  cause: '',
  proposal: '',
  result: '',
  discussion: '',
}

const initialEvidence = {
  background: [],
  issue: [],
  cause: [],
  proposal: [],
  result: [],
  discussion: [],
}

function App() {
  const [url, setUrl] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [result, setResult] = useState(null)

  const handleSubmit = async (event) => {
    event.preventDefault()
    setLoading(true)
    setError('')

    try {
      const response = await fetch(`${import.meta.env.VITE_API_URL ?? 'http://localhost:3000'}/api/papers/analyze`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url }),
      })

      if (!response.ok) {
        const payload = await response.json().catch(() => ({}))
        throw new Error(payload.error ?? 'Failed to analyze paper')
      }

      const data = await response.json()
      setResult(data)
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="page-shell">
      <section className="panel">
        <p className="eyebrow">paperlens</p>
        <h1>Research paper summary</h1>
        <form className="search-form" onSubmit={handleSubmit}>
          <label htmlFor="paper-url">PDF URL</label>
          <div className="input-row">
            <input
              id="paper-url"
              type="url"
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://.../paper.pdf"
              required
            />
            <button type="submit" disabled={loading}>
              {loading ? 'Analyzing…' : 'Analyze'}
            </button>
          </div>
        </form>

        {error && <div className="error-box">{error}</div>}

        {result && (
          <article className="result-card">
            <div className="result-header">
              <h2>{result.title || 'Paper summary'}</h2>
              <a href={result.source_url} target="_blank" rel="noreferrer">Open source</a>
            </div>

            <div className="summary-grid">
              {Object.entries(result.summary ?? initialSummary).map(([key, value]) => (
                <section key={key} className="summary-item">
                  <h3>{key}</h3>
                  <p>{value}</p>
                  <ul>
                    {(result.evidence?.[key] ?? initialEvidence[key] ?? []).map((item) => (
                      <li key={`${key}-${item}`}>{item}</li>
                    ))}
                  </ul>
                </section>
              ))}
            </div>
          </article>
        )}
      </section>
    </main>
  )
}

export default App
