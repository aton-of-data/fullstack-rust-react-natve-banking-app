import { describe, expect, it } from 'vitest';

import { LoginPage } from './LoginPage';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

describe('LoginPage', () => {
  it('renders welcome copy and sign-in controls', () => {
    const { root } = renderTestTree(<LoginPage />);
    expect(findByText(root, 'Welcome to Ficus')).toBeTruthy();
    expect(findByLabel(root, 'Username')).toBeTruthy();
    expect(findByLabel(root, 'Password')).toBeTruthy();
    expect(findByText(root, 'Sign In')).toBeTruthy();
  });
});
