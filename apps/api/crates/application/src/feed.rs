//! Public feed façade module.
//!
//! This module exists so the crate can expose [`FeedService`] beside
//! [`crate::TransferService`] at the root (`pub use feed::FeedService`) without
//! implying a separate feed package.
//!
//! **Implementation note:** [`FeedService`] is defined in [`crate::transfer`]
//! alongside [`crate::TransferService`], because feed items are projections of
//! completed transfers and share the transfer repository / broadcaster ports.
//! Import from either `ficus_application::FeedService` or
//! `ficus_application::feed::FeedService`.

pub use crate::transfer::FeedService;
