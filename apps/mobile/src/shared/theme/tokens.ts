/**
 * Color design tokens for the Ficus mobile app.
 */
export const colors = {
  /** Deep forest green — primary brand */
  primary: "#0B3D2E",
  /** Lighter green for interactive elements */
  primaryLight: "#147A5A",
  /** Accent gold for highlights */
  accent: "#D4A853",
  /** App background */
  background: "#F4F7F5",
  /** Elevated surface (cards) */
  surface: "#FFFFFF",
  /** Primary text */
  text: "#1A2E28",
  /** Secondary / muted text */
  textMuted: "#5C7269",
  /** Borders and dividers */
  border: "#D8E4DE",
  /** Error states */
  error: "#C0392B",
  /** Success states */
  success: "#1E8449",
  /** Warning states */
  warning: "#D68910",
  /** Overlay scrim */
  overlay: "rgba(11, 61, 46, 0.45)",
} as const;

/**
 * Spacing scale (4px base).
 */
export const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
  xxl: 48,
} as const;

/**
 * Typography scale.
 */
export const typography = {
  fontFamily: {
    regular: "System",
    medium: "System",
    bold: "System",
  },
  fontSize: {
    xs: 12,
    sm: 14,
    md: 16,
    lg: 20,
    xl: 28,
    xxl: 36,
  },
  lineHeight: {
    tight: 1.2,
    normal: 1.5,
    relaxed: 1.75,
  },
} as const;

/**
 * Border radius tokens.
 */
export const radii = {
  sm: 8,
  md: 12,
  lg: 16,
  full: 9999,
} as const;

/**
 * Shadow elevation tokens.
 */
export const shadows = {
  sm: {
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.08,
    shadowRadius: 4,
    elevation: 2,
  },
  md: {
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.12,
    shadowRadius: 8,
    elevation: 4,
  },
} as const;

/**
 * Combined theme object exported for StyleSheet usage.
 */
export const theme = {
  colors,
  spacing,
  typography,
  radii,
  shadows,
} as const;

/**
 * Theme type for typed StyleSheet consumers.
 */
export type Theme = typeof theme;
