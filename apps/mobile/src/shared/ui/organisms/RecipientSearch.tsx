import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import { selectUserId, selectUsername } from '@/features/auth';
import { selectRecipient, setSearchQuery } from '@/features/transfer-form';
import { selectSearchQuery } from '@/features/transfer-form';
import { useSearchUsersQuery } from '@/services';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import { AppText, Spinner } from '@/shared/ui/atoms';
import { EmptyState, FormField, SearchResultItem } from '@/shared/ui/molecules';

/**
 * Recipient search organism for the transfer wizard.
 *
 * @returns Recipient search view.
 */
export function RecipientSearch() {
  const dispatch = useAppDispatch();
  const query = useAppSelector(selectSearchQuery);
  const currentUserId = useAppSelector(selectUserId);
  const currentUsername = useAppSelector(selectUsername);
  const trimmed = query.trim();
  const { data, isFetching, isError } = useSearchUsersQuery(
    { query: trimmed },
    { skip: trimmed.length < 2 },
  );

  const results =
    data?.items.filter(
      (user) =>
        user.user_id !== currentUserId &&
        user.username.toLowerCase() !== (currentUsername ?? '').toLowerCase(),
    ) ?? [];

  return (
    <View style={styles.container} testID="transfer-search-step">
      <AppText variant="subtitle">Find recipient</AppText>
      <FormField
        label="Username"
        value={query}
        onChangeText={(text) => dispatch(setSearchQuery(text))}
        placeholder="Search by username"
        autoCapitalize="none"
        autoCorrect={false}
        accessibilityLabel="Search recipient username"
        testID="transfer-search"
      />

      {trimmed.length < 2 ? (
        <AppText variant="caption" muted>
          Enter at least 2 characters to search
        </AppText>
      ) : null}

      {isFetching ? <Spinner message="Searching…" /> : null}

      {isError ? (
        <AppText variant="caption" error testID="transfer-search-error">
          Search failed. Check your connection.
        </AppText>
      ) : null}

      {trimmed.length >= 2 && !isFetching && results.length === 0 && !isError ? (
        <EmptyState title="No users found" description="Try a different username." />
      ) : null}

      {results.map((user) => (
        <SearchResultItem
          key={user.user_id}
          user={user}
          onSelect={(selected) =>
            dispatch(
              selectRecipient({
                userId: selected.user_id,
                username: selected.username,
              }),
            )
          }
        />
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    gap: spacing.md,
  },
});
