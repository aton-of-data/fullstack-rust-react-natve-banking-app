import { Pressable, StyleSheet, View } from 'react-native';

import { colors, radii, spacing } from '@/shared/theme';
import { AppText } from './AppText';

/**
 * Props for the {@link ErrorBanner} atom.
 */
export interface ErrorBannerProps {
  /** Error message to display. */
  message: string;
  /** Called when the user dismisses the banner. */
  onDismiss?: () => void;
  /** Stable test identifier for E2E. */
  testID?: string;
}

/**
 * Dismissible error banner atom.
 *
 * @param props - Banner props.
 * @param props.message - Error message to display.
 * @param props.onDismiss - Dismiss handler.
 * @param props.testID - Optional test identifier.
 * @returns Error banner view.
 */
export function ErrorBanner({ message, onDismiss, testID }: ErrorBannerProps) {
  return (
    <View style={styles.container} accessibilityRole="alert" testID={testID}>
      <AppText variant="body" error style={styles.message}>
        {message}
      </AppText>
      {onDismiss ? (
        <Pressable
          onPress={onDismiss}
          accessibilityRole="button"
          accessibilityLabel="Dismiss error"
          hitSlop={8}
        >
          <AppText variant="label" style={styles.dismiss}>
            Dismiss
          </AppText>
        </Pressable>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FDECEA',
    borderRadius: radii.md,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.error,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: spacing.sm,
  },
  message: {
    flex: 1,
  },
  dismiss: {
    color: colors.error,
  },
});
