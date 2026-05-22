//! Behavioural components for the minigames

use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, camera::CursorPosition};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<ItemDropped>();
    app.add_systems(Update,
        (on_draggable_added, move_dragged)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
    );
}



/// Mark an entity as mouse draggable.
#[derive(Component, Clone, Debug, Default)]
#[require(Pickable)]
pub struct Draggable;

/// Used to update the position of a dragged [`Draggable`] entity.
#[derive(Component, Clone, Debug)]
pub struct Dragged(Vec2);

/// Used to detect drop of a [`Draggable`] entity
#[derive(Component, Clone, Debug)]
pub struct DropZone(pub Vec2);

impl DropZone {
    fn is_in_zone(&self, zone_position: Vec2, item_position: Vec2) -> bool {
        let rect = Rect::from_corners(zone_position + self.0/2., zone_position - self.0/2.);
        rect.contains(item_position)
    }
}

/// [`Message`] sent everytime a [`Draggable`] entity is dropped into a [`DropZone`]
#[derive(Message, Clone, Debug)]
pub struct ItemDropped {
    pub item: Entity,
    pub in_zone: Entity,
}

fn on_draggable_added(
    mut commands: Commands,
    draggables: Query<Entity, Added<Draggable>>,
) {
    for draggable in draggables {
        commands.entity(draggable)
            .observe(on_drag_start)
            .observe(on_drag_end);
    }
}

fn on_drag_start(
    event: On<Pointer<DragStart>>,
    mut commands: Commands,
    cursor_position: Res<CursorPosition>,
    mut draggables: Query<&mut Transform, With<Draggable>>,
) -> Result {
    let mut drag_transform = draggables.get_mut(event.entity)?;
    let cursor_pos = cursor_position.pos().ok_or("Dragged detected outside of window.")?;

    drag_transform.translation.z = 2.0;
    commands
        .entity(event.entity)
        .insert(Dragged(cursor_pos - drag_transform.translation.truncate()));

    Ok(())
}

fn on_drag_end(
    event: On<Pointer<DragEnd>>,
    mut commands: Commands,
    mut dropped_message: MessageWriter<ItemDropped>,
    items: Query<&GlobalTransform, With<Draggable>>,
    zones: Query<(Entity, &GlobalTransform, &DropZone)>,
) -> Result {
    commands.entity(event.entity).try_remove::<Dragged>();

    let item_position = items.get(event.entity)?.translation().truncate();
    for (zone_entity, zone_transform, zone) in zones {
        if zone.is_in_zone(zone_transform.translation().truncate(), item_position) {
            dropped_message.write(ItemDropped { item: event.entity, in_zone: zone_entity });
        }
    }
    Ok(())
}

fn move_dragged(
    cards: Query<(&mut Transform, &Dragged)>,
    cursor_position: Res<CursorPosition>,
) {
    let Some(cursor_pos) = cursor_position.pos() else {
        return;
    };

    for (mut transform, dragging) in cards {
        transform.translation = (cursor_pos - dragging.0).extend(transform.translation.z);
    }
}
