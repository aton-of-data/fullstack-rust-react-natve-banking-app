import { ActivityIndicator, StyleSheet, View } from 'react-native';

import { colors, spacing } from '@/shared/theme';
import { AppText } from './AppText';

/**
 * Props for the {@link Spinner} atom.
 */
export interface SpinnerProps {
  /** Optional loading message. */
  message?: string;
}

/**
 * Centered loading spinner with optional message.
 *
 * @param props - Spinner props.
 * @param props.message - Optional loading message.
 * @returns Loading indicator view.
 */
export function Spinner({ message = 'Loading…' }: SpinnerProps) {
  return (
    <View style={styles.container} accessibilityRole="progressbar" accessibilityLabel={message}>
      <ActivityIndicator size="large" color={colors.primary} />
      <AppText variant="caption" muted style={styles.message}>
        {message}
      </AppText>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    justifyContent: 'center',
    padding: spacing.xl,
    gap: spacing.md,
  },
  message: {
    textAlign: 'center',
  },
});
