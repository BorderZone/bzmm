use std::path::Path;
use std::sync::OnceLock;
use tauri::Manager;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    fmt, layer::SubscriberExt, prelude::*, util::SubscriberInitExt, EnvFilter,
};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init_logging(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = app.path().app_log_dir()?;
    std::fs::create_dir_all(&log_dir)?;
    let (writer, guard) = create_writer(&log_dir, "bzmm.log");
    let _ = LOG_GUARD.set(guard);

    let stdout_filter = EnvFilter::new("debug");
    let file_filter = EnvFilter::new("info");
    let stdout_layer = fmt::layer()
        .pretty()
        .with_writer(std::io::stdout)
        .with_filter(stdout_filter);

    let file_layer = tracing_tree::HierarchicalLayer::default()
        .with_ansi(false)
        .with_indent_lines(true)
        .with_verbose_entry(true)
        .with_targets(true)
        .with_writer(writer)
        .with_filter(file_filter);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    Ok(())
}

fn create_writer(
    directory: impl AsRef<Path>,
    name: impl AsRef<Path>,
) -> (NonBlocking, WorkerGuard) {
    let appender = tracing_appender::rolling::never(directory, name);
    tracing_appender::non_blocking(appender)
}
