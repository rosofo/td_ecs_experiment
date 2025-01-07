use std::marker::PhantomData;

use bevy_ecs::{prelude::*, system::SystemParam};

type NotSendSync = PhantomData<*const ()>;

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
/// ## Notes
///
/// - We have to acquire the GIL at some point to talk to TD, and that point is here!
/// - Implemented as a non-send resource because most TD methods aren't threadsafe
#[derive(Clone, Copy)]
pub struct TDApi {
    _non_send_and_sync: NotSendSync,
}

impl TDApi {
    pub fn op(path: &str) -> TDApiOp {}
}

pub fn apply_deferred_td(world: &mut World) {
    let api = *world.get_non_send_resource::<TDApi>().unwrap();
    let mut commands = world.get_resource_mut::<TDCommandQueue>().unwrap();
    for command in commands.queue.drain(..).collect::<Vec<_>>() {
        command.apply(world, &api);
    }
}
