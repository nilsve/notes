import { useCallback, useState } from 'react'
import { Box, Button, Dialog, HStack, Text } from '@chakra-ui/react'
import { Outlet, useNavigate } from 'react-router-dom'
import { createIncident } from '../api/incidents'
import { IncidentForm, type IncidentFormValues } from './incident-form'
import { toaster } from '../lib/toaster'

export const AppShell: React.FC = () => {
  const [isCreateOpen, setCreateOpen] = useState(false)
  const navigate = useNavigate()

  const handleCreate = useCallback(
    async (values: IncidentFormValues) => {
      try {
        const incident = await createIncident(values)
        setCreateOpen(false)
        toaster.create({
          title: 'Incident logged',
          description: incident.title,
          type: 'success',
        })
        navigate(`/${incident.id}`)
      } catch (error) {
        toaster.create({
          title: 'Could not save the incident',
          description: error instanceof Error ? error.message : 'Unknown error',
          type: 'error',
          duration: 8000,
        })
      }
    },
    [navigate],
  )

  return (
    <Box minH="100vh" bg="gray.50" color="gray.900">
      <Box
        as="header"
        position="sticky"
        top="0"
        zIndex="10"
        bg="white"
        borderBottomWidth="1px"
        borderColor="gray.200"
      >
        <Box maxW="1180px" mx="auto" px={{ base: '4', md: '6' }} py="3">
          <HStack gap="3" justify="space-between">
            <HStack
              gap="3"
              cursor="pointer"
              onClick={() => navigate('/')}
              userSelect="none"
              title="Incident log"
            >
              <Box
                w="9"
                h="9"
                borderRadius="lg"
                bg="blue.600"
                color="white"
                display="grid"
                placeItems="center"
                fontWeight="bold"
                fontSize="sm"
              >
                IL
              </Box>
              <Box display={{ base: 'none', sm: 'block' }}>
                <Text fontSize="md" fontWeight="bold" lineHeight="1.2">
                  Incident Log
                </Text>
                <Text fontSize="xs" color="gray.500">
                  What broke, and how we fixed it
                </Text>
              </Box>
            </HStack>

            <Button colorPalette="blue" onClick={() => setCreateOpen(true)}>
              New incident
            </Button>
          </HStack>
        </Box>
      </Box>

      <Box as="main" maxW="1180px" mx="auto" px={{ base: '4', md: '6' }} py="6">
        <Outlet />
      </Box>

      <Dialog.Root
        size="xl"
        open={isCreateOpen}
        onOpenChange={(details) => setCreateOpen(details.open)}
      >
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.Header>
              <Dialog.Title fontSize="lg" fontWeight="bold">
                Log a new incident
              </Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <IncidentForm
                submitLabel="Log incident"
                onSubmit={handleCreate}
                onCancel={() => setCreateOpen(false)}
                autoFocusTitle
              />
            </Dialog.Body>
          </Dialog.Content>
        </Dialog.Positioner>
      </Dialog.Root>
    </Box>
  )
}
