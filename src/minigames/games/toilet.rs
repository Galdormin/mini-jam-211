//! MiniGame: Clean the toilets

use bevy::prelude::*;

use crate::{
    AppSystems,
    minigames::{
        behaviour::{Draggable, DropZone, ItemDropped, LimitedDrag},
        games::{MiniGame, MinigameFinished, setup_minigame_background},
    },
};

pub(super) fn plugin(app: &mut App) {
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

fn setup_minigame(mut commands: Commands) {
    commands.insert_resource(RemainingToilets(2));

    let base = setup_minigame_background(&mut commands, super::MiniGame::Toilet);

    // Toilets
    commands.spawn((
        ChildOf(base),
        Toilet::default(),
        DropZone(vec2(210., 370.)),
        Sprite::from_color(Color::srgb_u8(200, 200, 200), vec2(200., 360.)),
        Transform::from_translation(vec3(-300., -50., 1.)),
    ));
    commands.spawn((
        ChildOf(base),
        Toilet::default(),
        DropZone(vec2(210., 370.)),
        Sprite::from_color(Color::srgb_u8(200, 200, 200), vec2(200., 360.)),
        Transform::from_translation(vec3(300., -50., 1.)),
    ));

    // Ventouse
    commands.spawn((
        ChildOf(base),
        Ventouse::default(),
        Draggable,
        Sprite::from_color(Color::srgb_u8(50, 50, 200), vec2(20., 200.)),
        Transform::from_translation(vec3(-500., -150., 1.1)),
        children![(
            Sprite::from_color(Color::srgb_u8(200, 50, 50), vec2(60., 60.)),
            Transform::from_translation(vec3(0., -100., 0.1)),
        )],
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
        ventouse_transform.translation = toilet_transform
            .translation
            .truncate()
            .extend(ventouse_transform.translation.z);
    }

    Ok(())
}

fn update_ventouse_count(
    mut commands: Commands,
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

        // Change toilet status
        let mut toilet = toilets.get_mut(ventouse.toilet.unwrap())?;
        toilet.0 = true;
        remaining_toilets.0 -= 1;

        // Disable limited drag for ventouse
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
        finished.write(MinigameFinished { game: MiniGame::Toilet });
    }
}

fn update_toilet_sprite(toilets: Query<(&mut Sprite, &Toilet), Changed<Toilet>>) {
    for (mut sprite, toilet) in toilets {
        if toilet.0 {
            sprite.color = Color::srgb_u8(200, 200, 200);
        } else {
            sprite.color = Color::srgb_u8(200, 100, 0);
        }
    }
}
