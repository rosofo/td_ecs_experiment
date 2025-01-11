use std::{any::Any, sync::Arc};

use bevy::{
    app::ScheduleRunnerPlugin,
    log::LogPlugin,
    prelude::*,
    remote::{http::RemoteHttpPlugin, RemotePlugin},
};
use pyo3::prelude::*;
use tracing::{debug, instrument};

use crate::{
    components::{randomize_pars, OpComponent},
    op::Op,
    touchdesigner::{apply_deferred_td, TDApi, TDCommandQueue},
};

#[pyclass(name = "World")]
pub struct PyWorld {
    app: App,
}
unsafe impl Sync for PyWorld {}
unsafe impl Send for PyWorld {}

impl PyWorld {
    #[instrument(skip(self))]
    fn id(&mut self, td_id: u32) -> Entity {
        debug!("Getting or creating entity for op {}", td_id);
        let world = self.app.world_mut();
        world
            .query::<(Entity, &Op)>()
            .iter(world)
            .find_map(|(e, op)| if op.id == td_id { Some(e) } else { None })
            .unwrap_or_else(|| {
                debug!("Spawning new entity for op: {}", td_id);
                world.spawn(Op { id: td_id }).id()
            })
    }
}

#[pymethods]
impl PyWorld {
    #[new]
    #[instrument]
    fn new() -> Self {
        debug!("Creating new PyWorld instance");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()));
        app.add_plugins(RemotePlugin::default());
        app.add_plugins(RemoteHttpPlugin::default());
        app.add_systems(Update, randomize_pars);
        // app.add_systems(PostUpdate, report_world);
        app.insert_resource(TDCommandQueue { queue: Vec::new() });

        app.finish();
        app.cleanup();

        Self { app }
    }

    #[instrument(skip(self))]
    fn insert(&mut self, td_id: u32, component: OpComponent) {
        debug!("Inserting component for op: {}", td_id);
        let entity = self.id(td_id);
        match component {
            OpComponent::Random(comp) => {
                debug!("Inserting Random component");
                self.app.world_mut().commands().entity(entity).insert(comp)
            }
        };
        self.app.world_mut().flush();
    }

    #[instrument(skip(self))]
    fn remove(&mut self, td_id: u32, component: OpComponent) {
        debug!("Removing component from op: {}", td_id);
        let entity = self.id(td_id);
        match component {
            OpComponent::Random(comp) => {
                debug!("Inserting Random component");
                let id = self
                    .app
                    .world()
                    .components()
                    .get_id(comp.type_id())
                    .unwrap();
                self.app
                    .world_mut()
                    .commands()
                    .entity(entity)
                    .remove_by_id(id);
            }
        };
        self.app.world_mut().flush();
    }

    #[instrument(skip(self))]
    fn despawn(&mut self, td_id: u32) {
        let entity = self.id(td_id);
        self.app.world_mut().commands().entity(entity).despawn();
        self.app.world_mut().flush();
    }

    #[instrument(skip(self))]
    fn run(&mut self) {
        self.app.update();
        Python::with_gil(|py| {
            let api = TDApi::new(py);
            apply_deferred_td(self.app.world_mut(), &api);
        });
    }
}

#[instrument(skip(world))]
fn report_world(world: &mut World) {
    debug!("entities: {:?}", world.entities());
}
