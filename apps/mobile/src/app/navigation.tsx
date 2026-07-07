import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';

import { selectAuthHydrated, selectIsAuthenticated } from '@/features/auth';
import { useAppSelector } from '@/store/hooks';
import { HomePage, LoginPage, TransferPage } from '@/pages';
import { colors } from '@/shared/theme';
import { Spinner } from '@/shared/ui/atoms';

/**
 * Root stack param list.
 */
export type RootStackParamList = {
  Login: undefined;
  Main: undefined;
};

/**
 * Main tab param list.
 */
export type MainTabParamList = {
  Home: undefined;
  Transfer: undefined;
};

const Stack = createNativeStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<MainTabParamList>();

/**
 * Authenticated bottom tab navigator.
 *
 * @returns Tab navigator.
 */
function MainTabs() {
  return (
    <Tab.Navigator
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: colors.primary,
        tabBarInactiveTintColor: colors.textMuted,
        tabBarStyle: { borderTopColor: colors.border },
      }}
    >
      <Tab.Screen name="Home" component={HomePage} options={{ title: 'Home' }} />
      <Tab.Screen name="Transfer" component={TransferPage} options={{ title: 'Send' }} />
    </Tab.Navigator>
  );
}

/**
 * Root navigation container with auth gating.
 *
 * @returns Navigation tree.
 */
export function RootNavigation() {
  const hydrated = useAppSelector(selectAuthHydrated);
  const isAuthenticated = useAppSelector(selectIsAuthenticated);

  if (!hydrated) {
    return <Spinner message="Starting Ficus…" />;
  }

  return (
    <NavigationContainer>
      <Stack.Navigator screenOptions={{ headerShown: false }}>
        {isAuthenticated ? (
          <Stack.Screen name="Main" component={MainTabs} />
        ) : (
          <Stack.Screen name="Login" component={LoginPage} />
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
