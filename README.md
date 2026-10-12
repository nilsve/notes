# Incident Log

A small self-hosted app for logging incidents and how they were solved, with semantic
(AI) search over the whole history.

- **Backend**: Rust (actix-web) + Postgres for storage, Weaviate for vector search,
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
| 5432 | Postgres (internal)                    | **not published** — reachable only on the docker network |
| 8081 | Weaviate (internal)                    | **not published** — reachable only on the docker network |
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
| `WS`     | `/api/ai/ws`          | Same, streamed frame by frame over a WebSocket    |

### Streaming answers

`/api/ai/ws` is a WebSocket. Upgrade, then send one JSON message per question:

```json
{"question":"Have we seen a 500 on checkout before?"}
```

Frames arrive as the model produces them:

```json
{"type":"reasoning","delta":"The user asks about ..."}
{"type":"answer","delta":"Yes — "}
{"type":"done"}
```

`reasoning` carries the model's chain of thought (controlled by
`OPENAI_REASONING_EFFORT`) and renders in a collapsed panel in the UI; `answer`
is the reply. Errors arrive as `{"type":"error","message":"..."}`. The socket
stays open for further questions until the client closes it, and a question that
arrives mid-answer is processed after the current one.

A WebSocket rather than a streamed `POST`/`EventSource` because HTTP responses get
buffered by clients and middleboxes — which is exactly what happened with the
earlier `application/x-ndjson` design — whereas every WebSocket frame lands as it
is sent. The frontend uses the native `WebSocket` (see `streamAboutIncidents` in
`frontend/src/api/incidents.ts`); the Vite dev server proxies it via `ws: true`.

## Configuration (environment variables)

| Variable             | Required | Default                                             | Purpose                                  |
| -------------------- | -------- | --------------------------------------------------- | ---------------------------------------- |
| `OPENAI_API_KEY`     | yes      | –                                                   | API key for your OpenAI-compatible server |
| `OPENAI_API_BASE`    | no       | `https://api.openai.com/v1`                         | Base URL of your OpenAI-compatible server |
| `OPENAI_MODEL`       | no       | `gpt-4o-mini`                                       | Model name served by that server          |
| `OPENAI_REASONING_EFFORT` | no | `medium`                                        | How hard the model thinks: `none`/`minimal`/`low`/`medium`/`high`/`xhigh` |
| `DATABASE_URL`       | yes      | `postgres://incidents:incidents@localhost:5432/incidents` | Postgres connection string          |
| `WEAVIATE_HOSTNAME`  | yes      | –                                                   | Weaviate host                             |
| `WEAVIATE_PORT`      | yes      | –                                                   | Weaviate port                             |
| `FRONTEND_LOCATION`  | no       | `static`                                            | Where the built frontend assets live      |
| `RUSTBERT_CACHE`     | no       | `$XDG_CACHE_HOME/.rustbert`                         | Where the embedding model weights are cached |

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
