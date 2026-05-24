//! MiniGame: Sell tickets to visitors

use bevy::prelude::*;
use rand::{Rng as _, seq::IndexedRandom};

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    math::*,
    minigames::{
        behaviour::{Clickable, Draggable, DropZone, ItemClicked, ItemDropped},
        games::{MiniGame, MinigameFinished, setup_minigame_background},
    },
};

const ANIM_SECS: f32 = 0.5;
const VISITOR_CENTER_X: f32 = 0.;
const VISITOR_ENTER_X: f32 = -1100.;
const VISITOR_EXIT_X: f32 = 1100.;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TicketAssets>();
    app.add_systems(OnEnter(MiniGame::Tickets), setup_minigame);
    app.add_systems(OnExit(MiniGame::Tickets), cleanup_minigame);
    app.add_systems(
        Update,
        tick_tickets
            .in_set(AppSystems::Update)
            .run_if(in_state(MiniGame::Tickets)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TicketAssets {
    #[dependency]
    desk: Handle<Image>,
    #[dependency]
    ticket: Handle<Image>,
    #[dependency]
    button: Handle<Image>,
    #[dependency]
    visitors: Vec<Handle<Image>>,
    #[dependency]
    ticket_sfx: Handle<AudioSource>,
}

impl FromWorld for TicketAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            desk: assets.load("images/minigames/tickets/desk.png"),
            ticket: assets.load("images/minigames/tickets/ticket.png"),
            button: assets.load("images/minigames/tickets/button.png"),
            visitors: vec![
                assets.load("images/minigames/tickets/character_1.png"),
                assets.load("images/minigames/tickets/character_2.png"),
                assets.load("images/minigames/tickets/character_3.png"),
                assets.load("images/minigames/tickets/character_4.png"),
            ],
            ticket_sfx: assets.load("audio/sound_effects/print_ticket.ogg"),
        }
    }
}

#[derive(Resource)]
struct RemainingVisitors(u32);

#[derive(Resource)]
struct TicketsBase(Entity);

#[derive(Resource)]
enum TicketsState {
    VisitorEntering { t: f32, visitor: Entity },
    WaitingForTicket { visitor: Entity },
    TicketActive { visitor: Entity },
    VisitorLeaving { t: f32, visitor: Entity },
    Done,
}

#[derive(Component)]
struct Visitor;

#[derive(Component)]
pub struct Ticket;

#[derive(Component)]
pub struct TicketButton;

fn cleanup_minigame(mut commands: Commands) {
    commands.remove_resource::<RemainingVisitors>();
    commands.remove_resource::<TicketsBase>();
    commands.remove_resource::<TicketsState>();
}

fn setup_minigame(mut commands: Commands, assets: Res<TicketAssets>) {
    let n = rand::rng().random_range(2..=4_u32);
    commands.insert_resource(RemainingVisitors(n));

    let base = setup_minigame_background(&mut commands, MiniGame::Tickets);
    commands.insert_resource(TicketsBase(base));

    commands.spawn((
        ChildOf(base),
        Sprite::from_image(assets.desk.clone()),
        Transform::from_translation(vec3(0., 0., 1.)).with_scale(Vec3::splat(0.75)),
    ));

    commands.spawn((
        ChildOf(base),
        Clickable,
        TicketButton,
        Sprite::from_image(assets.button.clone()),
        Transform::from_translation(vec3(-253., -104., 1.1)).with_scale(Vec3::splat(0.75)),
    ));

    let visitor = spawn_visitor(&mut commands, &assets.visitors, base);
    commands.insert_resource(TicketsState::VisitorEntering { t: 0., visitor });
}

fn spawn_visitor(commands: &mut Commands, images: &Vec<Handle<Image>>, base: Entity) -> Entity {
    let image = images.choose(&mut rand::rng()).unwrap().clone();

    commands
        .spawn((
            ChildOf(base),
            Visitor,
            Sprite::from_image(image),
            Transform::from_translation(vec3(VISITOR_ENTER_X, 100., 0.9)),
        ))
        .id()
}

fn tick_tickets(
    mut commands: Commands,
    assets: Res<TicketAssets>,
    base: Res<TicketsBase>,
    mut state: ResMut<TicketsState>,
    mut remaining: ResMut<RemainingVisitors>,
    time: Res<Time>,
    mut transforms: Query<&mut Transform>,
    mut clicked: MessageReader<ItemClicked>,
    mut dropped: MessageReader<ItemDropped>,
    buttons: Query<Entity, With<TicketButton>>,
    visitors: Query<Entity, With<Visitor>>,
    tickets: Query<Entity, With<Ticket>>,
    mut finished: MessageWriter<MinigameFinished>,
) {
    // Always consume messages to avoid stale buffering across states
    let mut button_clicked = false;
    for e in clicked.read() {
        if buttons.contains(e.item) {
            button_clicked = true;
        }
    }

    let mut dropped_ticket: Option<Entity> = None;
    for e in dropped.read() {
        if tickets.contains(e.item) && visitors.contains(e.in_zone) {
            dropped_ticket = Some(e.item);
        }
    }

    let next = match &mut *state {
        TicketsState::VisitorEntering { t, visitor } => {
            *t = (*t + time.delta_secs() / ANIM_SECS).min(1.0);
            if let Ok(mut tf) = transforms.get_mut(*visitor) {
                tf.translation.x = lerp(VISITOR_ENTER_X, VISITOR_CENTER_X, ease_out(*t));
            }
            if *t >= 1.0 {
                commands
                    .entity(*visitor)
                    .insert(DropZone(Vec2::new(300., 500.)));
                Some(TicketsState::WaitingForTicket { visitor: *visitor })
            } else {
                None
            }
        }
        TicketsState::WaitingForTicket { visitor } => {
            if button_clicked {
                commands.spawn((
                    ChildOf(base.0),
                    Ticket,
                    Draggable,
                    Sprite::from_image(assets.ticket.clone()),
                    Transform::from_translation(vec3(-250., 40., 3.)),
                ));

                commands.spawn(sound_effect(assets.ticket_sfx.clone()));

                Some(TicketsState::TicketActive { visitor: *visitor })
            } else {
                None
            }
        }
        TicketsState::TicketActive { visitor } => {
            if let Some(ticket_entity) = dropped_ticket {
                commands.entity(ticket_entity).despawn();
                commands.entity(*visitor).remove::<DropZone>();
                Some(TicketsState::VisitorLeaving {
                    t: 0.,
                    visitor: *visitor,
                })
            } else {
                None
            }
        }
        TicketsState::VisitorLeaving { t, visitor } => {
            *t = (*t + time.delta_secs() / ANIM_SECS).min(1.0);
            if let Ok(mut tf) = transforms.get_mut(*visitor) {
                tf.translation.x = lerp(VISITOR_CENTER_X, VISITOR_EXIT_X, ease_in(*t));
            }
            if *t >= 1.0 {
                commands.entity(*visitor).despawn();
                remaining.0 -= 1;
                if remaining.0 > 0 {
                    let next_visitor = spawn_visitor(&mut commands, &assets.visitors, base.0);
                    Some(TicketsState::VisitorEntering {
                        t: 0.,
                        visitor: next_visitor,
                    })
                } else {
                    finished.write(MinigameFinished {
                        game: MiniGame::Tickets,
                    });
                    Some(TicketsState::Done)
                }
            } else {
                None
            }
        }
        TicketsState::Done => None,
    };

    if let Some(new_state) = next {
        *state = new_state;
    }
}
