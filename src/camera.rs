//! Define the behavior of the Camera

use bevy::{prelude::*, window::PrimaryWindow};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, update_cursor_position)
        .init_resource::<CursorPosition>();
}

/// Used to help identify our main camera
#[derive(Component)]
#[require(Camera2d)]
pub struct MainCamera;

/// Used to store the cursor world position
#[derive(Resource, Debug, Default, Deref)]
pub struct CursorPosition(Option<Vec2>);

impl CursorPosition {
    pub fn pos(&self) -> Option<Vec2> {
        self.0
    }
}

fn update_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    cursor_position.0 = window
        .cursor_position()
        .and_then(|cursor| camera.0.viewport_to_world(camera.1, cursor).ok())
        .map(|ray| ray.origin.truncate())
}
