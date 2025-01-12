use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use pyo3::types::PyAnyMethods;

use crate::{
    op::Op,
    touchdesigner::{TDCommand, TDCommands},
};

#[derive(Component, Debug, Clone)]
#[require(SampleValues)]
pub struct Sample {
    pub timer: Timer,
    pub filter: String,
}

#[derive(Component, Debug, Default)]
pub struct SampleValues {
    pub values: HashMap<String, f32>,
}

impl Sample {
    pub fn new(interval: Duration, filter: String) -> Self {
        let timer = Timer::new(interval, bevy::time::TimerMode::Repeating);
        Self { timer, filter }
    }
}

#[derive(Debug)]
struct SampleCommand(Entity);

impl TDCommand for SampleCommand {
    fn apply(self, world: &mut World, api: &crate::touchdesigner::TDApi) {
        let (op, sample, mut values) = world
            .query::<(&Op, &Sample, &mut SampleValues)>()
            .get_mut(world, self.0)
            .unwrap();
        let binding = api.op(op.id);
        let iter = binding.pars();
        values.values.extend(
            iter.filter(|par| par.name.contains(&sample.filter))
                .filter_map(|par| Some((par.name, par.value.extract().ok()?))),
        );
    }
}

pub fn sample_ops(mut ops: Query<(Entity, &mut Sample)>, mut cmd: TDCommands, time: Res<Time>) {
    for (op, mut sample) in ops.iter_mut() {
        sample.timer.tick(time.delta());
        if sample.timer.finished() {
            cmd.queue(SampleCommand(op));
        }
    }
}
