import { StyleSheet, View } from "react-native";

import { spacing } from "@/shared/theme";
import {
  backToForm,
  selectRecipientUsername,
  selectTransferSubmitError,
  selectTransferSubmitting,
  submitFailed,
  submitStarted,
  submitSucceeded,
} from "@/features/transfer-form";
import { selectAmountInput, selectDescription } from "@/features/transfer-form";
import { majorToMinorUnits, formatMoney } from "@/shared/lib/money";
import { useCreateTransferMutation } from "@/services";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { AppText, Button, Card, ErrorBanner } from "@/shared/ui/atoms";

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
  const submitError = useAppSelector(selectTransferSubmitError);
  const [createTransfer] = useCreateTransferMutation();

  const minor = majorToMinorUnits(amountInput);
  const formatted = formatMoney(minor, "USD");

  const handleConfirm = async (): Promise<void> => {
    if (!recipient) {
      return;
    }
    dispatch(submitStarted());
    try {
      const request = {
        recipient_username: recipient,
        amount_minor: minor,
        currency: "USD",
        ...(description ? { description } : {}),
      };
      await createTransfer(request).unwrap();
      dispatch(submitSucceeded());
    } catch {
      dispatch(submitFailed("Transfer failed. Please try again."));
    }
  };

  return (
    <View style={styles.container}>
      <AppText variant="subtitle">Confirm transfer</AppText>

      {submitError ? <ErrorBanner message={submitError} /> : null}

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
        <Button
          label="Back"
          variant="ghost"
          onPress={() => dispatch(backToForm())}
        />
        <Button
          label="Send Money"
          onPress={() => void handleConfirm()}
          loading={submitting}
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
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  primary: {
    flex: 1,
    marginLeft: spacing.md,
  },
});
