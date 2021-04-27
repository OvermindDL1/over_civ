mod cursive_runner;

use crate::universal::exit::RequestExit;
use bevy::app::PluginGroupBuilder;
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
			.add(cursive_runner::CursiveRunnerPlugin::default())
			.add(ClientTuiPlugin);
	}
}

impl Plugin for ClientTuiPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.insert_resource(cursive_runner::CursiveFPS::new(Duration::from_secs_f64(
			1.0 / 20.0,
		)))
		.add_system(exit_on_window_close.system());
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
