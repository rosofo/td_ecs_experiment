use bevy_ecs::prelude::*;
use pyo3::prelude::*;
use tracing::{debug, field::debug, instrument};

use crate::{
    op::Op,
    touchdesigner::{TDCommand, TDCommands},
};

#[pyclass]
#[derive(Component, Debug, Clone)]
pub struct Random;

#[derive(Debug)]
struct RandomCommand {
    op: Entity,
}

impl TDCommand for RandomCommand {
    #[instrument]
    fn apply(self, world: &mut World, api: &crate::touchdesigner::TDApi) {
        debug!("Applying RandomCommand");
        let path = world
            .query::<&Op>()
            .get(world, self.op)
            .unwrap()
            .path
            .as_str();
        let pars = api.op(path);
        debug!("RandomCommand applied to path: {}", path);
    }
}

#[instrument(skip(query, cmd))]
pub fn randomize_pars(query: Query<(Entity, &Random)>, mut cmd: TDCommands) {
    for (entity, random) in query.iter() {
        debug!("Queuing random command for {entity}");
        cmd.queue(RandomCommand { op: entity });
    }
}
