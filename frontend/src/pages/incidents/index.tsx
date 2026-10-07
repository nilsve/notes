import { useEffect, useMemo, useState } from 'react'
import {
  Box,
  Button,
  EmptyState,
  HStack,
  Input,
  SimpleGrid,
  Skeleton,
  SkeletonText,
  Text,
} from '@chakra-ui/react'
import { listIncidents } from '../../api/incidents'
import {
  SEVERITIES,
  SEVERITY_LABEL,
  STATUSES,
  STATUS_LABEL,
  type Incident,
  type IncidentStatus,
  type Severity,
} from '../../domain/incident'
import { AskPanel } from '../../components/ask-panel'
import { IncidentCard } from '../../components/incident-card'

const STATUS_RANK: Record<IncidentStatus, number> = {
  open: 0,
  investigating: 1,
  mitigated: 2,
  resolved: 3,
}

const SEVERITY_RANK: Record<Severity, number> = {
  critical: 0,
  high: 1,
  medium: 2,
  low: 3,
}

function sortIncidents(incidents: Incident[]): Incident[] {
  return [...incidents].sort((a, b) => {
    const byStatus = STATUS_RANK[a.status] - STATUS_RANK[b.status]
    if (byStatus !== 0) {
      return byStatus
    }

    const bySeverity = SEVERITY_RANK[a.severity] - SEVERITY_RANK[b.severity]
    if (bySeverity !== 0) {
      return bySeverity
    }

    return a.title.localeCompare(b.title)
  })
}

type LoadState =
  | { status: 'loading' }
  | { status: 'error'; message: string }
  | { status: 'ready'; incidents: Incident[] }

export const IncidentsPage: React.FC = () => {
  const [state, setState] = useState<LoadState>({ status: 'loading' })
  const [query, setQuery] = useState('')
  const [severityFilter, setSeverityFilter] = useState<Severity | null>(null)
  const [statusFilter, setStatusFilter] = useState<IncidentStatus | null>(null)

  useEffect(() => {
    let cancelled = false

    listIncidents()
      .then((result) => {
        if (!cancelled) {
          setState({ status: 'ready', incidents: result })
        }
      })
      .catch((loadError: unknown) => {
        if (!cancelled) {
          setState({
            status: 'error',
            message: loadError instanceof Error ? loadError.message : 'Could not load incidents',
          })
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  const incidents = useMemo(() => (state.status === 'ready' ? state.incidents : []), [state])
  const isLoading = state.status === 'loading'

  const visibleIncidents = useMemo(() => {
    const needle = query.trim().toLowerCase()

    const filtered = incidents.filter((incident) => {
      if (severityFilter && incident.severity !== severityFilter) {
        return false
      }

      if (statusFilter && incident.status !== statusFilter) {
        return false
      }

      if (!needle) {
        return true
      }

      return (
        incident.title.toLowerCase().includes(needle) ||
        incident.content.toLowerCase().includes(needle)
      )
    })

    return sortIncidents(filtered)
  }, [incidents, query, severityFilter, statusFilter])

  const openCount = incidents.filter((incident) => incident.status !== 'resolved').length

  return (
    <Box>
      <AskPanel />

      <CardlessToolbar
        total={incidents.length}
        shown={visibleIncidents.length}
        openCount={openCount}
        query={query}
        onQueryChange={setQuery}
        severityFilter={severityFilter}
        onSeverityFilterChange={setSeverityFilter}
        statusFilter={statusFilter}
        onStatusFilterChange={setStatusFilter}
      />

      {state.status === 'error' ? (
        <Box
          p="4"
          borderRadius="md"
          bg="red.50"
          color="red.700"
          borderWidth="1px"
          borderColor="red.200"
          fontSize="sm"
        >
          {state.message}
        </Box>
      ) : null}

      {isLoading ? (
        <SimpleGrid columns={{ base: 1, sm: 2, lg: 3 }} gap="4">
          {[0, 1, 2, 3, 4, 5].map((key) => (
            <Box key={key} p="4" borderWidth="1px" borderColor="gray.200" borderRadius="lg" bg="white">
              <Skeleton height="20px" width="80px" borderRadius="full" mb="3" />
              <Skeleton height="20px" width="70%" mb="3" />
              <SkeletonText noOfLines={3} gap="2" />
            </Box>
          ))}
        </SimpleGrid>
      ) : null}

      {!isLoading && state.status !== 'error' && visibleIncidents.length === 0 ? (
        <EmptyState.Root height="auto" py="14" borderWidth="1px" borderColor="gray.200" borderRadius="lg">
          <EmptyState.Content>
            <EmptyState.Indicator fontSize="2xl">🧯</EmptyState.Indicator>
            <EmptyState.Title fontSize="lg" fontWeight="semibold">
              {incidents.length === 0 ? 'No incidents logged yet' : 'Nothing matches these filters'}
            </EmptyState.Title>
            <EmptyState.Description color="gray.500" fontSize="sm">
              {incidents.length === 0
                ? 'Use “New incident” in the top right to write down what broke and how you fixed it.'
                : 'Try clearing the search box or the severity/status filters.'}
            </EmptyState.Description>
          </EmptyState.Content>
        </EmptyState.Root>
      ) : null}

      {!isLoading && visibleIncidents.length > 0 ? (
        <SimpleGrid columns={{ base: 1, sm: 2, lg: 3 }} gap="4">
          {visibleIncidents.map((incident) => (
            <IncidentCard key={incident.id} incident={incident} />
          ))}
        </SimpleGrid>
      ) : null}
    </Box>
  )
}

interface ToolbarProps {
  total: number
  shown: number
  openCount: number
  query: string
  onQueryChange: (value: string) => void
  severityFilter: Severity | null
  onSeverityFilterChange: (value: Severity | null) => void
  statusFilter: IncidentStatus | null
  onStatusFilterChange: (value: IncidentStatus | null) => void
}

const CardlessToolbar: React.FC<ToolbarProps> = ({
  total,
  shown,
  openCount,
  query,
  onQueryChange,
  severityFilter,
  onSeverityFilterChange,
  statusFilter,
  onStatusFilterChange,
}) => (
  <Box
    bg="white"
    borderWidth="1px"
    borderColor="gray.200"
    borderRadius="lg"
    p="4"
    mb="4"
    display="flex"
    flexDir={{ base: 'column', lg: 'row' }}
    gap="4"
    alignItems={{ lg: 'center' }}
  >
    <Input
      value={query}
      onChange={(event) => onQueryChange(event.target.value)}
      placeholder="Filter by title or text…"
      flex="1"
      minW="220px"
    />

    <HStack gap="4" wrap="wrap" justify={{ base: 'flex-start', lg: 'flex-end' }}>
      <HStack gap="1.5">
        {SEVERITIES.map((severity) => (
          <Button
            key={severity}
            size="sm"
            variant={severityFilter === severity ? 'solid' : 'outline'}
            colorPalette="gray"
            onClick={() => onSeverityFilterChange(severityFilter === severity ? null : severity)}
          >
            {SEVERITY_LABEL[severity]}
          </Button>
        ))}
      </HStack>

      <HStack gap="1.5">
        {STATUSES.map((status) => (
          <Button
            key={status}
            size="sm"
            variant={statusFilter === status ? 'solid' : 'outline'}
            colorPalette="gray"
            onClick={() => onStatusFilterChange(statusFilter === status ? null : status)}
          >
            {STATUS_LABEL[status]}
          </Button>
        ))}
      </HStack>

      <Text fontSize="xs" color="gray.500" whiteSpace="nowrap">
        {shown} of {total} incidents · {openCount} unresolved
      </Text>
    </HStack>
  </Box>
)
