use bevy::app::{App, AppBuilder, AppExit, Events, ManualEventReader, Plugin};
use bevy::log::*;
use std::time::{Duration, Instant};

use bevy::window::{WindowCloseRequested, WindowId};
use cursive::event::Event;
use cursive::{CursiveRunnable, CursiveRunner};

pub struct CursiveFPS {
	pub preferred_frame_delta: Duration,
	last_frame_time: Option<Instant>,
}

impl CursiveFPS {
	pub fn new(preferred_frame_delta: Duration) -> Self {
		Self {
			preferred_frame_delta,
			last_frame_time: None,
		}
	}
}

#[derive(Default)]
pub struct CursiveRunnerPlugin();

impl Plugin for CursiveRunnerPlugin {
	fn build(&self, app: &mut AppBuilder) {
		trace!("Creating Cursive TUI instance");

		app.set_runner(cursive_runner);
	}
}

fn cursive_runner(mut app: App) {
	let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();

	let mut siv = cursive::default()
		.try_into_runner()
		.expect("unable to create TUI cursive runner");

	// Wipe the screen
	siv.refresh();

	while let Some(delay) = tick(&mut app, &mut siv, &mut app_exit_event_reader) {
		std::thread::sleep(delay);
	}
}

fn tick(
	app: &mut App,
	siv: &mut CursiveRunner<CursiveRunnable>,
	app_exit_event_reader: &mut ManualEventReader<AppExit>,
) -> Option<Duration> {
	let mut ret = if let Some(mut fps) = app.world.get_resource_mut::<CursiveFPS>() {
		let preferred_frame_delta = fps.preferred_frame_delta;
		let now = Instant::now();
		if let Some(last_frame_time) = &mut fps.last_frame_time {
			let delta = now - *last_frame_time;
			if delta < preferred_frame_delta {
				return Some(delta);
			}
			*last_frame_time = now;
			Some(delta - preferred_frame_delta)
		} else {
			fps.last_frame_time = Some(now);
			Some(preferred_frame_delta)
		}
	} else {
		Some(Duration::from_secs(0))
	};

	if let Some(app_exit_events) = app.world.get_resource_mut::<Events<AppExit>>() {
		if app_exit_event_reader
			.iter(&app_exit_events)
			.next_back()
			.is_some()
		{
			ret = None;
		}
	}

	let was_running = siv.is_running();
	if was_running {
		siv.process_events();
		if !siv.is_running() {
			if let Some(mut close_requested) =
				app.world.get_resource_mut::<Events<WindowCloseRequested>>()
			{
				close_requested.send(WindowCloseRequested {
					id: WindowId::primary(),
				});
			}
		}
	}
	app.update();
	if was_running {
		siv.on_event(Event::Refresh);
		siv.refresh();
	}

	ret
}
