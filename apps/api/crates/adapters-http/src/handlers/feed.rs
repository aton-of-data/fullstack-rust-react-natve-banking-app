use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use ficus_application::dto::{FeedItemResponse, PageResponse};
use futures::future;
use futures::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;

use crate::error::ApiError;
use crate::metrics::{sse_connection_closed, sse_connection_opened};
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

/// Query parameters for paginated feed listing.
#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct FeedQuery {
    /// Opaque pagination cursor.
    pub cursor: Option<String>,
}

/// Returns a paginated global transaction feed.
#[utoipa::path(
    get,
    path = "/v1/feed",
    params(FeedQuery),
    responses(
        (status = 200, description = "Feed page", body = PageResponse<FeedItemResponse>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "feed"
)]
pub async fn list_feed(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(params): Query<FeedQuery>,
) -> Result<Json<PageResponse<FeedItemResponse>>, ApiError> {
    let page = state
        .feed
        .list(params.cursor.as_deref(), state.default_page_size)
        .await?;

    let items = page.items.into_iter().map(Into::into).collect();
    Ok(Json(PageResponse {
        items,
        next_cursor: page.next_cursor,
    }))
}

/// Streams live feed events over Server-Sent Events.
#[utoipa::path(
    get,
    path = "/v1/feed/stream",
    responses(
        (status = 200, description = "SSE stream of feed events", content_type = "text/event-stream"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "feed"
)]
pub async fn stream_feed(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let replay = if let Some(ref cursor) = last_event_id {
        state
            .feed
            .list(Some(cursor), state.default_page_size)
            .await?
            .items
    } else {
        Vec::new()
    };

    let replay_stream = stream::iter(
        replay
            .into_iter()
            .map(|item| Ok::<Event, Infallible>(feed_event(&FeedItemResponse::from(item)))),
    );

    let live_rx = state.feed.subscribe();
    let live_stream = BroadcastStream::new(live_rx).filter_map(|msg| {
        future::ready(match msg {
            Ok(item) => Some(Ok::<Event, Infallible>(feed_event(
                &FeedItemResponse::from(item),
            ))),
            Err(_) => None,
        })
    });

    sse_connection_opened();
    let combined = GuardedSseStream {
        inner: replay_stream.chain(live_stream),
        _guard: SseConnectionGuard,
    };

    Ok(Sse::new(combined).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

fn feed_event(item: &FeedItemResponse) -> Event {
    Event::default()
        .id(item.transfer_id.to_string())
        .event("transfer")
        .json_data(item)
        .expect("feed item serializes")
}

/// Decrements the SSE gauge when the stream is dropped.
struct SseConnectionGuard;

impl Drop for SseConnectionGuard {
    fn drop(&mut self) {
        sse_connection_closed();
    }
}

struct GuardedSseStream<S> {
    inner: S,
    _guard: SseConnectionGuard,
}

impl<S> Stream for GuardedSseStream<S>
where
    S: Stream<Item = Result<Event, Infallible>> + Unpin,
{
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
