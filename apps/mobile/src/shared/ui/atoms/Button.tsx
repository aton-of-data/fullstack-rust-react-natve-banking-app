import {
  ActivityIndicator,
  Pressable,
  StyleSheet,
  Text,
  type PressableProps,
  type StyleProp,
  type ViewStyle,
} from 'react-native';

import { colors, radii, spacing, typography } from '@/shared/theme';

/**
 * Button visual variants.
 */
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';

/**
 * Props for the {@link Button} atom.
 */
export interface ButtonProps extends Omit<PressableProps, 'children'> {
  /** Button label text. */
  label: string;
  /** Visual style variant. */
  variant?: ButtonVariant;
  /** Shows a loading spinner and disables interaction. */
  loading?: boolean;
}

const variantStyles: Record<ButtonVariant, { bg: string; text: string; border?: string }> = {
  primary: { bg: colors.primary, text: colors.surface },
  secondary: {
    bg: colors.surface,
    text: colors.primary,
    border: colors.primary,
  },
  ghost: { bg: 'transparent', text: colors.primaryLight },
  danger: { bg: colors.error, text: colors.surface },
};

/**
 * Accessible pressable button atom.
 *
 * @param props - Button props.
 * @param props.label - Button label text.
 * @param props.variant - Visual style variant.
 * @param props.loading - Shows loading spinner.
 * @param props.disabled - Disables interaction.
 * @param props.style - Additional styles.
 * @param props.accessibilityLabel - Accessibility label override.
 * @param props.accessibilityHint - Accessibility hint for screen readers.
 * @param props.testID - Stable test identifier for E2E.
 * @returns Styled button element.
 */
export function Button({
  label,
  variant = 'primary',
  loading = false,
  disabled,
  style,
  accessibilityLabel,
  accessibilityHint,
  testID,
  ...rest
}: ButtonProps) {
  const v = variantStyles[variant];
  const isDisabled = disabled || loading;
  const baseStyle = styles.base;

  return (
    <Pressable
      accessibilityRole="button"
      accessibilityLabel={accessibilityLabel ?? label}
      accessibilityHint={accessibilityHint}
      accessibilityState={{ disabled: isDisabled, busy: loading }}
      disabled={isDisabled}
      testID={testID}
      style={({ pressed }): StyleProp<ViewStyle> => [
        baseStyle,
        {
          backgroundColor: v.bg,
          borderColor: v.border ?? 'transparent',
          opacity: pressed ? 0.85 : 1,
        },
        isDisabled ? styles.disabled : null,
        style as StyleProp<ViewStyle>,
      ]}
      {...rest}
    >
      {loading ? (
        <ActivityIndicator color={v.text} accessibilityLabel="Loading" />
      ) : (
        <Text style={[styles.label, { color: v.text }]}>{label}</Text>
      )}
    </Pressable>
  );
}

const styles = StyleSheet.create({
  base: {
    minHeight: 48,
    borderRadius: radii.md,
    borderWidth: 1.5,
    paddingHorizontal: spacing.lg,
    alignItems: 'center',
    justifyContent: 'center',
  },
  label: {
    fontSize: typography.fontSize.md,
    fontWeight: '600',
  },
  disabled: {
    opacity: 0.5,
  },
});
