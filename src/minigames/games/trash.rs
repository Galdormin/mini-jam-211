//! MiniGame: Trash all the trashes

use bevy::{audio::Volume, prelude::*};
use rand::{Rng, seq::IndexedRandom};

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    audio::{SoundEffect, sound_effect_random_speed},
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped},
        games::{MiniGame, MinigameFinished, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TrashAssets>();

    app.add_systems(OnEnter(MiniGame::Trash), setup_minigame);
    app.add_systems(OnExit(MiniGame::Trash), cleanup_minigame);
    app.add_systems(
        Update,
        (on_item_dropped_in_bin, check_completion, update_bin_sprite)
            .chain()
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Trash)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TrashAssets {
    #[dependency]
    bins: [Handle<Image>; 4],
    #[dependency]
    trashes: Vec<Handle<Image>>,
    #[dependency]
    trash_sounds: Handle<AudioSource>,
}

impl FromWorld for TrashAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            bins: [
                assets.load("images/minigames/trash/bin_0_3.png"),
                assets.load("images/minigames/trash/bin_1_3.png"),
                assets.load("images/minigames/trash/bin_2_3.png"),
                assets.load("images/minigames/trash/bin_3_3.png"),
            ],
            trashes: vec![
                assets.load("images/minigames/trash/trash_1.png"),
                assets.load("images/minigames/trash/trash_2.png"),
                assets.load("images/minigames/trash/trash_3.png"),
                assets.load("images/minigames/trash/trash_4.png"),
                assets.load("images/minigames/trash/trash_5.png"),
                assets.load("images/minigames/trash/trash_6.png"),
            ],
            trash_sounds: assets.load("audio/sound_effects/paper_sound.mp3"),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Trash;

#[derive(Component, Clone, Debug, Default)]
pub struct Bin;

#[derive(Resource, Clone, Debug, Default)]
struct TrashCount {
    total: u32,
    remaining: u32,
}

impl TrashCount {
    fn new(amount: u32) -> Self {
        TrashCount {
            total: amount,
            remaining: amount,
        }
    }

    fn percentage(&self) -> f32 {
        (self.total - self.remaining) as f32 / self.total as f32
    }
}

fn cleanup_minigame(mut commands: Commands) {
    commands.remove_resource::<TrashCount>();
}

fn setup_minigame(mut commands: Commands, trash_assets: Res<TrashAssets>) {
    let n_trash = rand::rng().random_range(4_u32..=8_u32);

    commands.insert_resource(TrashCount::new(n_trash));

    let base = setup_minigame_background(&mut commands, MiniGame::Trash);

    // Bin
    commands.spawn((
        ChildOf(base),
        Bin,
        Sprite::from_image(trash_assets.bins[0].clone()),
        Transform::from_translation(vec3(-200., 0., 1.)).with_scale(Vec3::splat(0.4)),
        children![(
            DropZone(vec2(300., 160.)),
            Transform::from_translation(vec3(0., 240., 0.1)),
        )],
    ));

    // Trash
    for _ in 0..n_trash {
        let mut rng = rand::rng();

        let pos_x = rng.random_range(-800.0..=800.0);
        let pos_y = rng.random_range(-350.0..=-150.0);
        let image = trash_assets.trashes.choose(&mut rng).unwrap().clone();

        commands.spawn((
            ChildOf(base),
            Trash,
            Draggable,
            Sprite::from_image(image),
            Transform::from_translation(vec3(pos_x, pos_y, 1.)).with_scale(Vec3::splat(0.4)),
        ));
    }
}

fn on_item_dropped_in_bin(
    mut commands: Commands,
    mut remaining: ResMut<TrashCount>,
    trash_assets: Res<TrashAssets>,
    mut dropped: MessageReader<ItemDropped>,
) {
    for item in dropped.read() {
        commands.entity(item.item).despawn();
        remaining.remaining -= 1;

        // Play sfx
        commands.spawn(sound_effect_random_speed(
            trash_assets.trash_sounds.clone(),
            0.9..1.3,
        ));
    }
}

fn update_bin_sprite(
    trash_count: Res<TrashCount>,
    trash_assets: Res<TrashAssets>,
    mut bin: Single<&mut Sprite, With<Bin>>,
) {
    if !trash_count.is_changed() {
        return;
    }

    bin.image = match trash_count.percentage() {
        0.0..0.33 => trash_assets.bins[0].clone(),
        0.33..0.66 => trash_assets.bins[1].clone(),
        0.66..0.99 => trash_assets.bins[2].clone(),
        1.0 => trash_assets.bins[3].clone(),
        _ => unreachable!(),
    };
}

fn check_completion(trash_count: Res<TrashCount>, mut finished: MessageWriter<MinigameFinished>) {
    if trash_count.is_changed() && trash_count.remaining == 0 {
        finished.write(MinigameFinished {
            game: MiniGame::Trash,
        });
    }
}
