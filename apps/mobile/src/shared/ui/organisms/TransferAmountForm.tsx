import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import {
  backToSearch,
  goToConfirm,
  selectRecipientUsername,
  setAmountInput,
  setDescription,
} from '@/features/transfer-form';
import { selectAmountInput, selectDescription } from '@/features/transfer-form';
import {
  beginTransferAttempt,
  resetSubmission,
  selectHasActiveAttempt,
  selectLockedAmountInput,
  selectLockedDescription,
  selectSubmissionStatus,
} from '@/features/transfer-submission';
import { isValidAmountInput } from '@/shared/lib/money';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import { AppText, Button } from '@/shared/ui/atoms';
import { FormField } from '@/shared/ui/molecules';

/**
 * Transfer amount and memo form organism.
 *
 * @returns Transfer form view.
 */
export function TransferAmountForm() {
  const dispatch = useAppDispatch();
  const recipient = useAppSelector(selectRecipientUsername);
  const amountInput = useAppSelector(selectAmountInput);
  const description = useAppSelector(selectDescription);
  const hasActiveAttempt = useAppSelector((state) =>
    selectHasActiveAttempt(state.transferSubmission),
  );
  const lockedAmount = useAppSelector((state) => selectLockedAmountInput(state.transferSubmission));
  const lockedDescription = useAppSelector((state) =>
    selectLockedDescription(state.transferSubmission),
  );
  const submissionStatus = useAppSelector((state) =>
    selectSubmissionStatus(state.transferSubmission),
  );
  const displayAmount = hasActiveAttempt && lockedAmount !== null ? lockedAmount : amountInput;
  const displayDescription =
    hasActiveAttempt && lockedDescription !== null ? lockedDescription : description;
  const valid = isValidAmountInput(displayAmount);
  const lockEdits = hasActiveAttempt && submissionStatus !== 'succeeded';

  return (
    <View style={styles.container} testID="transfer-amount-step">
      <AppText variant="subtitle">Send to @{recipient}</AppText>

      {lockEdits ? (
        <AppText variant="caption" muted accessibilityLabel="Transfer amount locked">
          Amount is locked for a safe retry. Cancel to start a new transfer with a new key.
        </AppText>
      ) : null}

      <FormField
        label="Amount (USD)"
        value={displayAmount}
        onChangeText={(text) => dispatch(setAmountInput(text))}
        keyboardType="decimal-pad"
        placeholder="0.00"
        accessibilityLabel="Transfer amount"
        testID="transfer-amount"
        editable={!lockEdits}
        {...(displayAmount && !valid
          ? {
              hint: 'Enter a valid amount greater than zero',
              hintIsError: true,
              hasError: true,
            }
          : {})}
      />

      <FormField
        label="Memo (optional)"
        value={displayDescription}
        onChangeText={(text) => dispatch(setDescription(text))}
        placeholder="What's this for?"
        accessibilityLabel="Transfer memo"
        testID="transfer-memo"
        editable={!lockEdits}
      />

      <View style={styles.actions}>
        {lockEdits ? (
          <Button
            label="Cancel attempt"
            variant="ghost"
            onPress={() => dispatch(resetSubmission())}
            accessibilityLabel="Cancel transfer attempt"
            testID="transfer-cancel-attempt"
          />
        ) : (
          <Button label="Back" variant="ghost" onPress={() => dispatch(backToSearch())} />
        )}
        <Button
          label="Review"
          testID="transfer-review"
          onPress={() => {
            dispatch(
              beginTransferAttempt({
                amountInput: displayAmount,
                description: displayDescription,
              }),
            );
            dispatch(goToConfirm());
          }}
          disabled={!valid}
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
  actions: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: spacing.sm,
  },
  primary: {
    flex: 1,
    marginLeft: spacing.md,
  },
});
