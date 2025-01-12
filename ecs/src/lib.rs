mod components;
mod op;
mod touchdesigner;
mod world;

use components::Random;
use pyo3::prelude::*;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use world::PyWorld;

/// A Python module implemented in Rust.
#[pymodule]
fn ecs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let appender = tracing_appender::rolling::never("logs", "bevy.log");
    tracing_subscriber::Registry::default()
        .with(tracing_tracy::TracyLayer::default())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(appender)
                .with_filter(LevelFilter::INFO),
        )
        .init();

    m.add_class::<PyWorld>()?;
    m.add_class::<Random>()?;
    Ok(())
}
