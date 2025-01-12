use std::{any::Any, sync::Arc, time::Duration};

use bevy::{
    app::ScheduleRunnerPlugin,
    log::LogPlugin,
    prelude::*,
    remote::{http::RemoteHttpPlugin, RemotePlugin},
};
use pyo3::prelude::*;
use tracing::{debug, instrument};

use crate::{
    components::{randomize_pars, sample_ops, Random, Sample},
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
    #[instrument(skip(self, component))]
    fn insert(&mut self, td_id: u32, component: impl Component) {
        debug!("Inserting component for op: {}", td_id);
        let entity = self.id(td_id);
        self.app
            .world_mut()
            .commands()
            .entity(entity)
            .insert(component);
        self.app.world_mut().flush();
    }

    #[instrument(skip(self))]
    fn remove<C: Component>(&mut self, td_id: u32) {
        debug!("Removing component from op: {}", td_id);
        let entity = self.id(td_id);
        self.app.world_mut().commands().entity(entity).remove::<C>();
        self.app.world_mut().flush();
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
        app.add_systems(Update, (randomize_pars, sample_ops));
        // app.add_systems(PostUpdate, report_world);
        app.insert_resource(TDCommandQueue { queue: Vec::new() });

        app.finish();
        app.cleanup();

        Self { app }
    }

    #[instrument(skip(self))]
    fn despawn(&mut self, td_id: u32) {
        let entity = self.id(td_id);
        self.app.world_mut().commands().entity(entity).despawn();
        self.app.world_mut().flush();
    }

    fn insert_random(&mut self, td_id: u32) {
        self.insert(td_id, Random);
    }

    fn remove_random(&mut self, td_id: u32) {
        self.remove::<Random>(td_id);
    }
    fn insert_sample(&mut self, td_id: u32, millis: u64, filter: String) {
        self.insert(td_id, Sample::new(Duration::from_millis(millis), filter));
    }

    fn remove_sample(&mut self, td_id: u32) {
        self.remove::<Sample>(td_id);
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
