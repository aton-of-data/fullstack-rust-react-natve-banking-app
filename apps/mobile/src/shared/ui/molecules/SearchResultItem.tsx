import type { UserSearchItem } from '@ficus/contracts';
import { Pressable, StyleSheet } from 'react-native';

import { colors, radii, spacing } from '@/shared/theme';
import { AppText } from '@/shared/ui/atoms';

/**
 * Props for the {@link SearchResultItem} molecule.
 */
export interface SearchResultItemProps {
  /** User search result. */
  user: UserSearchItem;
  /** Called when the user selects this result. */
  onSelect: (user: UserSearchItem) => void;
}

/**
 * Selectable user search result row.
 *
 * @param props - Search result props.
 * @param props.user - User search result.
 * @param props.onSelect - Selection handler.
 * @returns Pressable search result row.
 */
export function SearchResultItem({ user, onSelect }: SearchResultItemProps) {
  return (
    <Pressable
      onPress={() => onSelect(user)}
      accessibilityRole="button"
      accessibilityLabel={`Select recipient ${user.username}`}
      testID={`recipient-${user.username}`}
      style={({ pressed }) => [styles.row, pressed && styles.pressed]}
    >
      <AppText variant="body">@{user.username}</AppText>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  row: {
    paddingVertical: spacing.md,
    paddingHorizontal: spacing.md,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    borderRadius: radii.sm,
  },
  pressed: {
    backgroundColor: colors.background,
  },
});
