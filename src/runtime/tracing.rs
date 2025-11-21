/// Initializes the tracing/logging infrastructure for the application.
///
/// This sets up structured logging using the `tracing` crate with:
/// - **Environment-based filtering**: Controlled via `RUST_LOG` environment variable
/// - **Pretty formatting**: Human-readable output with timestamps and log levels
/// - **Span tracking**: Hierarchical context for debugging async operations
///
/// # Environment Variables
///
/// Set `RUST_LOG` to control log verbosity:
/// - `RUST_LOG=info` - Show info, warn, and error messages
/// - `RUST_LOG=debug` - Show debug and above
/// - `RUST_LOG=trace` - Show all messages (very verbose)
/// - `RUST_LOG=actor_recipe=debug` - Debug only for this crate
///
/// # Example
///
/// ```ignore
/// setup_tracing();
/// tracing::info!("Application started");
/// ```
pub fn setup_tracing() {
    // Initialize the tracing subscriber with environment-based filtering
    // This allows users to control log levels via the RUST_LOG env var
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
