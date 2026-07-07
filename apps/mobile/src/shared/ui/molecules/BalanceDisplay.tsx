import { StyleSheet, View } from "react-native";

import { colors, spacing } from "@/shared/theme";
import { formatMoney } from "@/shared/lib/money";
import { AppText, Card } from "@/shared/ui/atoms";

/**
 * Props for the {@link BalanceDisplay} molecule.
 */
export interface BalanceDisplayProps {
  /** Balance in minor units wire string. */
  balanceMinor: string;
  /** ISO 4217 currency code. */
  currency: string;
  /** Account holder username. */
  username?: string | null;
}

/**
 * Formatted balance display molecule.
 *
 * @param props - Balance display props.
 * @param props.balanceMinor - Balance in minor units.
 * @param props.currency - ISO 4217 currency code.
 * @param props.username - Account holder username.
 * @returns Balance card content.
 */
export function BalanceDisplay({
  balanceMinor,
  currency,
  username,
}: BalanceDisplayProps) {
  const formatted = formatMoney(balanceMinor, currency);

  return (
    <Card style={styles.card}>
      <AppText variant="label" style={styles.label}>
        Available Balance
      </AppText>
      {username ? (
        <AppText variant="caption" style={styles.username}>
          @{username}
        </AppText>
      ) : null}
      <AppText variant="title" style={styles.amount}>
        {formatted}
      </AppText>
      <View style={styles.accent} accessibilityElementsHidden />
    </Card>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.primary,
    borderColor: colors.primaryLight,
  },
  username: {
    color: "rgba(255,255,255,0.75)",
    marginTop: spacing.xs,
  },
  label: {
    color: "rgba(255,255,255,0.85)",
  },
  amount: {
    color: colors.surface,
    marginTop: spacing.sm,
  },
  accent: {
    position: "absolute",
    right: spacing.lg,
    top: spacing.lg,
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: colors.accent,
    opacity: 0.35,
  },
});
