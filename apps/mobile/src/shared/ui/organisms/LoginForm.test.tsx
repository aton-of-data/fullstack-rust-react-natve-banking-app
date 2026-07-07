import { describe, expect, it, vi } from 'vitest';

import { LoginForm } from './LoginForm';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

const loginMock = vi.fn();

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useLoginMutation: () => [
      loginMock.mockReturnValue({
        unwrap: () =>
          Promise.resolve({
            access_token: 'token',
            user_id: 'user-1',
            username: 'alice',
          }),
      }),
      { isLoading: false, error: null },
    ],
  };
});

describe('LoginForm', () => {
  it('renders sign-in fields and dispatches credentials on submit', async () => {
    const { store, root } = renderTestTree(<LoginForm />);
    const username = findByLabel(root, 'Username');
    const password = findByLabel(root, 'Password');

    username.props.onChangeText('alice');
    password.props.onChangeText('password123');
    findByLabel(root, 'Sign In').props.onPress();

    await vi.waitFor(() => {
      expect(loginMock).toHaveBeenCalled();
      expect(store.getState().auth.status).toBe('authenticated');
    });
  });
});
