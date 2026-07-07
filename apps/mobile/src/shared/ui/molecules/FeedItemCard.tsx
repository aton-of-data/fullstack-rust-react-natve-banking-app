import type { FeedItem } from "@ficus/contracts";
import { StyleSheet, View } from "react-native";

import { spacing } from "@/shared/theme";
import { formatMoney } from "@/shared/lib/money";
import { AppText, Card } from "@/shared/ui/atoms";

/**
 * Props for the {@link FeedItemCard} molecule.
 */
export interface FeedItemCardProps {
  /** Feed item data. */
  item: FeedItem;
  /** Current user's username for direction labeling. */
  currentUsername?: string | null;
}

/**
 * Single feed transaction item molecule.
 *
 * @param props - Feed item props.
 * @param props.item - Feed item data.
 * @param props.currentUsername - Current user for direction labeling.
 * @returns Feed item card.
 */
export function FeedItemCard({ item, currentUsername }: FeedItemCardProps) {
  const amount = formatMoney(item.amount_minor, item.currency);
  const isSender = currentUsername === item.sender_username;
  const direction = isSender ? "sent" : "received";
  const counterparty = isSender
    ? item.recipient_username
    : item.sender_username;

  return (
    <Card elevated={false} style={styles.card}>
      <View style={styles.row}>
        <AppText variant="subtitle">{amount}</AppText>
        <AppText variant="caption" muted>
          {direction}
        </AppText>
      </View>
      <AppText variant="body">
        {isSender ? "To" : "From"} @{counterparty}
      </AppText>
      {item.description ? (
        <AppText variant="caption" muted numberOfLines={2}>
          {item.description}
        </AppText>
      ) : null}
      <AppText variant="caption" muted style={styles.time}>
        {new Date(item.created_at).toLocaleString()}
      </AppText>
    </Card>
  );
}

const styles = StyleSheet.create({
  card: {
    marginBottom: spacing.sm,
  },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: spacing.xs,
  },
  time: {
    marginTop: spacing.xs,
  },
});
