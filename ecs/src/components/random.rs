use bevy_ecs::prelude::*;
use pyo3::prelude::*;
use tracing::{debug, field::debug, instrument};

use crate::{
    op::Op,
    touchdesigner::{TDCommand, TDCommands},
};

#[pyclass]
#[derive(Component, Clone)]
pub struct Random;

struct RandomCommand {
    op: Entity,
}

impl TDCommand for RandomCommand {
    fn apply(self, world: &mut World, api: &crate::touchdesigner::TDApi) {
        let path = world
            .query::<&Op>()
            .get(world, self.op)
            .unwrap()
            .path
            .as_str();
        let pars = api.op(path);
    }
}

#[instrument(skip(query, cmd))]
pub fn randomize_pars(query: Query<(Entity, &Random)>, mut cmd: TDCommands) {
    for (entity, random) in query.iter() {
        debug!("Queuing random command for {entity}");
        cmd.queue(RandomCommand { op: entity });
    }
}
