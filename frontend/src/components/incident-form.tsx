import { Box, Button, Field, HStack, Input, NativeSelect } from '@chakra-ui/react'
import { useForm } from 'react-hook-form'
import {
  SEVERITIES,
  SEVERITY_LABEL,
  STATUSES,
  STATUS_LABEL,
  NEW_INCIDENT_TEMPLATE,
  type Incident,
  type IncidentStatus,
  type Severity,
} from '../domain/incident'
import { AutoResizeTextarea } from './auto-resize-textarea'

export interface IncidentFormValues {
  title: string
  severity: Severity
  status: IncidentStatus
  content: string
}

interface Props {
  initial?: Incident
  submitLabel: string
  onSubmit: (values: IncidentFormValues) => void | Promise<void>
  onCancel?: () => void
  autoFocusTitle?: boolean
}

const EMPTY: IncidentFormValues = {
  title: '',
  severity: 'medium',
  status: 'open',
  content: NEW_INCIDENT_TEMPLATE,
}

export const IncidentForm: React.FC<Props> = ({
  initial,
  submitLabel,
  onSubmit,
  onCancel,
  autoFocusTitle = false,
}) => {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<IncidentFormValues>({
    values: initial
      ? {
          title: initial.title,
          severity: initial.severity,
          status: initial.status,
          content: initial.content,
        }
      : EMPTY,
  })

  return (
    <Box
      as="form"
      onSubmit={handleSubmit(async (values) => {
        await onSubmit({ ...values, title: values.title.trim() })
      })}
      display="flex"
      flexDirection="column"
      gap="4"
    >
      <Field.Root required invalid={Boolean(errors.title)}>
        <Field.Label fontSize="sm" fontWeight="medium" color="gray.700">
          Title
        </Field.Label>
        <Input
          placeholder="e.g. Checkout queue backed up after deploy"
          autoFocus={autoFocusTitle}
          {...register('title', { required: 'A short title is required' })}
        />
        {errors.title ? <Field.ErrorText>{errors.title.message}</Field.ErrorText> : null}
      </Field.Root>

      <HStack gap="3" wrap="wrap">
        <Field.Root flex="1" minW="180px">
          <Field.Label fontSize="sm" fontWeight="medium" color="gray.700">
            Severity
          </Field.Label>
          <NativeSelect.Root>
            <NativeSelect.Field {...register('severity')}>
              {SEVERITIES.map((severity) => (
                <option key={severity} value={severity}>
                  {SEVERITY_LABEL[severity]}
                </option>
              ))}
            </NativeSelect.Field>
            <NativeSelect.Indicator />
          </NativeSelect.Root>
        </Field.Root>

        <Field.Root flex="1" minW="180px">
          <Field.Label fontSize="sm" fontWeight="medium" color="gray.700">
            Status
          </Field.Label>
          <NativeSelect.Root>
            <NativeSelect.Field {...register('status')}>
              {STATUSES.map((status) => (
                <option key={status} value={status}>
                  {STATUS_LABEL[status]}
                </option>
              ))}
            </NativeSelect.Field>
            <NativeSelect.Indicator />
          </NativeSelect.Root>
        </Field.Root>
      </HStack>

      <Field.Root>
        <Field.Label fontSize="sm" fontWeight="medium" color="gray.700">
          Details
        </Field.Label>
        <Field.HelperText fontSize="xs" color="gray.500">
          Markdown supported. Describe the incident, the root cause and how it was solved.
        </Field.HelperText>
        <AutoResizeTextarea
          placeholder="What happened, how we found it, and how we fixed it…"
          minRows={8}
          mt="1"
          {...register('content')}
        />
      </Field.Root>

      <HStack gap="2" justify="flex-end" pt="1">
        {onCancel ? (
          <Button variant="ghost" colorPalette="gray" onClick={onCancel} disabled={isSubmitting}>
            Cancel
          </Button>
        ) : null}
        <Button type="submit" colorPalette="blue" loading={isSubmitting}>
          {submitLabel}
        </Button>
      </HStack>
    </Box>
  )
}
