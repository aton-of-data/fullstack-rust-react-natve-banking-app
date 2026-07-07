import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import { AppText } from '@/shared/ui/atoms';

/**
 * Props for the {@link EmptyState} molecule.
 */
export interface EmptyStateProps {
  /** Primary empty state title. */
  title: string;
  /** Supporting description. */
  description?: string;
}

/**
 * Centered empty state placeholder molecule.
 *
 * @param props - Empty state props.
 * @param props.title - Primary empty state title.
 * @param props.description - Supporting description.
 * @returns Empty state view.
 */
export function EmptyState({ title, description }: EmptyStateProps) {
  return (
    <View style={styles.container} accessibilityRole="text">
      <AppText variant="subtitle" style={styles.title}>
        {title}
      </AppText>
      {description ? (
        <AppText variant="body" muted style={styles.description}>
          {description}
        </AppText>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    padding: spacing.xl,
    gap: spacing.sm,
  },
  title: {
    textAlign: 'center',
  },
  description: {
    textAlign: 'center',
  },
});
