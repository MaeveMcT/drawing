use std::path::Path;

use log::{debug, error, info};
use raylib::{
    automation::{AutomationEvent, AutomationEventList},
    RaylibHandle,
};

use crate::{app::TestSettings, persistence::save, state::State};

pub fn load_replay(
    replay_path: &Path,
    rl: &RaylibHandle,
    automation_events_list: &mut AutomationEventList,
    automation_events: &mut Vec<AutomationEvent>,
) -> Option<()> {
    debug!("Trying to load replay from {:?}", replay_path);
    let loaded_automated_events = rl.load_automation_event_list(Some(replay_path.into()));
    if loaded_automated_events.count() == 0 {
        // Load unsuccessful
        // TODO: Show failure on UI
        error!(
            "Couldn't load automated event list from {}, or it was empty",
            replay_path.display()
        );
        return None;
    } else {
        // TODO: Does this leak memory?
        *automation_events_list = loaded_automated_events;
        rl.set_automation_event_list(automation_events_list);
        rl.set_automation_event_base_frame(0);

        *automation_events = automation_events_list.events();

        // TODO: Show success on UI
        info!(
            "Successfully loaded automated event list from {}",
            replay_path.display(),
        );
        return Some(());
    }
}

pub fn play_replay(state: &mut State) {
    state.is_playing_inputs = true;
    // TODO: Reset camera state etc
    state.current_play_frame = 0;
    state.play_frame_counter = 0;
}

pub fn stop_replay(state: &mut State) {
    state.is_playing_inputs = false;
    state.current_play_frame = 0;
    state.play_frame_counter = 0;
}

/// Returns true if the program should exit
pub fn replay_inputs(
    state: &mut State,
    test_options: &Option<TestSettings>,
    automation_events: &Vec<AutomationEvent>,
) -> bool {
    // NOTE: Multiple events could be executed in a single frame
    while state.play_frame_counter == automation_events[state.current_play_frame].frame() as usize {
        let event = &automation_events[state.current_play_frame];
        debug!(
            "Event {:?}: type {:?}",
            state.current_play_frame,
            event.get_type()
        );

        event.play();
        state.current_play_frame += 1;

        if state.current_play_frame == automation_events.len() {
            stop_replay(state);
            info!("Finished playing replay");
            if let Some(ref test_options) = test_options {
                if test_options.save_after_replay {
                    info!("Attempting to save since replay has finished");
                    match save(&state, &test_options.save_path) {
                        Ok(_) => {
                            info!("Successfully saved to {}", test_options.save_path.display())
                        }
                        Err(e) => error!(
                            "Failed to save to {}: {}",
                            test_options.save_path.display(),
                            e
                        ),
                    }
                } else {
                    info!("Not saving - Save after replay finishes has been disabled");
                }

                if test_options.quit_after_replay {
                    info!("Quitting - Quit after replay is enabled");
                    return true;
                }
            }
            break;
        }
    }
    state.play_frame_counter += 1;
    return false;
}
