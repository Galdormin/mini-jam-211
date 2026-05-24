//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::music, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<MenuAssets>();

    app.add_systems(OnEnter(Screen::Title), (open_main_menu, setup_title));
    app.add_systems(OnExit(Screen::Title), close_menu);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct MenuAssets {
    #[dependency]
    background: Handle<Image>,
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for MenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            background: assets.load("images/menu.png"),
            music: assets.load("audio/music/intro_menu.ogg"),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct MenuBackground;

fn setup_title(mut commands: Commands, menu_assets: Res<MenuAssets>) {
    commands.spawn((
        MenuBackground,
        DespawnOnEnter(Screen::Gameplay),
        Sprite::from_image(menu_assets.background.clone()),
    ));

    commands.spawn((
        Name::new("Menu Music"),
        DespawnOnExit(Screen::Title),
        music(menu_assets.music.clone()),
    ));
}
fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
