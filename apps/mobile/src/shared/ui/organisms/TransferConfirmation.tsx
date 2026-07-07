import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import {
  backToForm,
  selectRecipientUsername,
  selectTransferSubmitting,
  submitFailed,
  submitStarted,
  submitSucceeded,
} from '@/features/transfer-form';
import { selectAmountInput, selectDescription } from '@/features/transfer-form';
import {
  resetSubmission,
  selectIdempotencyKey,
  selectSubmissionErrorMessage,
  selectSubmissionRetryable,
  submissionFailed,
  submissionStarted,
  submissionSucceeded,
} from '@/features/transfer-submission';
import { majorToMinorUnits, formatMoney } from '@/shared/lib/money';
import { mapTransferError } from '@/shared/lib/transferErrors';
import { useCreateTransferMutation } from '@/services';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import { AppText, Button, Card, ErrorBanner } from '@/shared/ui/atoms';

/**
 * Transfer confirmation and submission organism.
 *
 * @returns Confirmation view.
 */
export function TransferConfirmation() {
  const dispatch = useAppDispatch();
  const recipient = useAppSelector(selectRecipientUsername);
  const amountInput = useAppSelector(selectAmountInput);
  const description = useAppSelector(selectDescription);
  const submitting = useAppSelector(selectTransferSubmitting);
  const idempotencyKey = useAppSelector((state) => selectIdempotencyKey(state.transferSubmission));
  const submitError = useAppSelector((state) =>
    selectSubmissionErrorMessage(state.transferSubmission),
  );
  const retryable = useAppSelector((state) => selectSubmissionRetryable(state.transferSubmission));
  const [createTransfer] = useCreateTransferMutation();

  const minor = majorToMinorUnits(amountInput);
  const formatted = formatMoney(minor, 'USD');

  const handleConfirm = async (): Promise<void> => {
    if (!recipient || !idempotencyKey) {
      return;
    }
    dispatch(submissionStarted());
    dispatch(submitStarted());
    try {
      const request = {
        recipient_username: recipient,
        amount_minor: minor,
        currency: 'USD',
        ...(description ? { description } : {}),
      };
      const result = await createTransfer({ body: request, idempotencyKey }).unwrap();
      dispatch(submissionSucceeded(result.transfer_id));
      dispatch(submitSucceeded());
      dispatch(resetSubmission());
    } catch (error) {
      const mapped = mapTransferError(error);
      dispatch(
        submissionFailed({
          code: mapped.code,
          message: mapped.message,
          retryable: mapped.retryable,
        }),
      );
      dispatch(submitFailed(mapped.message));
    }
  };

  return (
    <View style={styles.container}>
      <AppText variant="subtitle">Confirm transfer</AppText>

      {submitError ? <ErrorBanner message={submitError} /> : null}
      {retryable ? (
        <AppText variant="label" muted accessibilityLabel="Retry guidance">
          You can retry safely — the same idempotency key will be reused.
        </AppText>
      ) : null}

      <Card>
        <AppText variant="label" muted>
          Recipient
        </AppText>
        <AppText variant="body">@{recipient}</AppText>

        <AppText variant="label" muted style={styles.field}>
          Amount
        </AppText>
        <AppText variant="title">{formatted}</AppText>

        {description ? (
          <>
            <AppText variant="label" muted style={styles.field}>
              Memo
            </AppText>
            <AppText variant="body">{description}</AppText>
          </>
        ) : null}
      </Card>

      <View style={styles.actions}>
        <Button label="Back" variant="ghost" onPress={() => dispatch(backToForm())} />
        <Button
          label={retryable ? 'Retry Transfer' : 'Send Money'}
          onPress={() => void handleConfirm()}
          loading={submitting}
          disabled={submitting || !idempotencyKey}
          accessibilityLabel={retryable ? 'Retry transfer' : 'Send money'}
          style={styles.primary}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    gap: spacing.md,
  },
  field: {
    marginTop: spacing.md,
  },
  actions: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  primary: {
    flex: 1,
    marginLeft: spacing.md,
  },
});
