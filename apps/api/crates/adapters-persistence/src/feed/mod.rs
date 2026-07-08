//! Real-time feed broadcast adapters.
//!
//! Feed items are published by the **application** layer after a successful
//! transfer. This module provides in-process fan-out (and optional PostgreSQL
//! LISTEN/NOTIFY bridging). The transfer executor does not publish here.

pub mod in_memory_feed_broadcaster;

pub use in_memory_feed_broadcaster::InMemoryFeedBroadcaster;
