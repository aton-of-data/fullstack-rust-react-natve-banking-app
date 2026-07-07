import { StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
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
  const trimmed = query.trim();
  const { data, isFetching, isError } = useSearchUsersQuery(
    { query: trimmed },
    { skip: trimmed.length < 2 },
  );

  return (
    <View style={styles.container}>
      <AppText variant="subtitle">Find recipient</AppText>
      <FormField
        label="Username"
        value={query}
        onChangeText={(text) => dispatch(setSearchQuery(text))}
        placeholder="Search by username"
        autoCapitalize="none"
        autoCorrect={false}
        accessibilityLabel="Search recipient username"
      />

      {trimmed.length < 2 ? (
        <AppText variant="caption" muted>
          Enter at least 2 characters to search
        </AppText>
      ) : null}

      {isFetching ? <Spinner message="Searching…" /> : null}

      {isError ? (
        <AppText variant="caption" error>
          Search failed. Check your connection.
        </AppText>
      ) : null}

      {trimmed.length >= 2 && !isFetching && data?.items.length === 0 ? (
        <EmptyState title="No users found" description="Try a different username." />
      ) : null}

      {data?.items.map((user) => (
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
