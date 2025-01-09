use std::any::Any;

use bevy_ecs::{
    component::{ComponentInfo, Components},
    prelude::*,
    schedule::{InternedScheduleLabel, ScheduleLabel},
    world::CommandQueue,
};
use pyo3::prelude::*;
use tracing::{debug, instrument};

use crate::{
    components::{randomize_pars, OpComponent},
    op::Op,
    schedule::{PostUpdate, Update},
    touchdesigner::{apply_deferred_td, TDApi, TDCommandQueue},
};

#[pyclass(name = "World")]
pub struct PyWorld {
    world: World,
    update: InternedScheduleLabel,
    post_update: InternedScheduleLabel,
}

impl PyWorld {
    #[instrument(skip(self))]
    fn id(&mut self, td_id: u32) -> Entity {
        debug!("Getting or creating entity for op {}", td_id);
        self.world
            .query::<(Entity, &Op)>()
            .iter(&self.world)
            .find_map(|(e, op)| if op.id == td_id { Some(e) } else { None })
            .unwrap_or_else(|| {
                debug!("Spawning new entity for op: {}", td_id);
                self.world.spawn(Op { id: td_id }).id()
            })
    }
}

#[pymethods]
impl PyWorld {
    #[new]
    #[instrument]
    fn new() -> Self {
        debug!("Creating new PyWorld instance");
        let mut update = Schedule::new(Update);
        let update_label = update.label();
        update.add_systems(randomize_pars);
        let mut post_update = Schedule::new(PostUpdate);
        post_update.add_systems(report_world);
        let post_update_label = post_update.label();

        let mut world = World::new();
        world.insert_resource(TDCommandQueue { queue: Vec::new() });

        world.add_schedule(update);
        world.add_schedule(post_update);
        Self {
            world,
            update: update_label,
            post_update: post_update_label,
        }
    }

    #[instrument(skip(self))]
    fn insert(&mut self, td_id: u32, component: OpComponent) {
        debug!("Inserting component for op: {}", td_id);
        let entity = self.id(td_id);
        match component {
            OpComponent::Random(comp) => {
                debug!("Inserting Random component");
                self.world.commands().entity(entity).insert(comp)
            }
        };
        self.world.flush();
    }

    #[instrument(skip(self))]
    fn remove(&mut self, td_id: u32, component: OpComponent) {
        debug!("Removing component from op: {}", td_id);
        let entity = self.id(td_id);
        match component {
            OpComponent::Random(comp) => {
                debug!("Inserting Random component");
                let id = self.world.components().get_id(comp.type_id()).unwrap();
                self.world.commands().entity(entity).remove_by_id(id);
            }
        };
        self.world.flush();
    }

    #[instrument(skip(self))]
    fn despawn(&mut self, td_id: u32) {
        let entity = self.id(td_id);
        self.world.commands().entity(entity).despawn();
        self.world.flush();
    }

    #[instrument(skip(self))]
    fn run(&mut self) {
        debug!("Running schedules");
        self.world.run_schedule(self.update);
        self.world.run_schedule(self.post_update);

        Python::with_gil(|py| {
            let api = TDApi::new(py);
            apply_deferred_td(&mut self.world, &api);
        });
    }
}

#[instrument(skip(world))]
fn report_world(world: &mut World) {
    debug!("entities: {:?}", world.entities());
}
