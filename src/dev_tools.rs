//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};

use crate::{
    minigames::behaviour::{Draggable, DropZone},
    screens::Screen,
};

#[derive(Resource, Default)]
struct DebugGizmosEnabled(bool);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DebugGizmosEnabled>();

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    app.add_systems(
        Update,
        (
            toggle_debug.run_if(input_just_pressed(TOGGLE_KEY)),
            draw_minigame_gizmos.run_if(guizmo_enabled),
        ),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn guizmo_enabled(enabled: Res<DebugGizmosEnabled>) -> bool {
    enabled.0
}

fn toggle_debug(mut options: ResMut<UiDebugOptions>, mut enabled: ResMut<DebugGizmosEnabled>) {
    options.toggle();
    enabled.0 = !enabled.0;
}

fn draw_minigame_gizmos(
    mut gizmos: Gizmos,
    draggables: Query<&GlobalTransform, With<Draggable>>,
    drop_zones: Query<(&GlobalTransform, &DropZone)>,
) {
    for transform in &draggables {
        let pos = transform.translation().truncate();
        let half = 15.0;
        gizmos.line_2d(
            pos + Vec2::new(-half, -half),
            pos + Vec2::new(half, half),
            Color::srgb(1.0, 0.2, 0.2),
        );
        gizmos.line_2d(
            pos + Vec2::new(-half, half),
            pos + Vec2::new(half, -half),
            Color::srgb(1.0, 0.2, 0.2),
        );
    }

    for (transform, zone) in &drop_zones {
        let pos = transform.translation().truncate();
        gizmos.rect_2d(
            Isometry2d::from_translation(pos),
            zone.0,
            Color::srgb(0.2, 1.0, 0.2),
        );
    }
}
