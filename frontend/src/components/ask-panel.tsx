import { useCallback, useRef, useState, type FormEvent } from 'react'
import {
  Box,
  Button,
  Card,
  Collapsible,
  HStack,
  Input,
  Spinner,
  Text,
} from '@chakra-ui/react'
import { streamAboutIncidents } from '../api/incidents'
import { Markdown } from './markdown'

/**
 * Semantic search: asks the AI over the whole incident history instead of
 * matching on keywords. The answer is streamed in as the model generates it, and
 * the model's chain of thought is shown separately (collapsed by default).
 */
export const AskPanel: React.FC = () => {
  const [question, setQuestion] = useState('')
  const [answer, setAnswer] = useState<string | null>(null)
  const [reasoning, setReasoning] = useState('')
  const [thinkingOpen, setThinkingOpen] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isAsking, setIsAsking] = useState(false)
  const abortRef = useRef<AbortController | null>(null)

  const handleAsk = useCallback(async () => {
    const trimmed = question.trim()

    if (!trimmed || isAsking) {
      return
    }

    const controller = new AbortController()
    abortRef.current = controller

    setIsAsking(true)
    setAnswer(null)
    setReasoning('')
    setThinkingOpen(false)
    setError(null)

    let sawAnswer = false

    try {
      await streamAboutIncidents(trimmed, {
        signal: controller.signal,
        onReasoning: (delta) => {
          setReasoning((previous) => previous + delta)
        },
        onAnswer: (delta) => {
          sawAnswer = true
          setAnswer((previous) => (previous ?? '') + delta)
        },
      })

      if (!sawAnswer && !controller.signal.aborted) {
        setError('The model finished without producing an answer')
      }
    } catch (askError) {
      if (controller.signal.aborted) {
        return
      }

      setError(askError instanceof Error ? askError.message : 'Something went wrong asking the AI')
    } finally {
      setIsAsking(false)
      abortRef.current = null
    }
  }, [question, isAsking])

  const handleStop = useCallback(() => {
    abortRef.current?.abort()
  }, [])

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
          {isAsking ? (
            <Button
              type="button"
              variant="outline"
              colorPalette="gray"
              size="lg"
              minW="7rem"
              onClick={handleStop}
            >
              Stop
            </Button>
          ) : (
            <Button type="submit" colorPalette="blue" size="lg" minW="7rem">
              Ask
            </Button>
          )}
        </HStack>

        {isAsking && !reasoning && !answer ? (
          <HStack gap="2" color="gray.500" fontSize="sm">
            <Spinner size="sm" />
            Searching similar incidents…
          </HStack>
        ) : null}

        {reasoning ? (
          <Collapsible.Root open={thinkingOpen} onOpenChange={(details) => setThinkingOpen(details.open)}>
            <Collapsible.Trigger asChild>
              <Button
                variant="ghost"
                size="xs"
                colorPalette="gray"
                justifyContent="flex-start"
                px="0"
                _hover={{ bg: 'transparent', color: 'blue.600' }}
              >
                <HStack gap="2">
                  {isAsking && !answer ? <Spinner size="xs" /> : null}
                  <Text fontSize="xs" fontWeight="semibold" textTransform="uppercase">
                    {isAsking && !answer ? 'Thinking…' : `Thought for ${reasoning.length} chars`}
                  </Text>
                  <Text fontSize="xs" color="gray.400">
                    {thinkingOpen ? '▾' : '▸'}
                  </Text>
                </HStack>
              </Button>
            </Collapsible.Trigger>
            <Collapsible.Content>
              <Box
                mt="2"
                p="3"
                borderRadius="md"
                bg="gray.50"
                borderWidth="1px"
                borderColor="gray.200"
                fontSize="xs"
                color="gray.600"
                whiteSpace="pre-wrap"
                fontFamily="mono"
                maxH="16rem"
                overflowY="auto"
              >
                {reasoning}
              </Box>
            </Collapsible.Content>
          </Collapsible.Root>
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
