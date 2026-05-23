//! MiniGame: Trash all the trashes

use bevy::prelude::*;

use crate::{
    AppSystems,
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped},
        games::{MiniGame, MinigameFinished, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MiniGame::Trash), setup_minigame);
    app.add_systems(OnExit(MiniGame::Trash), cleanup_minigame);
    app.add_systems(
        Update,
        (
            on_item_dropped_in_bin,
            check_completion.after(on_item_dropped_in_bin),
        )
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Trash)),
    );
}

#[derive(Resource, Clone, Debug, Default)]
struct RemainingTrash(u32);

fn cleanup_minigame(mut commands: Commands) {
    commands.remove_resource::<RemainingTrash>();
}

fn setup_minigame(mut commands: Commands) {
    commands.insert_resource(RemainingTrash(3));

    let base = setup_minigame_background(&mut commands, MiniGame::Trash);

    // Bin
    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(125, 145, 138), vec2(200., 360.)),
        Transform::from_translation(vec3(-300., -150., 1.)),
        children![(
            Sprite::from_color(Color::srgb_u8(76, 145, 138), vec2(200., 100.)),
            Transform::from_translation(vec3(0., 180., 0.1)),
            DropZone(vec2(210., 110.))
        )],
    ));

    // Trash
    commands.spawn((
        Draggable,
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(255, 0, 0), vec2(80., 80.)),
        Transform::from_translation(vec3(450., -350., 1.)),
    ));
    commands.spawn((
        Draggable,
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(0, 255, 0), vec2(80., 80.)),
        Transform::from_translation(vec3(250., -260., 1.)),
    ));
    commands.spawn((
        Draggable,
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(0, 0, 255), vec2(80., 80.)),
        Transform::from_translation(vec3(50., -300., 1.)),
    ));
}

fn on_item_dropped_in_bin(
    mut dropped: MessageReader<ItemDropped>,
    mut remaining: ResMut<RemainingTrash>,
    mut commands: Commands,
) {
    for item in dropped.read() {
        commands.entity(item.item).despawn();
        remaining.0 -= 1;
    }
}

fn check_completion(remaining: Res<RemainingTrash>, mut finished: MessageWriter<MinigameFinished>) {
    if remaining.is_changed() && remaining.0 == 0 {
        finished.write(MinigameFinished {
            game: MiniGame::Trash,
        });
    }
}
