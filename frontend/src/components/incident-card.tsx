import { Card, HStack, Text } from '@chakra-ui/react'
import { Link as RouterLink } from 'react-router-dom'
import { SeverityBadge, StatusBadge } from './badges'
import { excerpt, formatRelative, type Incident } from '../domain/incident'

interface Props {
  incident: Incident
}

export const IncidentCard: React.FC<Props> = ({ incident }) => (
  <Card.Root
    className="incident-enter"
    asChild
    variant="outline"
    height="full"
    transition="border-color 120ms ease, transform 120ms ease, box-shadow 120ms ease"
    _hover={{ borderColor: 'blue.400', transform: 'translateY(-2px)', boxShadow: 'md' }}
  >
    <RouterLink to={`/${incident.id}`} style={{ textDecoration: 'none' }}>
      <Card.Body gap="3" p="4">
        <HStack gap="2" wrap="wrap">
          <SeverityBadge severity={incident.severity} />
          <StatusBadge status={incident.status} />
        </HStack>

        <Text fontSize="md" fontWeight="semibold" lineHeight="1.4" color="gray.900">
          {incident.title}
        </Text>

        {incident.content ? (
          <Text fontSize="sm" color="gray.600" lineClamp={3}>
            {excerpt(incident.content, 220)}
          </Text>
        ) : (
          <Text fontSize="sm" color="gray.400" fontStyle="italic">
            No description yet
          </Text>
        )}

        <HStack gap="1" mt="auto" fontSize="xs" color="gray.400">
          <span>Logged {formatRelative(incident.createdAt)}</span>
        </HStack>
      </Card.Body>
    </RouterLink>
  </Card.Root>
)
