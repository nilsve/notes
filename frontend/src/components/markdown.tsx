import { Box, Code, Link, Text } from '@chakra-ui/react'
import ReactMarkdown, { type Components } from 'react-markdown'

const components: Components = {
  h1: ({ children }) => (
    <Text as="h1" fontSize="xl" fontWeight="bold" mt="5" mb="2">
      {children}
    </Text>
  ),
  h2: ({ children }) => (
    <Text as="h2" fontSize="lg" fontWeight="bold" mt="5" mb="2">
      {children}
    </Text>
  ),
  h3: ({ children }) => (
    <Text as="h3" fontSize="md" fontWeight="semibold" mt="4" mb="1">
      {children}
    </Text>
  ),
  p: ({ children }) => (
    <Text as="p" mb="2" lineHeight="1.7">
      {children}
    </Text>
  ),
  ul: ({ children }) => (
    <Box as="ul" pl="6" mb="2" listStyleType="disc">
      {children}
    </Box>
  ),
  ol: ({ children }) => (
    <Box as="ol" pl="6" mb="2" listStyleType="decimal">
      {children}
    </Box>
  ),
  li: ({ children }) => (
    <Text as="li" mb="1" lineHeight="1.6">
      {children}
    </Text>
  ),
  a: ({ children, href }) => (
    <Link href={href} color="blue.600" textDecoration="underline" target="_blank" rel="noreferrer">
      {children}
    </Link>
  ),
  strong: ({ children }) => (
    <Text as="strong" fontWeight="semibold">
      {children}
    </Text>
  ),
  code: ({ children }) => (
    <Code px="1" py="0.5" borderRadius="sm" fontSize="0.9em">
      {children}
    </Code>
  ),
  pre: ({ children }) => (
    <Box
      as="pre"
      p="3"
      mb="3"
      borderRadius="md"
      bg="gray.900"
      color="gray.50"
      overflowX="auto"
      fontSize="sm"
    >
      {children}
    </Box>
  ),
  blockquote: ({ children }) => (
    <Box
      as="blockquote"
      borderLeftWidth="3px"
      borderLeftColor="gray.300"
      pl="3"
      mb="2"
      color="gray.600"
    >
      {children}
    </Box>
  ),
  hr: () => <Box as="hr" my="4" />,
  table: ({ children }) => (
    <Box as="table" w="full" mb="3" borderWidth="1px" borderColor="gray.200" borderRadius="md">
      {children}
    </Box>
  ),
  th: ({ children }) => (
    <Box as="th" px="2" py="1" textAlign="left" bg="gray.50" fontWeight="semibold">
      {children}
    </Box>
  ),
  td: ({ children }) => (
    <Box as="td" px="2" py="1" borderTopWidth="1px" borderColor="gray.100">
      {children}
    </Box>
  ),
}

interface Props {
  children: string
}

export const Markdown: React.FC<Props> = ({ children }) => (
  <Box className="markdown">
    <ReactMarkdown components={components}>{children}</ReactMarkdown>
  </Box>
)
