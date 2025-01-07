use bevy_ecs::{
    component::{ComponentInfo, Components},
    prelude::*,
    schedule::{InternedScheduleLabel, ScheduleLabel},
    world::CommandQueue,
};
use pyo3::prelude::*;

use crate::{
    components::OpComponent,
    op::Op,
    schedule::{PostUpdate, Update},
    touchdesigner::apply_deferred_td,
};

#[pyclass(name = "World")]
pub struct PyWorld {
    world: World,
    update: InternedScheduleLabel,
    post_update: InternedScheduleLabel,
}
impl PyWorld {
    fn id(&mut self, path: &str) -> Entity {
        self.world
            .query::<(Entity, &Op)>()
            .iter(&self.world)
            .find_map(|(e, op)| if op.path == path { Some(e) } else { None })
            .unwrap_or_else(|| {
                self.world
                    .spawn(Op {
                        path: path.to_string(),
                    })
                    .id()
            })
    }
}

#[pymethods]
impl PyWorld {
    #[new]
    fn new() -> Self {
        let update = Schedule::new(Update);
        let update_label = update.label();
        let mut post_update = Schedule::new(PostUpdate);
        let post_update_label = post_update.label();
        post_update.add_systems(apply_deferred_td);

        let mut world = World::new();
        world.add_schedule(update);
        world.add_schedule(post_update);
        Self {
            world,
            update: update_label,
            post_update: post_update_label,
        }
    }
    fn insert(&mut self, path: &str, component: OpComponent) {
        let entity = self.id(path);
        match component {
            OpComponent::Random(comp) => self.world.commands().entity(entity).insert(comp),
        };
    }

    fn run(&mut self) {
        self.world.run_schedule(self.update);
        self.world.run_schedule(self.post_update);
    }
}