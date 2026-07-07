import { StyleSheet } from "react-native";

import { clearCredentials } from "@/features/auth";
import { useLogoutMutation } from "@/services";
import { useAppDispatch } from "@/store/hooks";
import { spacing } from "@/shared/theme";
import { BalanceCard, FeedList } from "@/shared/ui/organisms";
import { MainTemplate } from "@/shared/ui/templates";
import { Button } from "@/shared/ui/atoms";

/**
 * Home page with balance card and live activity feed.
 *
 * @returns Home screen.
 */
export function HomePage() {
  const dispatch = useAppDispatch();
  const [logout, { isLoading }] = useLogoutMutation();

  const handleLogout = async (): Promise<void> => {
    try {
      await logout().unwrap();
    } catch {
      // Proceed with local logout even if server call fails
    }
    dispatch(clearCredentials());
  };

  return (
    <MainTemplate
      title="Ficus"
      headerRight={
        <Button
          label="Log out"
          variant="ghost"
          onPress={() => void handleLogout()}
          loading={isLoading}
          style={styles.logout}
        />
      }
    >
      <BalanceCard />
      <FeedList />
    </MainTemplate>
  );
}

const styles = StyleSheet.create({
  logout: {
    minHeight: 36,
    paddingHorizontal: spacing.sm,
  },
});
