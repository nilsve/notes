import { parseIncident, toRecord, type Incident, type IncidentRecord } from '../domain/incident'

const API_URL = '/api'

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: {
      Accept: 'application/json',
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      ...init.headers,
    },
  })

  if (!response.ok) {
    const details = await response.text().catch(() => '')
    throw new Error(details || `Request failed (${response.status})`)
  }

  if (response.status === 204) {
    return undefined as T
  }

  return (await response.json()) as T
}

export async function listIncidents(): Promise<Incident[]> {
  const records = await request<IncidentRecord[]>('/records')
  return records.map(parseIncident)
}

export async function getIncident(id: string): Promise<Incident | null> {
  const record = await request<IncidentRecord | null>(`/records/${id}`)
  return record ? parseIncident(record) : null
}

export async function createIncident(input: {
  title: string
  severity: Incident['severity']
  status: Incident['status']
  content: string
}): Promise<Incident> {
  const record = await request<IncidentRecord>('/records', {
    method: 'POST',
    body: JSON.stringify({ title: input.title, body: toRecord(input).body }),
  })

  return parseIncident(record)
}

export async function updateIncident(incident: Incident): Promise<Incident> {
  const record = await request<IncidentRecord>(`/records/${incident.id}`, {
    method: 'PUT',
    body: JSON.stringify({ id: incident.id, title: incident.title, body: toRecord(incident).body }),
  })

  return parseIncident(record)
}

export async function deleteIncident(id: string): Promise<void> {
  await request<void>(`/records/${id}`, { method: 'DELETE' })
}

export async function askAboutIncidents(question: string): Promise<string> {
  const { answer } = await request<{ answer: string }>('/ai', {
    method: 'POST',
    body: JSON.stringify({ question }),
  })

  return answer
}

/** One frame of the `/api/ai/ws` stream. */
export type AskStreamEvent =
  | { type: 'reasoning'; delta: string }
  | { type: 'answer'; delta: string }
  | { type: 'done' }
  | { type: 'error'; message: string }

export interface AskStreamHandlers {
  /** Chain-of-thought tokens, as produced by the model's reasoning stage. */
  onReasoning?: (delta: string) => void
  /** Answer tokens. */
  onAnswer?: (delta: string) => void
  /** Aborts the request; resolves rather than rejecting. */
  signal?: AbortSignal
}

/** Mirrors the page's scheme/host onto a WebSocket URL. */
function wsUrl(path: string): string {
  const scheme = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${scheme}//${window.location.host}${path}`
}

/**
 * Streaming variant of {@link askAboutIncidents} over a WebSocket.
 *
 * A streamed `POST` looked right in curl but got buffered in real deployments,
 * so the whole answer landed at once. A socket is a genuine bidirectional
 * channel: every frame arrives as the server sends it.
 *
 * Resolves when the answer is complete (or the caller aborts), rejects on a
 * transport failure or a server `error` frame.
 */
export function streamAboutIncidents(
  question: string,
  handlers: AskStreamHandlers = {},
): Promise<void> {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(wsUrl('/api/ai/ws'))

    let settled = false
    let aborted = false

    const settle = (settleWith: () => void) => {
      if (settled) {
        return
      }

      settled = true
      handlers.signal?.removeEventListener('abort', onAbort)

      if (socket.readyState === WebSocket.CONNECTING || socket.readyState === WebSocket.OPEN) {
        socket.close()
      }

      settleWith()
    }

    const onAbort = () => {
      aborted = true
      settle(resolve)
    }

    handlers.signal?.addEventListener('abort', onAbort, { once: true })

    socket.onopen = () => {
      socket.send(JSON.stringify({ question }))
    }

    socket.onmessage = (event) => {
      let frame: AskStreamEvent

      try {
        frame = JSON.parse(String(event.data)) as AskStreamEvent
      } catch {
        return
      }

      if (frame.type === 'reasoning') {
        handlers.onReasoning?.(frame.delta)
      } else if (frame.type === 'answer') {
        handlers.onAnswer?.(frame.delta)
      } else if (frame.type === 'error') {
        settle(() => reject(new Error(frame.message)))
      } else if (frame.type === 'done') {
        settle(resolve)
      }
    }

    socket.onerror = () => {
      settle(() => reject(new Error('Could not open the streaming connection')))
    }

    socket.onclose = (event) => {
      if (aborted) {
        settle(resolve)
        return
      }

      settle(() => reject(new Error(`Streaming connection closed (${event.code})`)))
    }
  })
}
