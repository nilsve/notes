import { forwardRef } from 'react'
import { Textarea, type TextareaProps } from '@chakra-ui/react'
import ResizeTextarea from 'react-textarea-autosize'

export interface AutoResizeTextareaProps extends Omit<TextareaProps, 'minRows'> {
  minRows?: number
  maxRows?: number
}

/** Textarea that grows with its content (Chakra styling around react-textarea-autosize). */
export const AutoResizeTextarea = forwardRef<HTMLTextAreaElement, AutoResizeTextareaProps>(
  ({ minRows = 4, maxRows, ...props }, ref) => (
    <Textarea asChild overflow="hidden" w="100%" resize="none" {...props}>
      <ResizeTextarea ref={ref} minRows={minRows} maxRows={maxRows} />
    </Textarea>
  ),
)

AutoResizeTextarea.displayName = 'AutoResizeTextarea'
