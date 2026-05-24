//! Stress bar, game timer, music speed

use bevy::prelude::*;

use crate::{
    AppSystems, audio::Music, math::*, minigames::games::MinigameFinished, screens::Screen,
};

const GAME_DURATION_SECS: f32 = 300.0;
pub(super) const DRAIN_BASE: f32 = 0.005;
pub(super) const DRAIN_PER_TASK: f32 = 0.02;
const COMPLETION_BONUS: f32 = 0.30;
const MUSIC_SPEED_MAX: f32 = 1.8;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameProgress>();
    app.init_resource::<GameTimer>();

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (reset_game_state, spawn_score_ui),
    );
    app.add_systems(
        Update,
        (
            (tick_game_timer)
                .in_set(AppSystems::TickTimers)
                .run_if(in_state(Screen::Gameplay)),
            (
                on_task_completed_progress,
                update_progress_ui,
                update_music_speed.run_if(resource_changed::<GameTimer>),
            )
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
        ),
    );
}

/// Stress bar: 1.0 = calm, 0.0 = game over.
#[derive(Resource)]
pub(crate) struct GameProgress(pub f32);

impl Default for GameProgress {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Absolute survival countdown in seconds.
#[derive(Resource)]
pub(crate) struct GameTimer(pub f32);

impl Default for GameTimer {
    fn default() -> Self {
        Self(GAME_DURATION_SECS)
    }
}

impl GameTimer {
    /// 0.0 at start, 1.0 at end — used to scale task interval difficulty.
    pub(crate) fn urgency(&self) -> f32 {
        1.0 - (self.0 / GAME_DURATION_SECS).clamp(0., 1.)
    }
}

#[derive(Component)]
struct ProgressBarFill;

#[derive(Component)]
struct GameTimerText;

fn reset_game_state(mut progress: ResMut<GameProgress>, mut timer: ResMut<GameTimer>) {
    progress.0 = 1.0;
    timer.0 = GAME_DURATION_SECS;
}

fn spawn_score_ui(mut commands: Commands) {
    // Timer text above the bar
    commands.spawn((
        GameTimerText,
        DespawnOnExit(Screen::Gameplay),
        GlobalZIndex(4),
        Text::new("5:00"),
        TextFont {
            font_size: 20.,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(8.),
            left: Val::Px(8.),
            ..default()
        },
    ));

    // Vertical stress bar on the right
    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        GlobalZIndex(4),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(11.),
            left: Val::Px(20.),
            width: Val::Px(25.),
            height: Val::Percent(78.),
            border: UiRect::all(Val::Px(2.)),
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::ColumnReverse,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(Color::srgba(0., 0., 0., 0.6)),
        children![(
            ProgressBarFill,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            BackgroundColor(Color::srgb(0., 0.8, 0.)),
        )],
    ));
}

fn tick_game_timer(mut timer: ResMut<GameTimer>, time: Res<Time>) {
    timer.0 = (timer.0 - time.delta_secs()).max(0.0);
}

fn on_task_completed_progress(
    mut finished: MessageReader<MinigameFinished>,
    mut progress: ResMut<GameProgress>,
) {
    for _ in finished.read() {
        progress.0 = (progress.0 + COMPLETION_BONUS).min(1.0);
    }
}

fn update_progress_ui(
    progress: Res<GameProgress>,
    timer: Res<GameTimer>,
    mut fill: Single<(&mut Node, &mut BackgroundColor), With<ProgressBarFill>>,
    mut timer_text: Single<&mut Text, With<GameTimerText>>,
) {
    if !progress.is_changed() && !timer.is_changed() {
        return;
    }

    let p = progress.0;
    let (ref mut node, ref mut bg) = *fill;
    node.height = Val::Percent(p * 100.);
    let r = if p > 0.5 { (1.0 - p) * 2.0 } else { 1.0 };
    let g = if p > 0.5 { 1.0 } else { p * 2.0 };
    bg.0 = Color::srgb(r, g, 0.);

    let secs = timer.0 as u32;
    timer_text.0 = format!("{}:{:02}", secs / 60, secs % 60);
}

fn update_music_speed(timer: Res<GameTimer>, mut music_sinks: Query<&AudioSink, With<Music>>) {
    let speed = lerp(1.0, MUSIC_SPEED_MAX, timer.urgency());
    for sink in &mut music_sinks {
        sink.set_speed(speed);
    }
}
