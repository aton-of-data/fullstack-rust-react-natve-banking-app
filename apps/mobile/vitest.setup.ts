import React from 'react';
import { vi } from 'vitest';

vi.mock('react-native', () => {
  const mockComponent =
    (name: string) =>
    ({ children, testID, ...props }: Record<string, unknown>) =>
      React.createElement(
        name.toLowerCase(),
        { 'data-testid': testID, ...props },
        children as React.ReactNode,
      );

  return {
    StyleSheet: {
      create: <T extends Record<string, unknown>>(styles: T): T => styles,
      flatten: (style: unknown) => style,
    },
    View: mockComponent('View'),
    Text: mockComponent('Text'),
    TextInput: mockComponent('TextInput'),
    Pressable: mockComponent('Pressable'),
    ActivityIndicator: mockComponent('ActivityIndicator'),
    ScrollView: mockComponent('ScrollView'),
    KeyboardAvoidingView: mockComponent('KeyboardAvoidingView'),
    FlatList: ({
      data,
      renderItem,
      keyExtractor,
      ...props
    }: {
      data?: unknown[];
      renderItem?: (info: { item: unknown }) => React.ReactNode;
      keyExtractor?: (item: unknown) => string;
    }) =>
      React.createElement(
        'flatlist',
        props,
        data?.map((item, index) =>
          React.createElement(
            'item',
            { key: keyExtractor?.(item) ?? String(index) },
            renderItem?.({ item }),
          ),
        ),
      ),
    Platform: { OS: 'ios', select: <T>(options: { ios?: T; default?: T }) => options.ios },
  };
});

vi.mock('expo-secure-store', () => ({
  getItemAsync: vi.fn().mockResolvedValue(null),
  setItemAsync: vi.fn().mockResolvedValue(undefined),
  deleteItemAsync: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('react-native-safe-area-context', () => ({
  SafeAreaView: ({ children, ...props }: { children: React.ReactNode }) =>
    React.createElement('safe-area', props, children),
  SafeAreaProvider: ({ children }: { children: React.ReactNode }) =>
    React.createElement('safe-area-provider', null, children),
}));

vi.mock('react-native-screens', () => ({
  enableScreens: vi.fn(),
}));
