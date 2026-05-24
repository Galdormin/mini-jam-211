use std::ops::Range;

use bevy::{audio::Volume, prelude::*};
use rand::Rng;

const SOUND_EFFECT_VOLUME: f32 = -10.0; // dB

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        apply_global_volume.run_if(resource_changed::<GlobalVolume>),
    );
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Music;

/// A music audio instance.
pub fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::LOOP, Music)
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SoundEffect;

/// A sound effect audio instance.
pub fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN.with_volume(Volume::Decibels(SOUND_EFFECT_VOLUME)),
        SoundEffect,
    )
}

/// A random speed sound effect audio instance.
pub fn sound_effect_random_speed(
    handle: Handle<AudioSource>,
    speed_range: Range<f32>,
) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Decibels(SOUND_EFFECT_VOLUME))
            .with_speed(rand::rng().random_range(speed_range)),
        SoundEffect,
    )
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    global_volume: Res<GlobalVolume>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink)>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_volume(global_volume.volume * playback.volume);
    }
}
