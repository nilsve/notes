import { useCallback, useEffect, useState } from 'react'
import { Box, Button, Card, Dialog, HStack, Separator, Spinner, Text } from '@chakra-ui/react'
import { Link as RouterLink, useNavigate, useParams } from 'react-router-dom'
import { deleteIncident, getIncident, updateIncident } from '../../api/incidents'
import { SeverityBadge, StatusBadge } from '../../components/badges'
import { IncidentForm, type IncidentFormValues } from '../../components/incident-form'
import { Markdown } from '../../components/markdown'
import { toaster } from '../../lib/toaster'
import type { Incident } from '../../domain/incident'

export const IncidentDetailPage: React.FC = () => {
  const { incidentId } = useParams<{ incidentId: string }>()
  const navigate = useNavigate()

  const [incident, setIncident] = useState<Incident | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [isEditing, setEditing] = useState(false)
  const [isDeleteOpen, setDeleteOpen] = useState(false)

  useEffect(() => {
    if (!incidentId) {
      return
    }

    let cancelled = false

    getIncident(incidentId)
      .then((result) => {
        if (cancelled) {
          return
        }

        setIncident(result)
        setLoadError(result ? null : 'Incident not found')
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setLoadError(error instanceof Error ? error.message : 'Could not load this incident')
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false)
        }
      })

    return () => {
      cancelled = true
    }
  }, [incidentId])

  const handleUpdate = useCallback(
    async (values: IncidentFormValues) => {
      if (!incident) {
        return
      }

      try {
        const updated = await updateIncident({ ...incident, ...values })
        setIncident(updated)
        setEditing(false)
        toaster.create({ title: 'Incident updated', type: 'success' })
      } catch (updateError) {
        toaster.create({
          title: 'Could not update the incident',
          description: updateError instanceof Error ? updateError.message : 'Unknown error',
          type: 'error',
          duration: 8000,
        })
      }
    },
    [incident],
  )

  const handleDelete = useCallback(async () => {
    if (!incident) {
      return
    }

    try {
      await deleteIncident(incident.id)
      toaster.create({ title: 'Incident deleted', description: incident.title, type: 'success' })
      navigate('/', { replace: true })
    } catch (deleteError) {
      toaster.create({
        title: 'Could not delete the incident',
        description: deleteError instanceof Error ? deleteError.message : 'Unknown error',
        type: 'error',
        duration: 8000,
      })
    } finally {
      setDeleteOpen(false)
    }
  }, [incident, navigate])

  if (!incidentId) {
    return (
      <Box
        p="4"
        borderRadius="md"
        bg="red.50"
        color="red.700"
        borderWidth="1px"
        borderColor="red.200"
        fontSize="sm"
      >
        This incident does not have a valid id.
        <Box mt="3">
          <RouterLink to="/" style={{ textDecoration: 'underline' }}>
            Back to the incident log
          </RouterLink>
        </Box>
      </Box>
    )
  }

  if (isLoading) {
    return (
      <HStack gap="2" color="gray.500">
        <Spinner size="sm" />
        Loading incident…
      </HStack>
    )
  }

  if (loadError || !incident) {
    return (
      <Box
        p="4"
        borderRadius="md"
        bg="red.50"
        color="red.700"
        borderWidth="1px"
        borderColor="red.200"
        fontSize="sm"
      >
        {loadError ?? 'Incident not found'}
        <Box mt="3">
          <RouterLink to="/" style={{ textDecoration: 'underline' }}>
            Back to the incident log
          </RouterLink>
        </Box>
      </Box>
    )
  }

  return (
    <Box>
      <Box mb="4">
        <RouterLink
          to="/"
          style={{ textDecoration: 'none', color: 'var(--chakra-colors-gray-600)', fontSize: '14px' }}
        >
          ← All incidents
        </RouterLink>
      </Box>

      <Card.Root variant="outline" bg="white">
        <Card.Body gap="4" p={{ base: '4', md: '6' }}>
          {isEditing ? (
            <>
              <Text fontSize="xs" fontWeight="semibold" color="gray.500" textTransform="uppercase">
                Editing incident
              </Text>
              <IncidentForm
                initial={incident}
                submitLabel="Save changes"
                onSubmit={handleUpdate}
                onCancel={() => setEditing(false)}
              />
            </>
          ) : (
            <>
              <HStack gap="3" align="flex-start" justify="space-between" wrap="wrap">
                <Box flex="1" minW="240px">
                  <Text fontSize="xl" fontWeight="bold" lineHeight="1.3" mb="2">
                    {incident.title}
                  </Text>
                  <HStack gap="2" wrap="wrap">
                    <SeverityBadge severity={incident.severity} />
                    <StatusBadge status={incident.status} />
                  </HStack>
                </Box>

                <HStack gap="2">
                  <Button variant="outline" colorPalette="gray" onClick={() => setEditing(true)}>
                    Edit
                  </Button>
                  <Button variant="ghost" colorPalette="red" onClick={() => setDeleteOpen(true)}>
                    Delete
                  </Button>
                </HStack>
              </HStack>

              <Separator />

              {incident.content ? (
                <Markdown>{incident.content}</Markdown>
              ) : (
                <Text color="gray.500" fontStyle="italic">
                  No details recorded yet. Hit “Edit” and add what happened, the root cause and the fix.
                </Text>
              )}
            </>
          )}
        </Card.Body>
      </Card.Root>

      <Dialog.Root open={isDeleteOpen} onOpenChange={(details) => setDeleteOpen(details.open)}>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content maxW="md">
            <Dialog.Header>
              <Dialog.Title fontSize="lg" fontWeight="bold">
                Delete this incident?
              </Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <Text color="gray.600" fontSize="sm">
                “{incident.title}” will be removed from the log and from the semantic index. This cannot
                be undone.
              </Text>
            </Dialog.Body>
            <Dialog.Footer>
              <HStack gap="2" justify="flex-end" w="100%">
                <Dialog.ActionTrigger asChild>
                  <Button variant="outline" colorPalette="gray">
                    Keep it
                  </Button>
                </Dialog.ActionTrigger>
                <Button colorPalette="red" onClick={handleDelete}>
                  Delete incident
                </Button>
              </HStack>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Positioner>
      </Dialog.Root>
    </Box>
  )
}
