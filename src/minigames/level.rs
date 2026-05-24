//! Task level behaviuour

use bevy::prelude::*;
use rand::Rng as _;

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    audio::{music, sound_effect},
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
    app.load_resource::<LevelAssets>();

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (reset_completion_state, start_music),
    );
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

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct LevelAssets {
    #[dependency]
    background: Handle<Image>,
    #[dependency]
    skeleton: [Handle<Image>; 2],
    #[dependency]
    toilets: [Handle<Image>; 2],
    #[dependency]
    cashier: [Handle<Image>; 2],
    #[dependency]
    trash: [Handle<Image>; 2],
    #[dependency]
    pannels: Handle<Image>,
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    task_completed_sfx: Handle<AudioSource>,
    #[dependency]
    new_task_sfx: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            background: assets.load("images/level/background.png"),
            skeleton: [
                assets.load("images/level/skeleton_full.png"),
                assets.load("images/level/skeleton_broken.png"),
            ],
            toilets: [
                assets.load("images/level/toilets_clean.png"),
                assets.load("images/level/toilets_dirty.png"),
            ],
            cashier: [
                assets.load("images/level/cashier_empty.png"),
                assets.load("images/level/cashier_visitors.png"),
            ],
            trash: [
                assets.load("images/level/trash_clean.png"),
                assets.load("images/level/trash_dirty.png"),
            ],
            pannels: assets.load("images/level/panneaux.png"),
            music: assets.load("audio/music/game_loop.ogg"),
            task_completed_sfx: assets.load("audio/sound_effects/task_completed.mp3"),
            new_task_sfx: assets.load("audio/sound_effects/new_task.mp3"),
        }
    }
}

impl LevelAssets {
    fn get_minigame_assets(&self, minigame: MiniGame) -> Option<&[Handle<Image>; 2]> {
        match minigame {
            MiniGame::Skeleton => Some(&self.skeleton),
            MiniGame::Cashier => Some(&self.cashier),
            MiniGame::Toilet => Some(&self.toilets),
            MiniGame::Trash => Some(&self.trash),
            _ => None,
        }
    }
}

fn start_music(mut commands: Commands, credits_music: Res<LevelAssets>) {
    commands.spawn((
        Name::new("Game Music"),
        DespawnOnExit(Screen::Title),
        music(credits_music.music.clone()),
    ));
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

pub(crate) fn spawn_minigames_selection(mut commands: Commands, level_assets: Res<LevelAssets>) {
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

    // Root entity — toutes les entités du niveau en sont enfants
    // Les images sont 2560×1440 ; à scale 0.75 elles font 1920×1080 (viewport exact)
    let bg = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    commands.spawn((
        ChildOf(bg),
        Sprite::from_image(level_assets.background.clone()),
        Transform::from_scale(Vec3::splat(0.75)),
    ));

    commands.spawn((
        ChildOf(bg),
        Sprite::from_image(level_assets.pannels.clone()),
        Transform::from_translation(vec3(0., 0., 0.2)).with_scale(Vec3::splat(0.75)),
    ));

    // Sprites décoratifs (non-interactifs)
    for (minigame, pos) in [
        (MiniGame::Skeleton, vec2(-390., -200.)),
        (MiniGame::Toilet, vec2(-577., 420.)),
        (MiniGame::Cashier, vec2(725., -100.)),
        (MiniGame::Trash, vec2(-350., 350.)),
    ] {
        let images = level_assets.get_minigame_assets(minigame).unwrap();
        commands.spawn((
            ChildOf(bg),
            StartOnClick(minigame),
            TaskState::Available,
            Sprite::from_image(images[0].clone()),
            Transform::from_translation(pos.extend(0.1)).with_scale(Vec3::splat(0.75)),
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
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
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

    // Play sfx
    commands.spawn(sound_effect(level_assets.new_task_sfx.clone()));

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
    level_assets: Res<LevelAssets>,
    tasks: Query<(&mut Sprite, &StartOnClick, &TaskState), Changed<TaskState>>,
) {
    for (mut sprite, minigame, state) in tasks {
        let Some(sprites) = level_assets.get_minigame_assets(minigame.0) else {
            continue;
        };

        sprite.image = match state {
            TaskState::Active => sprites[1].clone(),
            _ => sprites[0].clone(),
        };
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
    if !matches!(state, TaskState::Active) {
        return Ok(());
    }

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
    transform.scale = Vec3::splat(0.82);

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
    transform.scale = Vec3::splat(0.75);

    let (_, ref mut visibility) = *tooltip;
    **visibility = Visibility::Hidden;
}

fn update_completion_notification(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
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

                // Play sfx
                commands.spawn(sound_effect(level_assets.task_completed_sfx.clone()));
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
