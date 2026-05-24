//! Écran de fin — victoire ou défaite

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<OutroAssets>();
    app.add_systems(OnEnter(Screen::Outro(false)), spawn_outro(false));
    app.add_systems(OnEnter(Screen::Outro(true)), spawn_outro(true));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct OutroAssets {
    #[dependency]
    background_win: Handle<Image>,
    #[dependency]
    background_loose: Handle<Image>,
}

impl FromWorld for OutroAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            background_win: assets.load("images/message_win.png"),
            background_loose: assets.load("images/message_loose.png"),
        }
    }
}

fn spawn_outro(win: bool) -> impl Fn(Commands, Res<OutroAssets>) {
    move |mut commands, assets| {
        let screen = Screen::Outro(win);

        // Background
        let image = if win {
            assets.background_win.clone()
        } else {
            assets.background_loose.clone()
        };

        commands.spawn((DespawnOnExit(screen), Sprite::from_image(image)));

        // Boutons en bas à droite
        commands
            .spawn((
                DespawnOnExit(screen),
                GlobalZIndex(2),
                Node {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    bottom: Val::Px(50.),
                    right: Val::Px(50.),
                    row_gap: Val::Px(20.),
                    ..default()
                },
            ))
            .with_children(|parent| {
                if !win {
                    parent.spawn(widget::button("Retry", go_gameplay));
                }
                parent.spawn(widget::button("Menu", go_title));
            });
    }
}

fn go_gameplay(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn go_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
