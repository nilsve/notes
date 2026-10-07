import { Badge } from '@chakra-ui/react'
import {
  SEVERITY_COLOR,
  SEVERITY_LABEL,
  STATUS_COLOR,
  STATUS_LABEL,
  type IncidentStatus,
  type Severity,
} from '../domain/incident'

type BadgeSize = 'xs' | 'sm' | 'md'

export const SeverityBadge: React.FC<{ severity: Severity; size?: BadgeSize }> = ({
  severity,
  size = 'sm',
}) => (
  <Badge
    colorPalette={SEVERITY_COLOR[severity]}
    variant="subtle"
    size={size}
    borderRadius="full"
    textTransform="uppercase"
    letterSpacing="0.05em"
  >
    {SEVERITY_LABEL[severity]}
  </Badge>
)

export const StatusBadge: React.FC<{ status: IncidentStatus; size?: BadgeSize }> = ({
  status,
  size = 'sm',
}) => (
  <Badge
    colorPalette={STATUS_COLOR[status]}
    variant="surface"
    size={size}
    borderRadius="full"
  >
    {STATUS_LABEL[status]}
  </Badge>
)
