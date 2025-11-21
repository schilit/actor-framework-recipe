/// Initializes the tracing/logging infrastructure for the application.
///
/// Sets up the tracing subscriber for structured logging.
///
/// This initializes the global tracing subscriber with environment-based filtering.
/// The subscriber uses a compact format that shows span hierarchy inline.
///
/// # Configuration
///
/// Log levels are controlled via the `RUST_LOG` environment variable:
///
/// - `RUST_LOG=info` - Clean, concise output showing workflow hierarchy
/// - `RUST_LOG=debug` - Includes full request/response details (verbose)
/// - `RUST_LOG=trace` - Show all messages (very verbose)
/// - `RUST_LOG=actor_recipe::framework=debug` - Filter to specific modules
///
/// # Output Formats
///
/// The compact format shows span hierarchy inline:
/// - `INFO user_creation: Creating test user` - top-level span
/// - `INFO order_processing:create_order: Processing request` - nested spans
///
/// Use `debug` level to see full object details at function entry points.
///
/// # Examples
///
/// ```bash
/// # Clean output (recommended)
/// RUST_LOG=info cargo run
///
/// # Show full request details
/// RUST_LOG=debug cargo run
///
/// # Debug framework internals only
/// RUST_LOG=info,actor_recipe::framework=debug cargo run
/// ```
pub fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)  // Don't show module paths - we use entity_type instead
        .compact()  // Compact format shows spans inline (e.g., "order_processing:create_order")
        .init();
}
