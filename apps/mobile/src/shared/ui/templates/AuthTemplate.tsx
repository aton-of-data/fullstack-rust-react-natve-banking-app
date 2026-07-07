import { KeyboardAvoidingView, Platform, ScrollView, StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';

import { colors, spacing } from '@/shared/theme';

/**
 * Props for the {@link AuthTemplate} layout.
 */
export interface AuthTemplateProps {
  /** Page content. */
  children: React.ReactNode;
}

/**
 * Centered auth screen layout template.
 *
 * @param props - Template props.
 * @param props.children - Page content.
 * @returns Auth layout wrapper.
 */
export function AuthTemplate({ children }: AuthTemplateProps) {
  return (
    <SafeAreaView style={styles.safe}>
      <KeyboardAvoidingView
        behavior={Platform.OS === 'ios' ? 'padding' : undefined}
        style={styles.flex}
      >
        <ScrollView contentContainerStyle={styles.content} keyboardShouldPersistTaps="handled">
          {children}
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  flex: {
    flex: 1,
  },
  content: {
    flexGrow: 1,
    justifyContent: 'center',
    padding: spacing.lg,
  },
});
