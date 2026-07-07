import { StyleSheet, TextInput, type TextInputProps } from 'react-native';

import { colors, radii, spacing, typography } from '@/shared/theme';

/**
 * Props for the {@link Input} atom.
 */
export interface InputProps extends TextInputProps {
  /** Whether the field has a validation error. */
  hasError?: boolean;
}

/**
 * Accessible text input atom.
 *
 * @param props - Input props.
 * @param props.hasError - Validation error state.
 * @param props.style - Additional styles.
 * @param props.accessibilityLabel - Accessibility label.
 * @returns Styled text input.
 */
export function Input({ hasError = false, style, accessibilityLabel, ...rest }: InputProps) {
  return (
    <TextInput
      accessibilityLabel={accessibilityLabel}
      placeholderTextColor={colors.textMuted}
      style={[styles.input, hasError && styles.inputError, style]}
      {...rest}
    />
  );
}

const styles = StyleSheet.create({
  input: {
    minHeight: 48,
    borderWidth: 1.5,
    borderColor: colors.border,
    borderRadius: radii.md,
    paddingHorizontal: spacing.md,
    fontSize: typography.fontSize.md,
    color: colors.text,
    backgroundColor: colors.surface,
  },
  inputError: {
    borderColor: colors.error,
  },
});
