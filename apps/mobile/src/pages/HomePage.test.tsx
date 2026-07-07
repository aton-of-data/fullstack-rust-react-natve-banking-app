import { describe, expect, it, vi } from 'vitest';

import { HomePage } from './HomePage';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

const logoutMock = vi.fn();
const useLogoutMutationMock = vi.fn();

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useLogoutMutation: () => useLogoutMutationMock(),
    useGetBalanceQuery: () => ({
      data: { balance_minor: '2500', currency: 'USD' },
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    }),
    useGetFeedQuery: () => ({
      data: [],
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    }),
  };
});

describe('HomePage', () => {
  it('renders balance and feed sections for authenticated users', () => {
    useLogoutMutationMock.mockReturnValue([logoutMock, { isLoading: false }]);
    logoutMock.mockReturnValue({ unwrap: () => Promise.resolve(undefined) });

    const { root } = renderTestTree(<HomePage />, {
      preloadedState: {
        auth: {
          status: 'authenticated',
          accessToken: 'token',
          userId: 'user-1',
          username: 'alice',
          hydrated: true,
        },
      },
    });

    expect(findByText(root, 'Ficus')).toBeTruthy();
    expect(findByText(root, 'Log out')).toBeTruthy();
    expect(findByText(root, 'Recent Activity')).toBeTruthy();
    expect(findByText(root, 'No activity yet')).toBeTruthy();
  });

  it('clears credentials after logout even when API fails', async () => {
    useLogoutMutationMock.mockReturnValue([logoutMock, { isLoading: false }]);
    logoutMock.mockReturnValue({
      unwrap: () => Promise.reject(new Error('offline')),
    });

    const { store, root } = renderTestTree(<HomePage />, {
      preloadedState: {
        auth: {
          status: 'authenticated',
          accessToken: 'token',
          userId: 'user-1',
          username: 'alice',
          hydrated: true,
        },
      },
    });

    findByLabel(root, 'Log out').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().auth.status).toBe('unauthenticated');
    });
  });
});
