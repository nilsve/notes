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
