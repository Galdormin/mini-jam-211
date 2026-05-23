//! MiniGames

use bevy::prelude::*;

use crate::{AppSystems, screens::Screen};

pub(crate) mod behaviour;
mod games;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<MiniGame>();
    app.add_plugins((behaviour::plugin, games::plugin));
    app.add_systems(Update, on_add_start_on_click.in_set(AppSystems::Update));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum MiniGame {
    #[default]
    None,
    Skeleton,
    Trash,
    Other,
}

/// Start MiniGame on click
#[derive(Component, Clone, Debug, Deref)]
#[require(Pickable)]
pub struct StartOnClick(MiniGame);

/// Setup a test level to try minigames
pub(super) fn spawn_minigames_selection(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(45, 45, 45), vec2(2000., 2000.)),
        DespawnOnExit(Screen::Gameplay),
        children![
            (
                Sprite::from_color(Color::srgb_u8(249, 231, 231), vec2(70., 70.)),
                StartOnClick(MiniGame::Skeleton),
                Transform::from_translation(vec3(-250., 250., 0.1))
            ),
            (
                Sprite::from_color(Color::srgb_u8(76, 147, 138), vec2(70., 70.)),
                StartOnClick(MiniGame::Trash),
                Transform::from_translation(vec3(0., 250., 0.1))
            ),
            (
                Sprite::from_color(Color::srgb_u8(200, 30, 200), vec2(70., 70.)),
                StartOnClick(MiniGame::Other),
                Transform::from_translation(vec3(250., 250., 0.1))
            )
        ],
    ));
}

/// Add observer to [`StartOnClick`]
fn on_add_start_on_click(mut commands: Commands, starts: Query<Entity, Added<StartOnClick>>) {
    for start in starts {
        commands.entity(start).observe(on_click_start_on_click);
    }
}

/// Change [`MiniGame`] when [`StartOnClick`] is clicked
fn on_click_start_on_click(
    event: On<Pointer<Click>>,
    mut minigame: ResMut<NextState<MiniGame>>,
    starts: Query<&StartOnClick>,
) -> Result {
    minigame.set(**starts.get(event.entity)?);
    Ok(())
}
