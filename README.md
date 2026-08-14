# paperlens

PaperLens is a minimal v1 web application that takes a research paper PDF URL, extracts the content, and returns a six-part summary in JSON format.

## Purpose

- Accept a PDF URL for a paper
- Extract readable text from the source
- Summarize the paper with an LLM
- Return structured JSON with six fields

## Summary fields

- background
- issue
- cause
- proposal
- result
- discussion

## Run the app

### 1. Start the backend

```bash
cd apps/api
cargo run
```

The backend API runs at `http://localhost:3000`.

### 2. Start the frontend

```bash
cd apps/web
npm install
npm run dev
```

The frontend runs at `http://localhost:5173`.

## How to use it

1. Open the frontend in a browser.
2. Enter a paper PDF URL.
3. Click Analyze.
4. Review the paper summary for background, issue, cause, proposal, result, and discussion.

## API specification

### POST /api/papers/analyze

Request:

```json
{
  "url": "https://example.com/paper.pdf"
}
```

Response:

```json
{
  "paper_id": "...",
  "title": "...",
  "source_url": "https://example.com/paper.pdf",
  "summary": {
    "background": "...",
    "issue": "...",
    "cause": "...",
    "proposal": "...",
    "result": "...",
    "discussion": "..."
  },
  "evidence": {
    "background": ["..."],
    "issue": ["..."],
    "cause": ["..."],
    "proposal": ["..."],
    "result": ["..."],
    "discussion": ["..."]
  }
}
```

## Development notes

- Backend: Rust + axum
- Frontend: React + Vite
- LLM: OpenAI Responses API
- Environment variables: `OPENAI_API_KEY`, `OPENAI_MODEL`
