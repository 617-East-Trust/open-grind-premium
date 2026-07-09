//! Structured logging bootstrap.
//!
//! Uses the `tracing` ecosystem so logs have levels, targets, and fields.
//! On Android, stderr is still captured by logcat for native code; set
//! `RUST_LOG` (desktop) to tune verbosity (default: `info`).

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Install a global tracing subscriber. Safe to call once at process start.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Production-friendly default; bump to debug with RUST_LOG=debug.
        EnvFilter::new("info,open_grind_lib=info")
    });

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_ansi(cfg!(not(target_os = "android")))
        .with_writer(std::io::stderr);

    // Ignore error if something else already set a subscriber (tests / hot reload).
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();

    tracing::debug!("tracing subscriber initialized");
}
