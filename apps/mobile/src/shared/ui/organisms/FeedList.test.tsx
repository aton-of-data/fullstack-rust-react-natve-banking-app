import { describe, expect, it, vi } from 'vitest';

import { FeedList } from './FeedList';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

const useGetFeedQueryMock = vi.fn();

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useGetFeedQuery: () => useGetFeedQueryMock(),
  };
});

describe('FeedList', () => {
  it('renders empty state when feed has no items', () => {
    useGetFeedQueryMock.mockReturnValue({
      data: [],
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    });

    const { root } = renderTestTree(<FeedList />);
    expect(findByText(root, 'Recent Activity')).toBeTruthy();
    expect(findByText(root, 'No activity yet')).toBeTruthy();
  });

  it('renders feed list when data is available', () => {
    useGetFeedQueryMock.mockReturnValue({
      data: [
        {
          transfer_id: 't-1',
          sender_username: 'alice',
          recipient_username: 'bob',
          amount_minor: '500',
          currency: 'USD',
          created_at: '2025-01-01T00:00:00Z',
        },
      ],
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    });

    const { root } = renderTestTree(<FeedList />, {
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

    expect(findByLabel(root, 'Transaction feed')).toBeTruthy();
  });

  it('shows retry UI when feed query fails', () => {
    const refetch = vi.fn();
    useGetFeedQueryMock.mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      refetch,
    });

    const { root } = renderTestTree(<FeedList />);
    expect(findByText(root, 'Could not load activity feed.')).toBeTruthy();
    findByLabel(root, 'Retry loading feed').props.onPress();
    expect(refetch).toHaveBeenCalled();
  });
});
