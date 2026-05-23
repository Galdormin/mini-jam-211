//! List of all minigames

use bevy::prelude::*;

use crate::minigames::MiniGame;

mod skeleton;
mod trash;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((skeleton::plugin, trash::plugin));
}

pub(super) fn setup_minigame_background(commands: &mut Commands, minigame: MiniGame) -> Entity {
    commands
        .spawn((
            Button,
            DespawnOnExit(minigame),
            Node {
                position_type: PositionType::Absolute,
                right: px(50),
                top: px(50),
                width: px(50),
                height: px(50),
                border_radius: BorderRadius::all(px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            children![(
                Text::new("X"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        ))
        .observe(
            |_: On<Pointer<Click>>, mut next_state: ResMut<NextState<MiniGame>>| {
                next_state.set(MiniGame::None)
            },
        );

    commands
        .spawn((
            Sprite::from_color(Color::srgba(0., 0., 0., 0.6), vec2(2000., 2000.)),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            DespawnOnExit(minigame),
            Transform::from_translation(vec3(0., 0., 1.)),
        ))
        .id()
}
