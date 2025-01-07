use bevy_ecs::prelude::*;
use pyo3::prelude::*;

use crate::{op::Op, touchdesigner::{TDCommand, TDCommands}};

#[pyclass]
#[derive(Component, Clone)]
pub struct Random;

struct RandomCommand {op: Entity};

impl TDCommand for RandomCommand {
    fn apply(self, world: &mut World, api: &crate::touchdesigner::TDApi) {
        let path = world.query::<&Op>().get(world, self.op).unwrap().path.as_str();
        let pars = api.op(path).pars;
    }
}

pub fn randomize_pars(query: Query<(&Random, &Op)>, mut cmd: TDCommands) {
    for (random, op) in query.iter() {
        cmd.queue(RandomCommand);
    }
}
