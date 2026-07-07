import { StyleSheet, View } from "react-native";

import { spacing } from "@/shared/theme";
import { AppText, Input, type InputProps } from "@/shared/ui/atoms";

/**
 * Props for the {@link FormField} molecule.
 */
export interface FormFieldProps extends InputProps {
  /** Field label text. */
  label: string;
  /** Optional helper or error text below the input. */
  hint?: string;
  /** Whether the hint represents an error. */
  hintIsError?: boolean;
}

/**
 * Label + input + hint form field molecule.
 *
 * @param props - Form field props.
 * @param props.label - Field label text.
 * @param props.hint - Helper or error text.
 * @param props.hintIsError - Whether hint is an error.
 * @param props.hasError - Validation error state.
 * @returns Form field view.
 */
export function FormField({
  label,
  hint,
  hintIsError = false,
  hasError,
  ...inputProps
}: FormFieldProps) {
  return (
    <View style={styles.container}>
      <AppText variant="label" style={styles.label}>
        {label}
      </AppText>
      <Input hasError={hasError || hintIsError} {...inputProps} />
      {hint ? (
        <AppText variant="caption" error={hintIsError} muted={!hintIsError}>
          {hint}
        </AppText>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    gap: spacing.xs,
  },
  label: {
    marginBottom: spacing.xs,
  },
});
