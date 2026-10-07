# Incident Log

A small self-hosted app for logging incidents and how they were solved, with semantic
(AI) search over the whole history.

- **Backend**: Rust (actix-web) + DynamoDB for storage, Weaviate for vector search,
  a local sentence-embedding model for indexing, and any OpenAI-compatible server for
  the question answering.
- **Frontend**: React 19 + Chakra UI v3 (Vite + TypeScript).

## Domain model

The backend is deliberately generic: it stores **records** (`id`, `title`, `body`,
plus a vector embedding). The frontend gives those records meaning by treating the
`body` as markdown and keeping the incident metadata in its first line:

```markdown
**Severity:** high · **Status:** resolved

## Summary
...
```

Because the metadata lives inside the markdown, it stays searchable by the semantic
index and renders nicely, while the UI can still parse it into badges and filters.

| Concept   | Values                                                    |
| --------- | --------------------------------------------------------- |
| Severity  | `critical`, `high`, `medium`, `low`                        |
| Status    | `open`, `investigating`, `mitigated`, `resolved`           |

## Ports & routing

The app is **single-port**: one actix-web process serves both the JSON API (under
`/api`) and the compiled SPA (everything else). No reverse proxy or extra port is
needed in front of it — Caddy/Nginx are optional TLS terminators only.

| Port | What                                   | Notes                                            |
| ---- | -------------------------------------- | ------------------------------------------------ |
| 1337 | API + frontend (the only public port)  | `PORT` env var, default `1337`                   |
| 5432 | Postgres (internal)                    | published on `127.0.0.1` only                    |
| 8081 | Weaviate (internal)                    | published on `127.0.0.1` only                    |
| 5173 | Vite dev server (development only)     | proxies `/api` to `http://localhost:1337`         |

Routing inside the app:

- `/api/records*`, `/api/ai` → API handlers
- `/assets/*`, `/vite.svg`, … → static files from `FRONTEND_LOCATION`
- anything else → `index.html`, so SPA deep links (e.g. `/<incident-id>`) survive a
  refresh

`https://notes.nilles.dev` is forwarded to port **1337** on this machine.

## HTTP API

| Method   | Path                  | Description                          |
| -------- | --------------------- | ------------------------------------ |
| `GET`    | `/api/records`        | List records (body truncated)        |
| `GET`    | `/api/records/{id}`   | Fetch one record                     |
| `POST`   | `/api/records`        | Create a record                      |
| `PUT`    | `/api/records/{id}`   | Update a record                      |
| `DELETE` | `/api/records/{id}`   | Delete a record (and its vector)     |
| `POST`   | `/api/ai`             | `{ "question": "..." }` → `{ "answer": "..." }` |

## Configuration (environment variables)

| Variable             | Required | Default                                             | Purpose                                  |
| -------------------- | -------- | --------------------------------------------------- | ---------------------------------------- |
| `OPENAI_API_KEY`     | yes      | –                                                   | API key for your OpenAI-compatible server |
| `OPENAI_API_BASE`    | no       | `https://api.openai.com/v1`                         | Base URL of your OpenAI-compatible server |
| `OPENAI_MODEL`       | no       | `gpt-4o-mini`                                       | Model name served by that server          |
| `DATABASE_URL`       | yes      | `postgres://incidents:incidents@localhost:5432/incidents` | Postgres connection string          |
| `WEAVIATE_HOSTNAME`  | yes      | –                                                   | Weaviate host                             |
| `WEAVIATE_PORT`      | yes      | –                                                   | Weaviate port                             |
| `FRONTEND_LOCATION`  | no       | `static`                                            | Where the built frontend assets live      |

The bundled `.env.example` is pre-filled for the local **halogen** instance
(`https://ssh.nilles.dev/halogen/v1`, model `halogen-qwen3.8-flash-next`); copy it
to `.env` and set `OPENAI_API_KEY`.

The Postgres schema (a single `records` table) is created automatically at boot with
`CREATE TABLE IF NOT EXISTS`, so there is no manual migration step.

## Running

```bash
cp .env.example .env   # fill in your keys
docker compose up --build
```

The app is served on `http://localhost:1337` (and publicly on
`https://notes.nilles.dev`).

### Frontend development

```bash
cd frontend
npm install
npm run dev      # vite dev server, proxies /api to localhost:1337
npm run build    # type-check + production build
npm run lint     # eslint (flat config)
```

## Notes on the upgrade

- Rust toolchain and all crates were updated to their latest releases. `console` is
  pinned in `Cargo.lock` to `0.15.x` because `indicatif@0.16` (a `rust-bert`
  dependency) declares an overly broad range that resolves to a broken `console`
  release.
- `chatgpt_rs` was replaced with [`openai-api-rs`](https://github.com/dongri/openai-api-rs)
  so the assistant can point at any OpenAI-compatible endpoint.
- `chakra-ui-markdown-renderer` was dropped: it is unmaintained and incompatible with
  Chakra v3. Markdown rendering now uses a small component map
  (`frontend/src/components/markdown.tsx`).
- TypeScript is held at `6.x`: `@typescript-eslint` does not support TypeScript 7
  yet.
