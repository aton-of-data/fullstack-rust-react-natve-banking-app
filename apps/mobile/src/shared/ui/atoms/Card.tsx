import { StyleSheet, View, type ViewProps } from "react-native";

import { colors, radii, shadows, spacing } from "@/shared/theme";

/**
 * Props for the {@link Card} atom.
 */
export interface CardProps extends ViewProps {
  /** Elevated shadow style. */
  elevated?: boolean;
}

/**
 * Surface card container atom.
 *
 * @param props - Card props.
 * @param props.elevated - Elevated shadow style.
 * @param props.style - Additional styles.
 * @param props.children - Card content.
 * @returns Styled card view.
 */
export function Card({ elevated = true, style, children, ...rest }: CardProps) {
  return (
    <View style={[styles.card, elevated && shadows.md, style]} {...rest}>
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radii.lg,
    padding: spacing.lg,
    borderWidth: 1,
    borderColor: colors.border,
  },
});
