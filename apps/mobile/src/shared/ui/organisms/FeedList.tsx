import { FlatList, StyleSheet, View } from 'react-native';

import { spacing } from '@/shared/theme';
import { selectUsername } from '@/features/auth';
import { useGetFeedQuery } from '@/services';
import { useAppSelector } from '@/store/hooks';
import { AppText, ErrorBanner, Spinner } from '@/shared/ui/atoms';
import { EmptyState, FeedItemCard } from '@/shared/ui/molecules';

/**
 * Live transaction feed organism with SSE-backed RTK Query cache.
 *
 * @returns Feed list view.
 */
export function FeedList() {
  const username = useAppSelector(selectUsername);
  const { data, isLoading, isError, refetch } = useGetFeedQuery();

  if (isLoading) {
    return <Spinner message="Loading activity…" />;
  }

  if (isError) {
    return (
      <View style={styles.error}>
        <ErrorBanner message="Could not load activity feed." />
        <AppText
          variant="label"
          onPress={() => void refetch()}
          accessibilityRole="button"
          accessibilityLabel="Retry loading feed"
        >
          Tap to retry
        </AppText>
      </View>
    );
  }

  const items = data ?? [];

  return (
    <View style={styles.container}>
      <AppText variant="subtitle" style={styles.heading}>
        Recent Activity
      </AppText>
      {items.length === 0 ? (
        <EmptyState
          title="No activity yet"
          description="Transfers will appear here in real time."
        />
      ) : (
        <FlatList
          data={items}
          keyExtractor={(item) => item.transfer_id}
          renderItem={({ item }) => <FeedItemCard item={item} currentUsername={username} />}
          scrollEnabled={false}
          accessibilityLabel="Transaction feed"
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    gap: spacing.sm,
  },
  heading: {
    marginBottom: spacing.xs,
  },
  error: {
    gap: spacing.sm,
  },
});
