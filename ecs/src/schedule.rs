use bevy_ecs::{prelude::*, schedule::ScheduleLabel};

#[derive(ScheduleLabel, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct Update;
#[derive(ScheduleLabel, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct PostUpdate;
