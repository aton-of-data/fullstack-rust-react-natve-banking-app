import { describe, expect, it, vi } from 'vitest';

import { RecipientSearch } from './RecipientSearch';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

const useSearchUsersQueryMock = vi.fn();

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useSearchUsersQuery: (...args: unknown[]) => useSearchUsersQueryMock(...args),
  };
});

describe('RecipientSearch', () => {
  it('prompts for minimum query length', () => {
    useSearchUsersQueryMock.mockReturnValue({
      data: undefined,
      isFetching: false,
      isError: false,
    });

    const { root } = renderTestTree(<RecipientSearch />);
    expect(findByText(root, 'Enter at least 2 characters to search')).toBeTruthy();
  });

  it('renders search results and selects recipient', () => {
    useSearchUsersQueryMock.mockReturnValue({
      data: {
        items: [{ user_id: 'u2', username: 'bob' }],
      },
      isFetching: false,
      isError: false,
    });

    const { store, root } = renderTestTree(<RecipientSearch />, {
      preloadedState: {
        transferForm: {
          step: 'search',
          searchQuery: 'bo',
          recipientUserId: null,
          recipientUsername: null,
          amountInput: '',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
      },
    });

    findByLabel(root, 'Select recipient bob').props.onPress();
    expect(store.getState().transferForm.recipientUsername).toBe('bob');
  });

  it('shows loading and error states', () => {
    useSearchUsersQueryMock.mockReturnValue({
      data: undefined,
      isFetching: true,
      isError: false,
    });
    const { root: loadingRoot } = renderTestTree(<RecipientSearch />, {
      preloadedState: {
        transferForm: {
          step: 'search',
          searchQuery: 'bo',
          recipientUserId: null,
          recipientUsername: null,
          amountInput: '',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
      },
    });
    expect(findByText(loadingRoot, 'Searching…')).toBeTruthy();

    useSearchUsersQueryMock.mockReturnValue({
      data: undefined,
      isFetching: false,
      isError: true,
    });
    const { root: errorRoot } = renderTestTree(<RecipientSearch />, {
      preloadedState: {
        transferForm: {
          step: 'search',
          searchQuery: 'bo',
          recipientUserId: null,
          recipientUsername: null,
          amountInput: '',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
      },
    });
    expect(findByText(errorRoot, 'Search failed. Check your connection.')).toBeTruthy();
  });
});
