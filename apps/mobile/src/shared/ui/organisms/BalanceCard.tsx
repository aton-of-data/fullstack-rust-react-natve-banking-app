import { StyleSheet, View } from "react-native";

import { spacing } from "@/shared/theme";
import { selectUsername } from "@/features/auth";
import { useGetBalanceQuery } from "@/services";
import { useAppSelector } from "@/store/hooks";
import { AppText, ErrorBanner, Spinner } from "@/shared/ui/atoms";
import { BalanceDisplay } from "@/shared/ui/molecules";

/**
 * Balance card organism with loading and error states.
 *
 * @returns Balance card view.
 */
export function BalanceCard() {
  const username = useAppSelector(selectUsername);
  const { data, isLoading, isError, refetch } = useGetBalanceQuery();

  if (isLoading) {
    return <Spinner message="Loading balance…" />;
  }

  if (isError || !data) {
    return (
      <View style={styles.error}>
        <ErrorBanner message="Could not load your balance." />
        <AppText
          variant="label"
          style={styles.retry}
          onPress={() => void refetch()}
          accessibilityRole="button"
          accessibilityLabel="Retry loading balance"
        >
          Tap to retry
        </AppText>
      </View>
    );
  }

  return (
    <BalanceDisplay
      balanceMinor={data.balance_minor}
      currency={data.currency}
      username={username}
    />
  );
}

const styles = StyleSheet.create({
  error: {
    gap: spacing.sm,
  },
  retry: {
    textAlign: "center",
  },
});
