//! Define the behavior of the Camera

use bevy::{
    camera::ScalingMode,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

const REF_WIDTH: f32 = 1920.0;
const REF_HEIGHT: f32 = 1080.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (spawn_camera, setup_ui_scale))
        .add_systems(PreUpdate, (update_cursor_position, update_ui_scale))
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        MainCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1920.0,
                min_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn setup_ui_scale(mut ui_scale: ResMut<UiScale>, window: Single<&Window, With<PrimaryWindow>>) {
    ui_scale.0 = (window.width() / REF_WIDTH).min(window.height() / REF_HEIGHT);
}

fn update_ui_scale(mut ui_scale: ResMut<UiScale>, mut resize_events: MessageReader<WindowResized>) {
    for event in resize_events.read() {
        ui_scale.0 = (event.width / REF_WIDTH).min(event.height / REF_HEIGHT);
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
