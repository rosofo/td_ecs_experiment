use bevy_ecs::prelude::*;
use pyo3::prelude::*;

#[derive(Component)]
pub struct Op {
    pub id: u32,
}
