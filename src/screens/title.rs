//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::music, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<MenuAssets>();

    app.add_systems(OnEnter(Screen::Title), (open_main_menu, start_music));
    app.add_systems(OnExit(Screen::Title), close_menu);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct MenuAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for MenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/intro_menu.ogg"),
        }
    }
}

fn start_music(mut commands: Commands, credits_music: Res<MenuAssets>) {
    commands.spawn((
        Name::new("Menu Music"),
        DespawnOnExit(Screen::Title),
        music(credits_music.music.clone()),
    ));
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
