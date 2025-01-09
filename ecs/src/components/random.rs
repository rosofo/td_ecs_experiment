use bevy_ecs::prelude::*;
use pyo3::{intern, prelude::*, IntoPyObjectExt};
use rand::{thread_rng, Rng};
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
        let id = world.query::<&Op>().get(world, self.op).unwrap().id;
        let op = api.op(id);
        let py = api.py();
        op.set_par(
            intern!(py, "gain"),
            &thread_rng()
                .gen_range(0.0..2.0)
                .into_bound_py_any(py)
                .unwrap(),
        )
        .unwrap();
    }
}

#[instrument(skip(query, cmd))]
pub fn randomize_pars(query: Query<(Entity, &Random)>, mut cmd: TDCommands) {
    for (entity, random) in query.iter() {
        debug!("Queuing random command for {entity}");
        cmd.queue(RandomCommand { op: entity });
    }
}
