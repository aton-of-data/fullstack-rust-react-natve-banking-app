import { Text, type TextProps, type TextStyle } from 'react-native';

import { colors, typography } from '@/shared/theme';

/**
 * Text style variants.
 */
export type TextVariant = 'title' | 'subtitle' | 'body' | 'caption' | 'label';

/**
 * Props for the {@link AppText} atom.
 */
export interface AppTextProps extends TextProps {
  /** Typography variant. */
  variant?: TextVariant;
  /** Muted secondary color. */
  muted?: boolean;
  /** Error color. */
  error?: boolean;
}

const variantMap: Record<TextVariant, TextStyle> = {
  title: { fontSize: typography.fontSize.xl, fontWeight: '700' },
  subtitle: { fontSize: typography.fontSize.lg, fontWeight: '600' },
  body: { fontSize: typography.fontSize.md, fontWeight: '400' },
  caption: { fontSize: typography.fontSize.sm, fontWeight: '400' },
  label: {
    fontSize: typography.fontSize.sm,
    fontWeight: '600',
    letterSpacing: 0.5,
  },
};

/**
 * Typography atom with design-token variants.
 *
 * @param props - Text props.
 * @param props.variant - Typography variant.
 * @param props.muted - Muted secondary color.
 * @param props.error - Error color.
 * @param props.style - Additional styles.
 * @param props.children - Text content.
 * @returns Styled text element.
 */
export function AppText({
  variant = 'body',
  muted = false,
  error = false,
  style,
  children,
  ...rest
}: AppTextProps) {
  const color = error ? colors.error : muted ? colors.textMuted : colors.text;

  return (
    <Text style={[variantMap[variant], { color }, style]} {...rest}>
      {children}
    </Text>
  );
}
