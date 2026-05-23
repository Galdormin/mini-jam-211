//! List of all minigames

use bevy::prelude::*;

mod skeleton;
mod toilet;
mod trash;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<MiniGame>();
    app.add_message::<MinigameFinished>();
    app.add_plugins((skeleton::plugin, toilet::plugin, trash::plugin));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[allow(unused)]
pub enum MiniGame {
    #[default]
    None,
    Cashier,
    PopCorn,
    Skeleton,
    Trash,
    Toilet,
}

impl MiniGame {
    pub fn title(self) -> &'static str {
        match self {
            MiniGame::None => "",
            MiniGame::Cashier => "Caisse",
            MiniGame::PopCorn => "Machine à Pop-corn",
            MiniGame::Skeleton => "Squelette de Spinosaure",
            MiniGame::Trash => "Poubelle",
            MiniGame::Toilet => "Toilettes",
        }
    }
}

/// [`Message`] sent everytime a minigame is finished
#[derive(Message, Clone, Debug)]
#[allow(unused)]
pub struct MinigameFinished {
    pub game: MiniGame,
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
            Sprite::from_color(Color::srgba(0., 0., 0., 0.9), vec2(2000., 2000.)),
            Pickable::default(),
            DespawnOnExit(minigame),
            Transform::from_translation(vec3(0., 0., 1.)),
        ))
        .id()
}
