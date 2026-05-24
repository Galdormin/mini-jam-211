//! Behavioural components for the minigames

use bevy::{prelude::*, transform::TransformSystems};

use crate::{AppSystems, PausableSystems, camera::CursorPosition};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<ItemDropped>();
    app.add_message::<ItemClicked>();
    app.add_systems(
        Update,
        (on_draggable_added, on_clickable_added, move_dragged)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        PostUpdate,
        on_limited_drag_added.after(TransformSystems::Propagate),
    );
}

/// Mark an entity as mouse draggable.
#[derive(Component, Clone, Debug, Default)]
#[require(Pickable)]
pub struct Draggable;

/// Used to update the position of a dragged [`Draggable`] entity.
#[derive(Component, Clone, Debug)]
pub struct Dragged(Vec2);

/// Used to limit the movement of a [`Draggable`] entity on a [`Segment2d`]
#[derive(Component, Clone, Debug)]
pub struct LimitedDrag(pub Segment2d);

impl LimitedDrag {
    pub fn progression(&self, pos: Vec2) -> f32 {
        let d1 = pos.distance(self.0.point1());
        d1 / self.0.length()
    }
}

/// Used to detect drop of a [`Draggable`] entity
#[derive(Component, Clone, Debug)]
pub struct DropZone(pub Vec2);

impl DropZone {
    pub fn rect(&self, position: Vec2) -> Rect {
        Rect::from_corners(position + self.0 / 2., position - self.0 / 2.)
    }
}

/// Mark an entity as mouse clickable.
#[derive(Component, Clone, Debug, Default)]
#[require(Pickable)]
pub struct Clickable;

/// [`Message`] sent every time a [`Clickable`] entity is clicked.
#[derive(Message, Clone, Debug)]
pub struct ItemClicked {
    pub item: Entity,
}

/// [`Message`] sent everytime a [`Draggable`] entity is dropped into a [`DropZone`]
#[derive(Message, Clone, Debug)]
pub struct ItemDropped {
    pub item: Entity,
    pub in_zone: Entity,
}

fn on_clickable_added(mut commands: Commands, clickables: Query<Entity, Added<Clickable>>) {
    for clickable in clickables {
        commands.entity(clickable).observe(on_click);
    }
}

fn on_click(event: On<Pointer<Click>>, mut clicked: MessageWriter<ItemClicked>) {
    clicked.write(ItemClicked { item: event.entity });
}

fn on_draggable_added(mut commands: Commands, draggables: Query<Entity, Added<Draggable>>) {
    for draggable in draggables {
        commands
            .entity(draggable)
            .observe(on_drag_start)
            .observe(on_drag_end);
    }
}

/// Update the local limited drag position to global position
fn on_limited_drag_added(
    limited_drags: Query<(&GlobalTransform, &mut LimitedDrag), Added<LimitedDrag>>,
) {
    for (transform, mut limited_drag) in limited_drags {
        limited_drag.0 = limited_drag
            .0
            .translated(transform.translation().truncate());
    }
}

fn on_drag_start(
    event: On<Pointer<DragStart>>,
    mut commands: Commands,
    cursor_position: Res<CursorPosition>,
    mut draggables: Query<&mut Transform, With<Draggable>>,
) -> Result {
    let mut drag_transform = draggables.get_mut(event.entity)?;
    let cursor_pos = cursor_position
        .pos()
        .ok_or("Dragged detected outside of window.")?;

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
        if zone
            .rect(zone_transform.translation().truncate())
            .contains(item_position)
        {
            dropped_message.write(ItemDropped {
                item: event.entity,
                in_zone: zone_entity,
            });
        }
    }
    Ok(())
}

fn move_dragged(
    cards: Query<(&mut Transform, &Dragged, Option<&LimitedDrag>)>,
    cursor_position: Res<CursorPosition>,
) {
    let Some(cursor_pos) = cursor_position.pos() else {
        return;
    };

    for (mut transform, dragging, maybe_limited) in cards {
        let dragged_position = cursor_pos - dragging.0;

        transform.translation = if let Some(limited_drag) = maybe_limited {
            limited_drag
                .0
                .closest_point(dragged_position)
                .extend(transform.translation.z)
        } else {
            dragged_position.extend(transform.translation.z)
        };
    }
}
