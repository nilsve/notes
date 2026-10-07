# Frontend

React 19 + Chakra UI v3 single page app for the incident log.

```bash
npm install
npm run dev      # dev server on :5173, proxies /api -> http://localhost:8080
npm run build    # tsc + vite build
npm run lint     # eslint (flat config, fails on warnings)
```

Layout:

- `src/domain/incident.ts` – incident domain model and the markdown metadata codec
- `src/api/incidents.ts`  – typed fetch client for `/api/records` and `/api/ai`
- `src/components/`       – reusable UI (form, cards, badges, markdown, toaster)
- `src/pages/`            – list page and incident detail page

See the repository README for the backend and configuration.
