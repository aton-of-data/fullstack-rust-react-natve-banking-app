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
  const valid = isValidAmountInput(amountInput);

  return (
    <View style={styles.container}>
      <AppText variant="subtitle">Send to @{recipient}</AppText>

      <FormField
        label="Amount (USD)"
        value={amountInput}
        onChangeText={(text) => dispatch(setAmountInput(text))}
        keyboardType="decimal-pad"
        placeholder="0.00"
        accessibilityLabel="Transfer amount"
        {...(amountInput && !valid
          ? {
              hint: 'Enter a valid amount greater than zero',
              hintIsError: true,
              hasError: true,
            }
          : {})}
      />

      <FormField
        label="Memo (optional)"
        value={description}
        onChangeText={(text) => dispatch(setDescription(text))}
        placeholder="What's this for?"
        accessibilityLabel="Transfer memo"
      />

      <View style={styles.actions}>
        <Button label="Back" variant="ghost" onPress={() => dispatch(backToSearch())} />
        <Button
          label="Review"
          onPress={() => dispatch(goToConfirm())}
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
