//! Task level behaviuour

use bevy::prelude::*;

use crate::{
    AppSystems,
    math::*,
    minigames::games::{MiniGame, MinigameFinished},
    screens::Screen,
};

const ANIM_SECS: f32 = 0.3;
const NOTIF_ON_Y: f32 = 1080. / 2.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CompletionState>();
    app.add_systems(OnEnter(Screen::Gameplay), reset_completion_state);
    app.add_systems(
        Update,
        (
            on_add_start_on_click.in_set(AppSystems::Update),
            update_completion_notification.run_if(in_state(Screen::Gameplay)),
        ),
    );
}

/// Marker component for the Level Tooltip on hover
#[derive(Component)]
struct LevelTooltip;

#[derive(Component)]
struct CompletionNotification;

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

/// Setup a test level to try minigames
pub fn spawn_minigames_selection(mut commands: Commands) {
    // Minigame tooltip
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

    commands.spawn((
        Sprite::from_color(Color::srgb_u8(45, 45, 45), vec2(2000., 2000.)),
        DespawnOnExit(Screen::Gameplay),
        children![
            (
                Sprite::from_color(Color::srgb_u8(249, 231, 231), vec2(500., 300.)),
                StartOnClick(MiniGame::Skeleton),
                Transform::from_translation(vec3(-200., -150., 0.1))
            ),
            (
                Sprite::from_color(Color::srgb_u8(76, 147, 138), vec2(20., 40.)),
                StartOnClick(MiniGame::Trash),
                Transform::from_translation(vec3(-360., 250., 0.1))
            ),
            (
                Sprite::from_color(Color::srgb_u8(200, 30, 200), vec2(360., 150.)),
                StartOnClick(MiniGame::Toilet),
                Transform::from_translation(vec3(-650., 270., 0.1))
            ),
            (
                Sprite::from_color(Color::srgb_u8(200, 30, 30), vec2(340., 200.)),
                StartOnClick(MiniGame::Cashier),
                Transform::from_translation(vec3(770., 100., 0.1))
            )
        ],
    ));
}

fn update_completion_notification(
    mut state: ResMut<CompletionState>,
    mut notification: Single<(&mut Node, &mut Visibility), With<CompletionNotification>>,
    mut finished: MessageReader<MinigameFinished>,
    mut next_minigame: ResMut<NextState<MiniGame>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    let count = finished.read().count();
    if count > 0 && matches!(state.0, NotificationPhase::Idle) {
        state.0 = NotificationPhase::Pending(Timer::from_seconds(0.1, TimerMode::Once));
    }

    let (ref mut node, ref mut visibility) = *notification;

    // Simple state machine
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

/// Add observer to [`StartOnClick`]
fn on_add_start_on_click(mut commands: Commands, starts: Query<Entity, Added<StartOnClick>>) {
    for start in starts {
        commands
            .entity(start)
            .observe(on_click_start_on_click)
            .observe(on_hover_start_on_click)
            .observe(on_out_start_on_click);
    }
}

/// Change [`MiniGame`] when [`StartOnClick`] is clicked
fn on_click_start_on_click(
    event: On<Pointer<Click>>,
    mut minigame: ResMut<NextState<MiniGame>>,
    starts: Query<&StartOnClick>,
) -> Result {
    minigame.set(**starts.get(event.entity)?);
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
