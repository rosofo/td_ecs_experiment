use std::marker::PhantomData;

use bevy_ecs::{prelude::*, system::SystemParam};
use pyo3::{
    intern,
    types::{PyAnyMethods, PyModule},
    Bound, Py, PyAny, Python,
};
use tracing::{debug, instrument};

/// Queue commands to be run against the [TDApi]
#[derive(SystemParam)]
pub struct TDCommands<'w>(ResMut<'w, TDCommandQueue>);

impl TDCommands<'_> {
    pub fn queue(&mut self, cmd: impl TDCommand) {
        self.0.queue.push(Box::new(cmd));
    }
}

#[derive(Resource)]
pub struct TDCommandQueue {
    pub queue: Vec<Box<dyn TDCommand>>,
}

#[derive(Event)]
pub struct TDCommandDeferred(Box<dyn TDCommand>);

pub trait TDCommand: BoxTDCommand + Send + Sync + 'static {
    fn apply(self, world: &mut World, api: &TDApi);
}

/// Solution for calling apply on Box<dyn TDCommand>, because we need to store them before executing
/// Courtesy of https://users.rust-lang.org/t/calling-a-method-with-a-self-parameter-on-box-dyn-trait/106220/3
pub trait BoxTDCommand: Send + Sync + 'static {
    fn apply_boxed(self: Box<Self>, world: &mut World, api: &TDApi);
}

impl<C: TDCommand> BoxTDCommand for C {
    fn apply_boxed(self: Box<Self>, world: &mut World, api: &TDApi) {
        (*self).apply(world, api);
    }
}

impl<C: TDCommand + ?Sized> TDCommand for Box<C> {
    fn apply(self, world: &mut World, api: &TDApi) {
        self.apply_boxed(world, api);
    }
}

/// Bindings to the TD python API
///
/// ## Note
///
/// We have to acquire the GIL at some point to talk to TD, and that point is here!
pub struct TDApi<'py> {
    td: Bound<'py, PyModule>,
}

impl<'py> TDApi<'py> {
    pub fn new(py: Python<'py>) -> Self {
        let td = py.import(intern!(py, "td")).unwrap();
        Self { td }
    }

    pub fn op(&self, path: &str) -> TDApiOp {
        let op = self
            .td
            .call_method1(intern!(self.td.py(), "op"), (path,))
            .unwrap();
        debug!("got op {op}");
        TDApiOp { op }
    }
}

pub struct TDApiOp<'py> {
    op: Bound<'py, PyAny>,
}

#[instrument(skip(world, api))]
pub fn apply_deferred_td(world: &mut World, api: &TDApi) {
    let mut commands = world.get_resource_mut::<TDCommandQueue>().unwrap();
    let len = commands.queue.len();
    debug!("appying {len} commands");
    for command in commands.queue.drain(..).collect::<Vec<_>>() {
        command.apply(world, api);
    }
}
