import { createToaster } from '@chakra-ui/react'

export const toaster = createToaster({
  placement: 'bottom-end',
  overlap: true,
  gap: 3,
  duration: 4000,
})
