mod tui_plugin;

use crate::universal::exit::RequestExit;
use bevy::app::{PluginGroupBuilder, ScheduleRunnerSettings};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::window::WindowCloseRequested;
use std::time::Duration;

#[derive(Default)]
pub struct ClientTuiPluginGroup;

struct ClientTuiPlugin;

impl PluginGroup for ClientTuiPluginGroup {
	fn build(&mut self, group: &mut PluginGroupBuilder) {
		group
			.add(bevy::audio::AudioPlugin::default())
			.add(bevy::gilrs::GilrsPlugin::default())
			.add(bevy::app::ScheduleRunnerPlugin::default())
			.add(
				tui_plugin::TuiRunnerPlugin::default()
					.title(env!("CARGO_PKG_NAME"))
					.start_in_raw_mode(true)
					.start_with_mouse_captured(true)
					.enable_alternate_screen(true)
					.max_events_per_tick(128),
			)
			.add(ClientTuiPlugin)
			.add(bevy::app::ScheduleRunnerPlugin::default());
	}
}

impl Plugin for ClientTuiPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
			1.0 / 20.0,
		)))
		.add_system(exit_on_window_close.system())
		.add_system(exit_on_escape.system());
	}
}

fn exit_on_escape(mut keys: EventReader<KeyboardInput>, mut exit: EventWriter<RequestExit>) {
	for key in keys.iter() {
		if key.key_code == Some(KeyCode::Escape) && key.state == ElementState::Released {
			trace!("escape pressed to request exit");
			exit.send(RequestExit);
		}
	}
}

fn exit_on_window_close(
	mut windows_closed: EventReader<WindowCloseRequested>,
	mut exit: EventWriter<RequestExit>,
) {
	// We only support a single window currently, change this if that changes
	if let Some(window_closed) = windows_closed.iter().next() {
		trace!("Window closed `{:?}`: exiting", window_closed.id);
		exit.send(RequestExit);
	}
}
