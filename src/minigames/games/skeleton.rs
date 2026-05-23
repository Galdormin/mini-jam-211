//! MiniGame: Repair a broken dinosaur's skeleton

use bevy::prelude::*;

use crate::{
    AppSystems,
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped},
        games::{MiniGame, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MiniGame::Skeleton), setup_minigame);
    app.add_systems(
        Update,
        on_bone_dropped
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Skeleton)),
    );
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum BoneType {
    Head,
    Torso,
    Leg,
}

fn setup_minigame(mut commands: Commands) {
    let base = setup_minigame_background(&mut commands, MiniGame::Skeleton);

    // Sample bones
    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(200, 50, 50), vec2(100., 40.)),
        Transform::from_translation(vec3(0., 200., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        DropZone(vec2(100., 100.)),
        BoneType::Head,
        Transform::from_translation(vec3(-200., 200., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(200, 50, 50), vec2(100., 40.)),
        Transform::from_translation(vec3(200., 200., 1.)),
    ));

    commands.spawn((
        ChildOf(base),
        DropZone(vec2(100., 100.)),
        BoneType::Torso,
        Transform::from_translation(vec3(0., 400., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(50, 200, 50), vec2(40., 100.)),
        Transform::from_translation(vec3(-200., 400., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(50, 200, 50), vec2(40., 100.)),
        Transform::from_translation(vec3(200., 400., 1.)),
    ));

    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(50, 50, 200), vec2(100., 100.)),
        Transform::from_translation(vec3(0., 0., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Sprite::from_color(Color::srgb_u8(50, 50, 200), vec2(100., 100.)),
        Transform::from_translation(vec3(-200., 0., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        DropZone(vec2(100., 100.)),
        BoneType::Leg,
        Transform::from_translation(vec3(200., 0., 1.)),
    ));

    // Droppable
    commands.spawn((
        ChildOf(base),
        Draggable,
        BoneType::Head,
        Sprite::from_color(Color::srgb_u8(200, 50, 50), vec2(100., 40.)),
        Transform::from_translation(vec3(-170., -400., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Draggable,
        BoneType::Torso,
        Sprite::from_color(Color::srgb_u8(50, 200, 50), vec2(40., 100.)),
        Transform::from_translation(vec3(20., -400., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Draggable,
        BoneType::Leg,
        Sprite::from_color(Color::srgb_u8(50, 50, 200), vec2(100., 100.)),
        Transform::from_translation(vec3(89., -400., 1.)),
    ));
}

fn on_bone_dropped(
    mut commands: Commands,
    mut dropped: MessageReader<ItemDropped>,
    zones: Query<(&Transform, &BoneType), Without<Draggable>>,
    mut items: Query<(&mut Transform, &BoneType), With<Draggable>>,
) -> Result {
    for item in dropped.read() {
        let (zone_transform, zone_type) = zones.get(item.in_zone)?;
        let (mut item_transform, item_type) = items.get_mut(item.item)?;

        if zone_type != item_type {
            continue;
        }

        commands
            .entity(item.item)
            .try_remove::<Draggable>()
            .try_remove::<Pickable>();
        *item_transform = *zone_transform;
    }

    Ok(())
}
