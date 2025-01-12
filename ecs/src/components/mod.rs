mod random;
use bevy::prelude::*;
pub use random::*;
mod sample;
pub use sample::*;
mod apply;
pub use apply::*;

use crate::{commands::td_queue, touchdesigner::TDCommand};

pub trait TDComponent {
    fn plugin(app: &mut App);
}

pub fn plugin(app: &mut App) {
    app.add_plugins(Random::plugin);
    app.add_plugins(Sample::plugin);
    app.add_plugins(Apply::plugin);
}
