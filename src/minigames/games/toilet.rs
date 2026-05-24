//! MiniGame: Clean the toilets

use bevy::prelude::*;
use rand::{Rng as _, seq::SliceRandom as _};

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped, LimitedDrag},
        games::{MiniGame, MinigameFinished, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<ToiletAssets>();
    app.add_systems(OnEnter(MiniGame::Toilet), setup_minigame);
    app.add_systems(OnExit(MiniGame::Toilet), cleanup_minigame);
    app.add_systems(
        Update,
        (
            on_ventouse_dropped,
            update_ventouse_count,
            update_toilet_sprite,
            check_completion.after(update_ventouse_count),
        )
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Toilet)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ToiletAssets {
    #[dependency]
    toilet_clean: Handle<Image>,
    #[dependency]
    toilet_dirty: Handle<Image>,
    #[dependency]
    ventouse: Handle<Image>,
    #[dependency]
    flush_sfx: Handle<AudioSource>,
}

impl FromWorld for ToiletAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            toilet_clean: assets.load("images/minigames/toilet/toilet_clean.png"),
            toilet_dirty: assets.load("images/minigames/toilet/toilet_dirty.png"),
            ventouse: assets.load("images/minigames/toilet/ventouse.png"),
            flush_sfx: assets.load("audio/sound_effects/toilet_flush.ogg"),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Toilet(pub bool);

#[derive(Component, Clone, Debug)]
pub struct Ventouse {
    toilet: Option<Entity>,
    count: u32,
    direction: bool,
}

impl Default for Ventouse {
    fn default() -> Self {
        Self {
            toilet: None,
            count: 5,
            direction: false,
        }
    }
}

impl Ventouse {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Resource, Clone, Debug, Default)]
struct RemainingToilets(u32);

fn cleanup_minigame(mut commands: Commands) {
    commands.remove_resource::<RemainingToilets>();
}

fn setup_minigame(mut commands: Commands, assets: Res<ToiletAssets>) {
    let n_dirty = rand::rng().random_range(1..=3_usize);
    let mut indices = [0usize, 1, 2];
    indices.shuffle(&mut rand::rng());
    let dirty: &[usize] = &indices[..n_dirty];

    commands.insert_resource(RemainingToilets(n_dirty as u32));

    let base = setup_minigame_background(&mut commands, MiniGame::Toilet);

    let positions = [
        vec3(-430., -50., 1.),
        vec3(0., -50., 1.),
        vec3(430., -50., 1.),
    ];

    for (i, pos) in positions.iter().enumerate() {
        let is_dirty = dirty.contains(&i);
        let image = if is_dirty {
            assets.toilet_dirty.clone()
        } else {
            assets.toilet_clean.clone()
        };

        let mut entity = commands.spawn((
            ChildOf(base),
            Toilet(!is_dirty),
            Sprite::from_image(image),
            Transform::from_translation(*pos).with_scale(Vec3::splat(0.35)),
        ));

        if is_dirty {
            entity.insert(DropZone(vec2(160., 200.)));
        }
    }

    // Ventouse
    commands.spawn((
        ChildOf(base),
        Ventouse::default(),
        Draggable,
        Sprite::from_image(assets.ventouse.clone()),
        Transform::from_translation(vec3(-680., -100., 1.1)).with_scale(Vec3::splat(0.5)),
    ));
}

fn on_ventouse_dropped(
    mut commands: Commands,
    mut dropped: MessageReader<ItemDropped>,
    toilets: Query<&Transform, (With<Toilet>, Without<Draggable>)>,
    mut ventouses: Query<(&mut Transform, &mut Ventouse), With<Draggable>>,
) -> Result {
    for item in dropped.read() {
        let toilet_transform = toilets.get(item.in_zone)?;
        let (mut ventouse_transform, mut ventouse) = ventouses.get_mut(item.item)?;

        commands
            .entity(item.item)
            .insert(LimitedDrag(Segment2d::new(vec2(0., 0.), vec2(0., 50.))));

        commands.entity(item.in_zone).remove::<DropZone>();

        ventouse.toilet = Some(item.in_zone);

        let pos = toilet_transform.translation.truncate() + vec2(0., 50.);
        ventouse_transform.translation = pos.extend(ventouse_transform.translation.z);
    }

    Ok(())
}

fn update_ventouse_count(
    mut commands: Commands,
    toilet_assets: Res<ToiletAssets>,
    mut remaining_toilets: ResMut<RemainingToilets>,
    ventouses: Query<(Entity, &GlobalTransform, &LimitedDrag, &mut Ventouse)>,
    mut toilets: Query<&mut Toilet, Without<Ventouse>>,
) -> Result {
    for (entity, transform, limit_drag, mut ventouse) in ventouses {
        let progression = limit_drag.progression(transform.translation().truncate());

        if ventouse.direction && progression < 0.1 || !ventouse.direction && progression > 0.9 {
            ventouse.count -= 1;
            ventouse.direction = !ventouse.direction;
        }

        if ventouse.count > 0 || ventouse.toilet.is_none() {
            continue;
        }

        let mut toilet = toilets.get_mut(ventouse.toilet.unwrap())?;
        toilet.0 = true;
        remaining_toilets.0 -= 1;
        commands.spawn(sound_effect(toilet_assets.flush_sfx.clone()));

        commands.entity(entity).remove::<LimitedDrag>();
        ventouse.reset();
    }

    Ok(())
}

fn check_completion(
    remaining: Res<RemainingToilets>,
    mut finished: MessageWriter<MinigameFinished>,
) {
    if remaining.is_changed() && remaining.0 == 0 {
        finished.write(MinigameFinished {
            game: MiniGame::Toilet,
        });
    }
}

fn update_toilet_sprite(
    toilets: Query<(&mut Sprite, &Toilet), Changed<Toilet>>,
    assets: Res<ToiletAssets>,
) {
    for (mut sprite, toilet) in toilets {
        sprite.image = if toilet.0 {
            assets.toilet_clean.clone()
        } else {
            assets.toilet_dirty.clone()
        };
    }
}
