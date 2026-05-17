use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAudio>()
            .add_systems(Startup, load_audio)
            .add_systems(Update, play_event_sounds);
    }
}

#[derive(Resource, Default)]
struct GameAudio {
    click: Handle<AudioSource>,
    turn_end: Handle<AudioSource>,
    combat: Handle<AudioSource>,
    planet_voice: Handle<AudioSource>,
}

fn load_audio(mut audio: ResMut<GameAudio>, asset_server: Res<AssetServer>) {
    audio.click = asset_server.load("audio/ui_click.ogg");
    audio.turn_end = asset_server.load("audio/turn_end.ogg");
    audio.combat = asset_server.load("audio/combat.ogg");
    audio.planet_voice = asset_server.load("audio/planet_voice.ogg");
}

fn play_event_sounds(
    mut commands: Commands,
    game_state: Res<crate::GameStateResource>,
    audio: Res<GameAudio>,
    mut last_log_len: Local<usize>,
) {
    let current_log = &game_state.0.log;
    if current_log.len() > *last_log_len {
        for entry in &current_log[*last_log_len..] {
            match entry.category {
                smac_core::EventCategory::Narrative => {
                    commands.spawn(AudioBundle {
                        source: audio.planet_voice.clone(),
                        ..default()
                    });
                }
                smac_core::EventCategory::Crisis => {
                    commands.spawn(AudioBundle {
                        source: audio.combat.clone(),
                        ..default()
                    });
                }
                _ => {}
            }
        }
        *last_log_len = current_log.len();
    }
}
