import { describe, expect, it, vi } from 'vitest';

import { BalanceCard } from './BalanceCard';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

const useGetBalanceQueryMock = vi.fn();

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useGetBalanceQuery: () => useGetBalanceQueryMock(),
  };
});

const authState = {
  status: 'authenticated' as const,
  accessToken: 'token',
  userId: 'user-1',
  username: 'alice',
  hydrated: true,
};

describe('BalanceCard', () => {
  it('shows loading spinner while balance loads', () => {
    useGetBalanceQueryMock.mockReturnValue({
      isLoading: true,
      isError: false,
      refetch: vi.fn(),
    });

    const { root } = renderTestTree(<BalanceCard />, { preloadedState: { auth: authState } });
    expect(findByText(root, 'Loading balance…')).toBeTruthy();
  });

  it('renders formatted balance when data is available', () => {
    useGetBalanceQueryMock.mockReturnValue({
      data: { balance_minor: '12345', currency: 'USD' },
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    });

    const { root } = renderTestTree(<BalanceCard />, { preloadedState: { auth: authState } });
    expect(findByText(root, '$123.45')).toBeTruthy();
    expect(findByText(root, '@alice')).toBeTruthy();
  });

  it('shows retry guidance on error', () => {
    const refetch = vi.fn();
    useGetBalanceQueryMock.mockReturnValue({
      isLoading: false,
      isError: true,
      refetch,
    });

    const { root } = renderTestTree(<BalanceCard />, { preloadedState: { auth: authState } });
    expect(findByText(root, 'Could not load your balance.')).toBeTruthy();
    findByLabel(root, 'Retry loading balance').props.onPress();
    expect(refetch).toHaveBeenCalled();
  });
});
