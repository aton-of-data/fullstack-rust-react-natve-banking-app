//! Shared API contract types for the Ficus backend.
//!
//! This crate is a **placeholder** for cross-cutting request/response DTOs and
//! versioned contract types that are shared by HTTP adapters and (eventually)
//! other clients as endpoints stabilize. It currently re-exports nothing and
//! owns no runtime behavior.
//!
//! # Architectural role
//!
//! Sits beside the hexagonal rings as a thin shared-types surface. It must not
//! become a dumping ground for domain rules or persistence entities.
//!
//! # What this crate may depend on
//!
//! - `serde` / pure value types suitable for wire contracts
//! - Optionally `ficus-domain` value types when contracts intentionally mirror
//!   domain concepts (prefer mapping at the adapter boundary when unclear)
//!
//! # What this crate must not depend on
//!
//! - Axum extractors, SeaORM entities, JWT/Argon2, or infrastructure wiring
//! - Application services or port trait implementations
//!
//! # Neighboring crates
//!
//! - `adapters-http` may consume stabilized contract types for OpenAPI schemas.
//! - `application` DTOs remain the orchestration-facing shapes until this crate
//!   is populated intentionally.
//!
//! Populate types here only when an endpoint contract is stable enough to share
//! without coupling adapters to application internals.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]
