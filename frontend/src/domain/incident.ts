export type Severity = 'critical' | 'high' | 'medium' | 'low'

export type IncidentStatus = 'open' | 'investigating' | 'mitigated' | 'resolved'

/** Shape returned by the backend `/api/records` endpoints. */
export interface IncidentRecord {
  id: string
  title: string
  body: string
}

/** Domain model: a record enriched with the metadata encoded in its markdown body. */
export interface Incident {
  id: string
  title: string
  severity: Severity
  status: IncidentStatus
  /** Markdown body without the metadata line. */
  content: string
}

export const SEVERITIES: Severity[] = ['critical', 'high', 'medium', 'low']

export const STATUSES: IncidentStatus[] = ['open', 'investigating', 'mitigated', 'resolved']

export const SEVERITY_LABEL: Record<Severity, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
}

export const SEVERITY_COLOR: Record<Severity, string> = {
  critical: 'red',
  high: 'orange',
  medium: 'yellow',
  low: 'blue',
}

export const STATUS_LABEL: Record<IncidentStatus, string> = {
  open: 'Open',
  investigating: 'Investigating',
  mitigated: 'Mitigated',
  resolved: 'Resolved',
}

export const STATUS_COLOR: Record<IncidentStatus, string> = {
  open: 'red',
  investigating: 'orange',
  mitigated: 'yellow',
  resolved: 'green',
}

const SEVERITY_PATTERN = /\*\*Severity:\*\*\s*([a-z0-9-]+)/i
const STATUS_PATTERN = /\*\*Status:\*\*\s*([a-z0-9-]+)/i

const DEFAULT_SEVERITY: Severity = 'medium'
const DEFAULT_STATUS: IncidentStatus = 'open'

function asSeverity(value: string | undefined, fallback: Severity): Severity {
  const normalized = value?.trim().toLowerCase()
  return SEVERITIES.includes(normalized as Severity) ? (normalized as Severity) : fallback
}

function asStatus(value: string | undefined, fallback: IncidentStatus): IncidentStatus {
  const normalized = value?.trim().toLowerCase()
  return STATUSES.includes(normalized as IncidentStatus) ? (normalized as IncidentStatus) : fallback
}

/**
 * Incidents are stored as plain records. Severity and status live in the first markdown
 * line (`**Severity:** high · **Status:** resolved`) so they stay searchable by the
 * semantic index while rendering nicely as markdown.
 */
export function parseIncident(record: IncidentRecord): Incident {
  const severity = asSeverity(record.body.match(SEVERITY_PATTERN)?.[1], DEFAULT_SEVERITY)
  const status = asStatus(record.body.match(STATUS_PATTERN)?.[1], DEFAULT_STATUS)

  const content = record.body
    .split('\n')
    .filter((line) => !SEVERITY_PATTERN.test(line) && !STATUS_PATTERN.test(line))
    .join('\n')
    .replace(/^\s*\n+/, '')
    .trim()

  return { id: record.id, title: record.title, severity, status, content }
}

export function toRecord(incident: {
  id?: string
  title: string
  severity: Severity
  status: IncidentStatus
  content: string
}): { id?: string; title: string; body: string } {
  const meta = `**Severity:** ${incident.severity} · **Status:** ${incident.status}`
  const body = incident.content.trim() ? `${meta}\n\n${incident.content.trim()}\n` : `${meta}\n`

  return {
    ...(incident.id ? { id: incident.id } : {}),
    title: incident.title,
    body,
  }
}

export const NEW_INCIDENT_TEMPLATE = `## Summary

What happened, in one or two sentences.

## Impact

Who/what was affected, and how badly.

## Detection

How we noticed it (alert, customer report, ...).

## Root cause

Why it happened.

## Resolution

What we did to fix it.

## Lessons learned

What we will do differently next time.
`

/** Plain-text preview of an incident, used on the list cards. */
export function excerpt(content: string, maxLength = 180): string {
  const plain = content
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/`([^`]*)`/g, '$1')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^\s*[-*+]\s+/gm, '')
    .replace(/^>\s?/gm, '')
    .replace(/[*_~]/g, '')
    .replace(/\s+/g, ' ')
    .trim()

  if (plain.length <= maxLength) {
    return plain
  }

  return `${plain.slice(0, maxLength).trimEnd()}…`
}
