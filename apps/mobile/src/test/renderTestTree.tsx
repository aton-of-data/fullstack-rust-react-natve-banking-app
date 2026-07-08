import type { ReactElement } from 'react';
import renderer, { type ReactTestInstance } from 'react-test-renderer';
import { Provider } from 'react-redux';

import { createTestStore } from './renderWithProviders';

type TestStore = ReturnType<typeof createTestStore>;

/**
 * Renders a component with Redux provider using react-test-renderer.
 *
 * @param ui React element to render.
 * @param options Optional render options.
 * @param options.preloadedState Optional partial Redux initial state.
 * @returns Renderer tree and store.
 */
export function renderTestTree(
  ui: ReactElement,
  options?: { preloadedState?: Record<string, unknown> },
): { store: TestStore; root: ReactTestInstance } {
  const store = createTestStore(options?.preloadedState);
  const tree = renderer.create(<Provider store={store}>{ui}</Provider>);
  return { store, root: tree.root };
}

/**
 * Finds the first node whose rendered text includes the given substring.
 *
 * @param root React test renderer root.
 * @param text Text to search for.
 * @returns Matching instance.
 */
export function findByText(root: ReactTestInstance, text: string): ReactTestInstance {
  const matches = root.findAll(
    (node: ReactTestInstance) =>
      typeof node.children[0] === 'string' && node.children.join('').includes(text),
  );
  if (matches.length === 0) {
    throw new Error(`Text not found: ${text}`);
  }
  return matches[0]!;
}

/**
 * Finds a node by accessibility label prop.
 *
 * @param root React test renderer root.
 * @param label Accessibility label value.
 * @returns Matching instance.
 */
export function findByLabel(root: ReactTestInstance, label: string): ReactTestInstance {
  return root.findByProps({ accessibilityLabel: label });
}
