//! MiniGame: Trash all the trashes

use bevy::prelude::*;

use crate::{
    AppSystems,
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped, LimitedDrag},
        games::{MiniGame, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MiniGame::Trash), setup_minigame);
    app.add_systems(
        Update,
        on_item_dropped_in_bin
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Trash)),
    );
}

fn setup_minigame(mut commands: Commands) {
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

fn on_item_dropped_in_bin(mut dropped: MessageReader<ItemDropped>, mut commands: Commands) {
    for item in dropped.read() {
        commands.entity(item.item).despawn();
    }
}
