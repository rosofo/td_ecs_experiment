use std::{cmp::Ordering, collections::HashMap};

use bevy::prelude::*;
use iterstats::Iterstats;
use itertools::*;
use pyo3::IntoPyObjectExt;
use tracing::instrument;
use tracing_subscriber::field::debug;

use crate::{
    op::Op,
    touchdesigner::{TDCommand, TDCommands},
};

use super::{SampleValues, TDComponent};

#[derive(Component)]
pub struct Apply {
    pub filter: String,
    pub strat: Strat,
}

impl TDComponent for Apply {
    fn plugin(app: &mut App) {
        app.add_systems(Update, select);
    }
}

pub enum Strat {
    Mean,
    Max,
}

#[derive(Debug)]
pub struct ApplyCommand {
    id: u32,
    vals: HashMap<String, f32>,
}

impl TDCommand for ApplyCommand {
    #[instrument(skip(self, _world, api))]
    fn apply(self, _world: &mut World, api: &crate::touchdesigner::TDApi) {
        let op = api.op(self.id);
        let py = api.py();
        for (name, val) in self.vals.into_iter() {
            debug!("Setting par {} to {}", name, val);
            op.set_par(name, &val.into_bound_py_any(py).unwrap())
                .unwrap();
        }
    }
}

#[instrument(skip(samplers, appliers, cmd))]
fn select(samplers: Query<&SampleValues>, appliers: Query<(&Op, &Apply)>, mut cmd: TDCommands) {
    for (op, apply) in appliers.iter() {
        debug!("Collecting previously sampled values");
        let vals = samplers
            .iter()
            .flat_map(|values| values.values.iter())
            .filter(|(key, _)| key.contains(&apply.filter))
            .map(|(key, val)| (key.clone(), *val))
            .into_grouping_map();
        let vals = match apply.strat {
            Strat::Max => vals.max_by(|_, a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)),
            Strat::Mean => {
                let vals = vals.collect::<Vec<f32>>();
                HashMap::from_iter(
                    vals.into_iter()
                        .map(|(key, vals)| (key, vals.into_iter().mean())),
                )
            }
        };
        debug!("Queuing ApplyCommand for op {}", op.id);
        cmd.queue(ApplyCommand { id: op.id, vals });
    }
}
