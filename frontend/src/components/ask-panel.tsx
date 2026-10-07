import { useCallback, useState, type FormEvent } from 'react'
import { Box, Button, Card, HStack, Input, Spinner, Text } from '@chakra-ui/react'
import { askAboutIncidents } from '../api/incidents'
import { Markdown } from './markdown'

/**
 * Semantic search: asks the AI over the whole incident history instead of
 * matching on keywords.
 */
export const AskPanel: React.FC = () => {
  const [question, setQuestion] = useState('')
  const [answer, setAnswer] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isAsking, setIsAsking] = useState(false)

  const handleAsk = useCallback(async () => {
    const trimmed = question.trim()

    if (!trimmed || isAsking) {
      return
    }

    setIsAsking(true)
    setAnswer(null)
    setError(null)

    try {
      setAnswer(await askAboutIncidents(trimmed))
    } catch (askError) {
      setError(askError instanceof Error ? askError.message : 'Something went wrong asking the AI')
    } finally {
      setIsAsking(false)
    }
  }, [question, isAsking])

  const handleSubmit = useCallback(
    (event: FormEvent) => {
      event.preventDefault()
      void handleAsk()
    },
    [handleAsk],
  )

  return (
    <Card.Root variant="outline" mb="6" bg="white">
      <Card.Body gap="3" p="5">
        <HStack gap="2" mb="1">
          <Box
            w="7"
            h="7"
            borderRadius="full"
            bg="blue.50"
            color="blue.600"
            display="grid"
            placeItems="center"
            fontSize="sm"
            fontWeight="bold"
          >
            ?
          </Box>
          <Box>
            <Text fontSize="md" fontWeight="semibold" color="gray.900">
              Ask the incident log
            </Text>
            <Text fontSize="xs" color="gray.500">
              Semantic search over every incident we have logged
            </Text>
          </Box>
        </HStack>

        <HStack as="form" onSubmit={handleSubmit} gap="2" align="stretch">
          <Input
            value={question}
            onChange={(event) => setQuestion(event.target.value)}
            placeholder="Have we seen a 500 on checkout before?"
            size="lg"
          />
          <Button type="submit" colorPalette="blue" size="lg" loading={isAsking} minW="7rem">
            {isAsking ? 'Searching' : 'Ask'}
          </Button>
        </HStack>

        {isAsking ? (
          <HStack gap="2" color="gray.500" fontSize="sm">
            <Spinner size="sm" />
            Searching similar incidents…
          </HStack>
        ) : null}

        {error ? (
          <Box
            p="3"
            borderRadius="md"
            bg="red.50"
            color="red.700"
            fontSize="sm"
            borderWidth="1px"
            borderColor="red.200"
          >
            {error}
          </Box>
        ) : null}

        {answer ? (
          <Box p="4" borderRadius="md" bg="gray.50" borderWidth="1px" borderColor="gray.200">
            <Markdown>{answer}</Markdown>
          </Box>
        ) : null}
      </Card.Body>
    </Card.Root>
  )
}
