use bevy::prelude::*;

use crate::touchdesigner::{TDCommand, TDCommands};

pub fn td_queue<C: TDCommand>(In(cmd): In<C>, mut commands: TDCommands) {
    commands.queue(cmd);
}
