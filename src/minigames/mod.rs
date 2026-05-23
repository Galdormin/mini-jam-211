//! MiniGames

use bevy::prelude::*;

pub mod behaviour;
mod games;
pub mod level;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((behaviour::plugin, games::plugin, level::plugin));
}
