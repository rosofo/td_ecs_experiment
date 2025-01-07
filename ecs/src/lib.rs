mod components;
mod op;
mod schedule;
mod touchdesigner;
mod world;

use components::{OpComponent, Random};
use pyo3::prelude::*;
use world::PyWorld;

/// A Python module implemented in Rust.
#[pymodule]
fn ecs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyWorld>()?;
    m.add_class::<OpComponent>()?;
    m.add_class::<Random>()?;
    Ok(())
}
