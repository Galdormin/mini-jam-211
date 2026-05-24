//! Task level behaviuour

use bevy::prelude::*;
use rand::Rng as _;

use crate::{
    AppSystems,
    math::*,
    minigames::games::{MiniGame, MinigameFinished},
    screens::Screen,
};

const ANIM_SECS: f32 = 0.3;
const NOTIF_ON_Y: f32 = 1080. / 2.;

const COOLDOWN_SECS: f32 = 5.0;
const WAIT_MIN_SECS: f32 = 2.0;
const WAIT_MAX_SECS: f32 = 8.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CompletionState>();
    app.init_resource::<NextTaskTimer>();
    app.add_systems(OnEnter(Screen::Gameplay), reset_completion_state);
    app.add_systems(
        Update,
        (
            (tick_cooldowns, tick_next_task_timer)
                .in_set(AppSystems::TickTimers)
                .run_if(in_state(Screen::Gameplay)),
            (
                on_add_start_on_click,
                on_minigame_finished,
                update_task_indicators.after(on_minigame_finished),
                update_completion_notification,
            )
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
        ),
    );
}

/// State for the minigame tasks
#[derive(Component)]
enum TaskState {
    Available,
    Active,
    Cooldown(Timer),
}

impl TaskState {
    fn cooldown() -> Self {
        Self::Cooldown(Timer::from_seconds(COOLDOWN_SECS, TimerMode::Once))
    }
}

/// Global timer that fires to activate a random available task
#[derive(Resource)]
struct NextTaskTimer(Timer);

impl Default for NextTaskTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            rand::rng().random_range(WAIT_MIN_SECS..WAIT_MAX_SECS),
            TimerMode::Once,
        ))
    }
}

/// Marker component for the Minigame tooltip when hovered
#[derive(Component)]
struct LevelTooltip;

/// Marker component for the minifame completion notification
#[derive(Component)]
struct CompletionNotification;

/// Marker component for the task indicator
#[derive(Component)]
struct TaskIndicator;

/// Minigame finished notification state machine
#[derive(Default)]
enum NotificationPhase {
    #[default]
    Idle,
    Pending(Timer),
    Entering(f32),
    Visible,
    Leaving(f32),
}

#[derive(Resource, Default)]
struct CompletionState(NotificationPhase);

fn reset_completion_state(mut state: ResMut<CompletionState>) {
    state.0 = NotificationPhase::Idle;
}

/// Start MiniGame on click
#[derive(Component, Clone, Debug, Deref)]
#[require(Pickable)]
pub struct StartOnClick(MiniGame);

pub(crate) fn spawn_minigames_selection(mut commands: Commands) {
    // Hover tooltip
    commands.spawn((
        LevelTooltip,
        Text::new(""),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
        Visibility::Hidden,
    ));

    // Completion notification
    commands.spawn((
        CompletionNotification,
        DespawnOnExit(Screen::Gameplay),
        GlobalZIndex(5),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Visibility::Hidden,
        children![(
            Text::new("Minijeu terminé"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    ));

    // Background
    let bg = commands
        .spawn((
            Sprite::from_color(Color::srgb_u8(45, 45, 45), vec2(2000., 2000.)),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    // Tasks
    let tasks = [
        (
            Color::srgb_u8(249, 231, 231),
            vec2(500., 300.),
            MiniGame::Skeleton,
            vec3(-200., -150., 0.1),
        ),
        (
            Color::srgb_u8(76, 147, 138),
            vec2(20., 40.),
            MiniGame::Trash,
            vec3(-360., 250., 0.1),
        ),
        (
            Color::srgb_u8(200, 30, 200),
            vec2(360., 150.),
            MiniGame::Toilet,
            vec3(-650., 270., 0.1),
        ),
        (
            Color::srgb_u8(200, 30, 30),
            vec2(340., 200.),
            MiniGame::Cashier,
            vec3(770., 100., 0.1),
        ),
    ];

    for (color, size, game, pos) in tasks {
        let task = commands
            .spawn((
                ChildOf(bg),
                Sprite::from_color(color, size),
                StartOnClick(game),
                Transform::from_translation(pos),
                TaskState::Available,
            ))
            .id();

        commands.spawn((
            ChildOf(task),
            TaskIndicator,
            Sprite::from_color(Color::srgb_u8(255, 160, 0), vec2(40., 40.)),
            Transform::from_translation(vec3(0., size.y / 2. + 35., 0.1)),
            Visibility::Hidden,
        ));
    }
}

fn tick_cooldowns(mut tasks: Query<&mut TaskState, With<StartOnClick>>, time: Res<Time>) {
    for mut state in &mut tasks {
        if let TaskState::Cooldown(timer) = &mut *state {
            timer.tick(time.delta());
            if timer.just_finished() {
                *state = TaskState::Available;
            }
        }
    }
}

fn tick_next_task_timer(
    mut timer: ResMut<NextTaskTimer>,
    mut tasks: Query<(Entity, &mut TaskState), With<StartOnClick>>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    timer.0 = Timer::from_seconds(
        rand::rng().random_range(WAIT_MIN_SECS..WAIT_MAX_SECS),
        TimerMode::Once,
    );

    let available: Vec<Entity> = tasks
        .iter()
        .filter(|(_, state)| matches!(**state, TaskState::Available))
        .map(|(e, _)| e)
        .collect();

    if available.is_empty() {
        return;
    }

    let chosen = available[rand::rng().random_range(0..available.len())];
    if let Ok((_, mut state)) = tasks.get_mut(chosen) {
        *state = TaskState::Active;
    }
}

fn on_minigame_finished(
    mut finished: MessageReader<MinigameFinished>,
    mut tasks: Query<(&StartOnClick, &mut TaskState)>,
) {
    for msg in finished.read() {
        for (start, mut state) in &mut tasks {
            if **start == msg.game && matches!(*state, TaskState::Active) {
                *state = TaskState::cooldown();
            }
        }
    }
}

fn update_task_indicators(
    tasks: Query<(&TaskState, &Children), (With<StartOnClick>, Changed<TaskState>)>,
    mut indicators: Query<&mut Visibility, With<TaskIndicator>>,
) {
    for (state, children) in &tasks {
        for &child in children {
            if let Ok(mut visibility) = indicators.get_mut(child) {
                *visibility = match state {
                    TaskState::Active => Visibility::Visible,
                    _ => Visibility::Hidden,
                };
            }
        }
    }
}

fn on_add_start_on_click(mut commands: Commands, starts: Query<Entity, Added<StartOnClick>>) {
    for start in starts {
        commands
            .entity(start)
            .observe(on_click_start_on_click)
            .observe(on_hover_start_on_click)
            .observe(on_out_start_on_click);
    }
}

fn on_click_start_on_click(
    event: On<Pointer<Click>>,
    mut minigame: ResMut<NextState<MiniGame>>,
    starts: Query<(&StartOnClick, &TaskState)>,
) -> Result {
    let (start, state) = starts.get(event.entity)?;
    // if !matches!(state, TaskState::Active) {
    //     return Ok(());
    // }

    minigame.set(**start);
    Ok(())
}

fn on_hover_start_on_click(
    event: On<Pointer<Over>>,
    mut starts: Query<(&mut Transform, &StartOnClick)>,
    mut tooltip: Single<(&mut Text, &mut Visibility), With<LevelTooltip>>,
) {
    let Ok((mut transform, start)) = starts.get_mut(event.entity) else {
        return;
    };
    transform.scale = 1.1 * Vec3::ONE;

    let (ref mut text, ref mut visibility) = *tooltip;
    text.0 = start.title().to_string();
    **visibility = Visibility::Visible;
}

fn on_out_start_on_click(
    event: On<Pointer<Out>>,
    mut starts: Query<&mut Transform, With<StartOnClick>>,
    mut tooltip: Single<(&mut Text, &mut Visibility), With<LevelTooltip>>,
) {
    let Ok(mut transform) = starts.get_mut(event.entity) else {
        return;
    };
    transform.scale = Vec3::ONE;

    let (_, ref mut visibility) = *tooltip;
    **visibility = Visibility::Hidden;
}

fn update_completion_notification(
    mut state: ResMut<CompletionState>,
    mut notification: Single<(&mut Node, &mut Visibility), With<CompletionNotification>>,
    mut finished: MessageReader<MinigameFinished>,
    mut next_minigame: ResMut<NextState<MiniGame>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if finished.read().count() > 0 && matches!(state.0, NotificationPhase::Idle) {
        state.0 = NotificationPhase::Pending(Timer::from_seconds(0.5, TimerMode::Once));
    }

    let (ref mut node, ref mut visibility) = *notification;

    match &mut state.0 {
        NotificationPhase::Idle => {}
        NotificationPhase::Pending(timer) => {
            timer.tick(time.delta());
            if timer.just_finished() {
                **visibility = Visibility::Visible;
                state.0 = NotificationPhase::Entering(0.0);
            }
        }
        NotificationPhase::Entering(t) => {
            *t = (*t + time.delta_secs() / ANIM_SECS).min(1.0);
            node.bottom = Val::Px(lerp(0., NOTIF_ON_Y, ease_out(*t)));
            if *t >= 1.0 {
                state.0 = NotificationPhase::Visible;
            }
        }
        NotificationPhase::Visible => {
            if mouse.just_pressed(MouseButton::Left) {
                state.0 = NotificationPhase::Leaving(0.0);
            }
        }
        NotificationPhase::Leaving(t) => {
            *t = (*t + time.delta_secs() / ANIM_SECS).min(1.0);
            node.bottom = Val::Px(lerp(NOTIF_ON_Y, 0., ease_in(*t)));
            if *t >= 1.0 {
                **visibility = Visibility::Hidden;
                node.bottom = Val::Px(0.);
                state.0 = NotificationPhase::Idle;
                next_minigame.set(MiniGame::None);
            }
        }
    }
}
