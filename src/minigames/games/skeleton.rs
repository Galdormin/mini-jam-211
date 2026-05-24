//! MiniGame: Repair a broken dinosaur's skeleton

use bevy::prelude::*;
use rand::Rng;

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped},
        games::{MiniGame, MinigameFinished, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SkeletonAssets>();
    app.add_systems(OnEnter(MiniGame::Skeleton), setup_minigame);
    app.add_systems(OnExit(MiniGame::Skeleton), cleanup_minigame);
    app.add_systems(
        Update,
        (on_bone_dropped, check_completion.after(on_bone_dropped))
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Skeleton)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SkeletonAssets {
    #[dependency]
    head: Handle<Image>,
    #[dependency]
    body: Handle<Image>,
    #[dependency]
    tail: Handle<Image>,
    #[dependency]
    front: Handle<Image>,
}

impl FromWorld for SkeletonAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            head: assets.load("images/minigames/skeleton/skeleton_head.png"),
            body: assets.load("images/minigames/skeleton/skeleton_body.png"),
            tail: assets.load("images/minigames/skeleton/skeleton_tail.png"),
            front: assets.load("images/minigames/skeleton/skeleton_front.png"),
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
struct RemainingBones(u32);

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum BoneType {
    Head,
    Body,
    Tail,
    Front,
}

fn cleanup_minigame(mut commands: Commands) {
    commands.remove_resource::<RemainingBones>();
}

fn setup_minigame(mut commands: Commands, assets: Res<SkeletonAssets>) {
    let base = setup_minigame_background(&mut commands, MiniGame::Skeleton);

    // (bone_type, image, zone_position, scale, zone_size)
    let bones = [
        (
            BoneType::Head,
            assets.head.clone(),
            vec3(-423., 49., 1.5),
            0.5,
            vec2(295., 215.),
        ),
        (
            BoneType::Body,
            assets.body.clone(),
            vec3(10., 55., 1.5),
            0.55,
            vec2(355., 200.),
        ),
        (
            BoneType::Tail,
            assets.tail.clone(),
            vec3(414., 14., 1.5),
            0.45,
            vec2(295., 90.),
        ),
        (
            BoneType::Front,
            assets.front.clone(),
            vec3(-80., -170., 1.5),
            0.45,
            vec2(240., 95.),
        ),
    ];

    // Randomly pre-place 0 or 1 bone
    let preplaced_idx: Option<usize> = if rand::rng().random_bool(0.5) {
        Some(rand::rng().random_range(0..bones.len()))
    } else {
        None
    };

    let remaining = bones.len() - preplaced_idx.is_some() as usize;
    commands.insert_resource(RemainingBones(remaining as u32));

    for (i, (bone_type, image, pos, scale, zone_size)) in bones.iter().enumerate() {
        if preplaced_idx == Some(i) {
            // Already placed: full opacity at zone position, no interaction needed
            commands.spawn((
                ChildOf(base),
                Sprite::from_image(image.clone()),
                Transform::from_translation(*pos).with_scale(Vec3::splat(*scale)),
            ));
            continue;
        }

        // Ghost: dim reference showing where the bone belongs
        commands.spawn((
            ChildOf(base),
            Sprite {
                image: image.clone(),
                color: Color::srgba(1., 1., 1., 0.25),
                ..default()
            },
            Transform::from_translation(pos.with_z(1.0)).with_scale(Vec3::splat(*scale)),
        ));

        // Drop zone (invisible, at zone z so snapped bone lands above ghost)
        commands.spawn((
            ChildOf(base),
            DropZone(*zone_size),
            bone_type.clone(),
            Transform::from_translation(*pos).with_scale(Vec3::splat(*scale)),
        ));

        // Draggable bone at random position along the bottom
        let drag_pos = vec3(
            rand::rng().random_range(-600.0..600.0),
            rand::rng().random_range(-480.0..-300.0),
            2.0,
        );
        commands.spawn((
            ChildOf(base),
            Draggable,
            bone_type.clone(),
            Sprite::from_image(image.clone()),
            Transform::from_translation(drag_pos).with_scale(Vec3::splat(*scale)),
        ));
    }
}

fn on_bone_dropped(
    mut commands: Commands,
    mut dropped: MessageReader<ItemDropped>,
    mut remaining_bones: ResMut<RemainingBones>,
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
        item_transform.translation = zone_transform.translation;

        remaining_bones.0 -= 1;
    }

    Ok(())
}

fn check_completion(remaining: Res<RemainingBones>, mut finished: MessageWriter<MinigameFinished>) {
    if remaining.is_changed() && remaining.0 == 0 {
        finished.write(MinigameFinished {
            game: MiniGame::Skeleton,
        });
    }
}
