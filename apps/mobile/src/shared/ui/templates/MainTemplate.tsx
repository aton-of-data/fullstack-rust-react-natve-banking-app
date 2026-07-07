import { ScrollView, StyleSheet, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { colors, spacing } from "@/shared/theme";
import { AppText } from "@/shared/ui/atoms";

/**
 * Props for the {@link MainTemplate} layout.
 */
export interface MainTemplateProps {
  /** Screen title in the header. */
  title: string;
  /** Optional header right action element. */
  headerRight?: React.ReactNode;
  /** Page content. */
  children: React.ReactNode;
}

/**
 * Main authenticated screen layout template with header.
 *
 * @param props - Template props.
 * @param props.title - Screen title in the header.
 * @param props.headerRight - Optional header right action.
 * @param props.children - Page content.
 * @returns Main layout wrapper.
 */
export function MainTemplate({
  title,
  headerRight,
  children,
}: MainTemplateProps) {
  return (
    <SafeAreaView style={styles.safe} edges={["top", "left", "right"]}>
      <View style={styles.header}>
        <AppText variant="title" style={styles.title}>
          {title}
        </AppText>
        {headerRight}
      </View>
      <ScrollView contentContainerStyle={styles.content}>{children}</ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  header: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.md,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.surface,
  },
  title: {
    fontSize: 22,
  },
  content: {
    padding: spacing.lg,
    gap: spacing.lg,
  },
});
