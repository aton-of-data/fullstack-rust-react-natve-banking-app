import { Provider } from 'react-redux';
import { StatusBar } from 'expo-status-bar';

import { store } from '@/store';
import { RootNavigation } from './navigation';

/**
 * Root application component with Redux provider and navigation.
 *
 * @returns Application tree.
 */
export function App() {
  return (
    <Provider store={store}>
      <StatusBar style="auto" />
      <RootNavigation />
    </Provider>
  );
}
