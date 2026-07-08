import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import { selectLoginForm } from '@/features/ui';
import { useLoginMutation } from '@/services';
import { setCredentials } from '@/features/auth';
import { clearLoginForm, setLoginPassword, setLoginUsername } from '@/features/ui';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import { AppText, Button, ErrorBanner } from '@/shared/ui/atoms';
import { FormField } from '@/shared/ui/molecules';

/**
 * Login form organism wired to Redux and RTK Query.
 *
 * @returns Login form view.
 */
export function LoginForm() {
  const dispatch = useAppDispatch();
  const { username, password } = useAppSelector(selectLoginForm);
  const [login, { isLoading, error }] = useLoginMutation();

  const handleSubmit = async (): Promise<void> => {
    try {
      const result = await login({ username, password }).unwrap();
      dispatch(
        setCredentials({
          accessToken: result.access_token,
          userId: result.user_id,
          username: result.username,
        }),
      );
      dispatch(clearLoginForm());
    } catch {
      // Error surfaced via RTK Query error state
    }
  };

  const errorMessage =
    error && 'status' in error ? 'Invalid username or password. Please try again.' : null;

  return (
    <View style={styles.container} testID="login-screen">
      <AppText variant="title" style={styles.title} testID="login-title">
        Welcome to Ficus
      </AppText>
      <AppText variant="body" muted style={styles.subtitle}>
        Sign in to send and receive money
      </AppText>

      {errorMessage ? <ErrorBanner message={errorMessage} testID="login-error" /> : null}

      <FormField
        label="Username"
        value={username}
        onChangeText={(text) => dispatch(setLoginUsername(text))}
        autoCapitalize="none"
        autoCorrect={false}
        accessibilityLabel="Username"
        textContentType="username"
        testID="login-username"
      />

      <FormField
        label="Password"
        value={password}
        onChangeText={(text) => dispatch(setLoginPassword(text))}
        secureTextEntry
        accessibilityLabel="Password"
        textContentType="password"
        testID="login-password"
      />

      <Button
        label="Sign In"
        testID="login-submit"
        onPress={() => void handleSubmit()}
        loading={isLoading}
        disabled={!username || !password || isLoading}
        accessibilityLabel="Sign In"
        style={styles.button}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    gap: spacing.md,
  },
  title: {
    textAlign: 'center',
  },
  subtitle: {
    textAlign: 'center',
    marginBottom: spacing.md,
  },
  button: {
    marginTop: spacing.sm,
  },
});
