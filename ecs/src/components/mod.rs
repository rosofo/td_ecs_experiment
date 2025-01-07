mod random;
use bevy_ecs::prelude::*;
use pyo3::prelude::*;
pub use random::*;

#[pyclass]
#[derive(Clone)]
pub enum OpComponent {
    Random(random::Random),
}
