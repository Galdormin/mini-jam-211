//! The screen state for the intro.

use bevy::prelude::*;

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    minigames::behaviour::{Draggable, DropZone, ItemDropped, LimitedDrag},
    screens::{Screen, title::MenuBackground},
};

const FADE_DURATION: f32 = 1.;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<IntroAssets>();
    app.add_systems(OnEnter(Screen::Intro), spawn_intro);
    app.add_systems(
        Update,
        run_intro
            .in_set(AppSystems::Update)
            .run_if(resource_exists::<IntroState>),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct IntroAssets {
    #[dependency]
    locked: Handle<Image>,
    #[dependency]
    message: Handle<Image>,
    #[dependency]
    phone_unlocker: Handle<Image>,
    #[dependency]
    phone_unlock_sfx: Handle<AudioSource>,
    #[dependency]
    phone_notif_sfx: Handle<AudioSource>,
}

impl FromWorld for IntroAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            locked: assets.load("images/intro_locked.png"),
            message: assets.load("images/intro_message.png"),
            phone_unlocker: assets.load("images/phone_unlocker.png"),
            phone_unlock_sfx: assets.load("audio/sound_effects/phone_unlock.ogg"),
            phone_notif_sfx: assets.load("audio/sound_effects/phone_notification.mp3"),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
struct IntroBackground;

#[derive(Component, Clone, Debug, Default)]
struct PhoneUnlocker;

#[derive(Component, Clone, Debug, Default)]
struct PhoneDropZone;

#[derive(Resource, Clone, Debug)]
enum IntroState {
    FadeOutMenu(f32),
    FadeInIntro(f32),
    UnlockPhone,
    WaitIntro,
    FadeOutIntro(f32),
}

fn spawn_intro(mut commands: Commands, intro_assets: Res<IntroAssets>) {
    commands.insert_resource(IntroState::FadeOutMenu(1.0));

    commands.spawn((
        IntroBackground,
        DespawnOnExit(Screen::Intro),
        Sprite {
            image: intro_assets.locked.clone(),
            color: Color::srgba(1., 1., 1., 0.),
            ..default()
        },
        Transform::from_scale(Vec3::splat(0.75)),
        Visibility::Hidden,
    ));

    // Draggable unlocker — starts hidden, shown when UnlockPhone state begins
    commands.spawn((
        PhoneUnlocker,
        Draggable,
        DespawnOnExit(Screen::Intro),
        Sprite::from_image(intro_assets.phone_unlocker.clone()),
        Transform::from_translation(vec3(-165., -375., 1.)).with_scale(Vec3::splat(0.75)),
        LimitedDrag(Segment2d::new(vec2(0., 0.), vec2(320., 0.))),
        Visibility::Hidden,
    ));

    // Invisible drop zone at the right end of the drag segment
    commands.spawn((
        PhoneDropZone,
        DropZone(vec2(80., 80.)),
        DespawnOnExit(Screen::Intro),
        Transform::from_translation(vec3(155., -375., 0.)),
    ));
}

fn run_intro(
    mut commands: Commands,
    intro_assets: Res<IntroAssets>,
    time: Res<Time>,
    mut intro_state: ResMut<IntroState>,
    mut menu_background: Single<
        (&mut Sprite, &mut Visibility),
        (
            With<MenuBackground>,
            Without<IntroBackground>,
            Without<PhoneUnlocker>,
        ),
    >,
    mut intro_background: Single<
        (&mut Sprite, &mut Visibility),
        (
            With<IntroBackground>,
            Without<MenuBackground>,
            Without<PhoneUnlocker>,
        ),
    >,
    mut unlocker_visibility: Single<
        (&mut Sprite, &mut Visibility),
        (
            With<PhoneUnlocker>,
            Without<IntroBackground>,
            Without<MenuBackground>,
        ),
    >,
    mut dropped: MessageReader<ItemDropped>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let (ref mut intro_sprite, ref mut intro_visibility) = *intro_background;
    let (ref mut menu_sprite, ref mut menu_visibility) = *menu_background;
    let (ref mut unlocker_sprite, ref mut locker_visibility) = *unlocker_visibility;

    let next = match &mut *intro_state {
        IntroState::FadeOutMenu(t) => {
            *t = (*t - time.delta_secs() / FADE_DURATION).min(1.0);
            menu_sprite.color = Color::srgba(1., 1., 1., *t);
            if *t <= 0.0 {
                **menu_visibility = Visibility::Hidden;
                Some(IntroState::FadeInIntro(0.0))
            } else {
                None
            }
        }
        IntroState::FadeInIntro(t) => {
            *t = (*t + time.delta_secs() / FADE_DURATION).min(1.0);
            intro_sprite.color = Color::srgba(1., 1., 1., *t);
            unlocker_sprite.color = Color::srgba(1., 1., 1., *t);
            (*t >= 1.0).then_some(IntroState::UnlockPhone)
        }
        IntroState::UnlockPhone => {
            if dropped.read().count() > 0 {
                Some(IntroState::WaitIntro)
            } else {
                None
            }
        }
        IntroState::WaitIntro => {
            let any_input = mouse.just_pressed(MouseButton::Left)
                || keys.just_pressed(KeyCode::Space)
                || keys.just_pressed(KeyCode::Enter);
            any_input.then_some(IntroState::FadeOutIntro(0.0))
        }
        IntroState::FadeOutIntro(t) => {
            *t = (*t + time.delta_secs() / FADE_DURATION).min(1.0);
            intro_sprite.color = Color::srgba(1., 1., 1., 1.0 - *t);
            if *t >= 1.0 {
                next_screen.set(Screen::Gameplay);
                commands.remove_resource::<IntroState>();
            }
            None
        }
    };

    if let Some(new_state) = next {
        match &new_state {
            IntroState::FadeInIntro(_) => {
                **intro_visibility = Visibility::Visible;
                **locker_visibility = Visibility::Visible;
                intro_sprite.color = Color::srgba(1., 1., 1., 0.);

                commands.spawn(sound_effect(intro_assets.phone_notif_sfx.clone()));
            }
            IntroState::WaitIntro => {
                commands.spawn(sound_effect(intro_assets.phone_unlock_sfx.clone()));
                intro_sprite.image = intro_assets.message.clone();
                **locker_visibility = Visibility::Hidden;
            }
            _ => {}
        }
        *intro_state = new_state;
    }
}
