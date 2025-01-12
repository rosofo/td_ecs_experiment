use bevy::prelude::*;
use pyo3::{intern, prelude::*, IntoPyObjectExt};
use rand::{thread_rng, Rng};
use tracing::{debug, field::debug, instrument};

use crate::{
    commands::td_queue,
    op::Op,
    touchdesigner::{TDCommand, TDCommands},
};

use super::TDComponent;

#[pyclass]
#[derive(Component, Debug, Clone)]
pub struct Random;

impl TDComponent for Random {
    fn plugin(app: &mut bevy::prelude::App) {
        app.add_systems(Update, randomize_pars.pipe(td_queue));
    }
}

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

#[instrument(skip(query))]
pub fn randomize_pars(query: Query<(Entity, &Random)>) -> Vec<Box<dyn TDCommand>> {
    query
        .iter()
        .map(|(entity, _)| {
            debug!("Queuing random command for {entity}");
            let cmd: Box<dyn TDCommand> = Box::from(RandomCommand { op: entity });
            cmd
        })
        .collect::<Vec<Box<dyn TDCommand>>>()
}
