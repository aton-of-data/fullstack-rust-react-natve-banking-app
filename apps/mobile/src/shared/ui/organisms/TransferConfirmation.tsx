import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import {
  backToForm,
  selectRecipientUsername,
  selectTransferSubmitting,
  submitFailed,
  submitSettled,
  submitStarted,
  submitSucceeded,
} from '@/features/transfer-form';
import { selectAmountInput, selectDescription } from '@/features/transfer-form';
import { selectUsername } from '@/features/auth';
import {
  resetSubmission,
  selectIdempotencyKey,
  selectLastTransferId,
  selectLockedAmountInput,
  selectLockedDescription,
  selectSubmissionErrorCode,
  selectSubmissionErrorMessage,
  selectSubmissionRetryable,
  selectSubmissionStatus,
  submissionFailed,
  submissionStarted,
  submissionSucceeded,
} from '@/features/transfer-submission';
import { majorToMinorUnits, formatMoney } from '@/shared/lib/money';
import { mapTransferError } from '@/shared/lib/transferErrors';
import { useCreateTransferMutation } from '@/services';
import { useAppDispatch, useAppSelector, useAppStore } from '@/store/hooks';
import { AppText, Button, Card, ErrorBanner } from '@/shared/ui/atoms';

/**
 * Transfer confirmation and submission organism.
 *
 * @returns Confirmation view.
 */
export function TransferConfirmation() {
  const dispatch = useAppDispatch();
  const store = useAppStore();
  const recipient = useAppSelector(selectRecipientUsername);
  const formAmount = useAppSelector(selectAmountInput);
  const formDescription = useAppSelector(selectDescription);
  const lockedAmount = useAppSelector((state) => selectLockedAmountInput(state.transferSubmission));
  const lockedDescription = useAppSelector((state) =>
    selectLockedDescription(state.transferSubmission),
  );
  const amountInput = lockedAmount ?? formAmount;
  const description = lockedDescription ?? formDescription;
  const submitting = useAppSelector(selectTransferSubmitting);
  const sourceUsername = useAppSelector(selectUsername);
  const idempotencyKey = useAppSelector((state) => selectIdempotencyKey(state.transferSubmission));
  const submitError = useAppSelector((state) =>
    selectSubmissionErrorMessage(state.transferSubmission),
  );
  const errorCode = useAppSelector((state) => selectSubmissionErrorCode(state.transferSubmission));
  const retryable = useAppSelector((state) => selectSubmissionRetryable(state.transferSubmission));
  const status = useAppSelector((state) => selectSubmissionStatus(state.transferSubmission));
  const lastTransferId = useAppSelector((state) => selectLastTransferId(state.transferSubmission));
  const [createTransfer] = useCreateTransferMutation();

  const minor = majorToMinorUnits(amountInput);
  const formatted = formatMoney(minor, 'USD');
  const isConflict = errorCode === 'IDEMPOTENCY_CONFLICT';
  const isSucceeded = status === 'succeeded';

  const handleConfirm = async (): Promise<void> => {
    // Read live store state to block double-tap races before re-render.
    const live = store.getState().transferSubmission;
    if (
      !recipient ||
      !live.idempotencyKey ||
      live.status === 'submitting' ||
      live.status === 'succeeded'
    ) {
      return;
    }
    const key = live.idempotencyKey;
    const lockedMinor = majorToMinorUnits(live.lockedAmountInput ?? amountInput);
    const lockedMemo = live.lockedDescription ?? description;
    dispatch(submissionStarted());
    dispatch(submitStarted());
    try {
      const request = {
        recipient_username: recipient,
        amount_minor: lockedMinor,
        currency: 'USD',
        ...(lockedMemo ? { description: lockedMemo } : {}),
      };
      const result = await createTransfer({ body: request, idempotencyKey: key }).unwrap();
      dispatch(submissionSucceeded(result.transfer_id));
      dispatch(submitSettled());
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

  if (isSucceeded) {
    return (
      <View style={styles.container} testID="transfer-success">
        <AppText variant="subtitle" accessibilityRole="header">
          Transfer sent
        </AppText>
        <Card>
          <AppText variant="body">
            {formatted} to @{recipient}
          </AppText>
          {lastTransferId ? (
            <AppText
              variant="caption"
              muted
              style={styles.field}
              accessibilityLabel={`Transfer reference ${lastTransferId}`}
            >
              Reference: {lastTransferId}
            </AppText>
          ) : null}
        </Card>
        <Button
          label="Done"
          testID="transfer-success-done"
          accessibilityLabel="Done with transfer"
          onPress={() => {
            dispatch(resetSubmission());
            dispatch(submitSucceeded());
          }}
        />
      </View>
    );
  }

  return (
    <View style={styles.container} testID="transfer-confirm-step">
      <AppText variant="subtitle">Confirm transfer</AppText>

      {submitError ? <ErrorBanner message={submitError} testID="transfer-error" /> : null}
      {retryable ? (
        <AppText variant="label" muted accessibilityLabel="Retry guidance">
          Outcome unknown. Retry reuses the same Idempotency-Key so you will not be charged twice.
        </AppText>
      ) : null}
      {isConflict ? (
        <AppText variant="label" muted accessibilityLabel="Idempotency conflict guidance">
          Start a new transfer to generate a new key. Do not reuse this attempt.
        </AppText>
      ) : null}

      <Card>
        {sourceUsername ? (
          <>
            <AppText variant="label" muted>
              From
            </AppText>
            <AppText variant="body">@{sourceUsername}</AppText>
          </>
        ) : null}

        <AppText variant="label" muted style={styles.field}>
          Recipient
        </AppText>
        <AppText variant="body" testID="transfer-confirm-recipient">
          @{recipient}
        </AppText>

        <AppText variant="label" muted style={styles.field}>
          Amount
        </AppText>
        <AppText variant="title" testID="transfer-confirm-amount">
          {formatted}
        </AppText>
        <AppText variant="caption" muted>
          USD
        </AppText>

        {description ? (
          <>
            <AppText variant="label" muted style={styles.field}>
              Memo
            </AppText>
            <AppText variant="body" testID="transfer-confirm-memo">
              {description}
            </AppText>
          </>
        ) : null}

        <AppText variant="caption" muted style={styles.field} accessibilityLabel="Finality warning">
          Money movement is final in this demo. Double-check the recipient and amount before
          sending.
        </AppText>
      </Card>

      <View style={styles.actions}>
        <Button
          label="Back"
          variant="ghost"
          testID="transfer-confirm-back"
          accessibilityLabel={
            retryable
              ? 'Go back — amount remains locked for safe retry'
              : 'Go back to edit transfer'
          }
          onPress={() => dispatch(backToForm())}
          disabled={submitting}
        />
        {isConflict ? (
          <Button
            label="Start new transfer"
            testID="transfer-start-new"
            onPress={() => {
              dispatch(resetSubmission());
              dispatch(backToForm());
            }}
            style={styles.primary}
          />
        ) : (
          <Button
            label={retryable ? 'Retry Transfer' : 'Send Money'}
            testID="transfer-confirm"
            onPress={() => void handleConfirm()}
            loading={submitting}
            disabled={submitting || !idempotencyKey}
            accessibilityLabel={retryable ? 'Retry transfer' : 'Send money'}
            accessibilityHint="Submits the transfer once. Disabled while in flight."
            style={styles.primary}
          />
        )}
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
