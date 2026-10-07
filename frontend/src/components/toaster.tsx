import { HStack, Spinner, Toast, Toaster as ChakraToaster } from '@chakra-ui/react'
import { toaster } from '../lib/toaster'

/** Renders the toast queue created in `lib/toaster`. Mount this once, near the app root. */
export const AppToaster: React.FC = () => (
  <ChakraToaster toaster={toaster} offset="1.5rem" width={{ base: 'full', md: 'sm' }}>
    {(toast) => (
      <Toast.Root width={{ md: 'sm' }}>
        {toast.type === 'loading' ? <Spinner size="sm" color="blue.solid" /> : <Toast.Indicator />}
        <HStack gap="1" flex="1" align="start">
          {toast.title ? <Toast.Title>{toast.title}</Toast.Title> : null}
          {toast.description ? <Toast.Description>{toast.description}</Toast.Description> : null}
        </HStack>
        {toast.action ? <Toast.ActionTrigger>{toast.action.label}</Toast.ActionTrigger> : null}
        <Toast.CloseTrigger />
      </Toast.Root>
    )}
  </ChakraToaster>
)
