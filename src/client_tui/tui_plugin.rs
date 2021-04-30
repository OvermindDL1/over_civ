use bevy::app::{AppExit, Events};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::window::{WindowCommand, WindowId, WindowMode, WindowResizeConstraints};
use crossterm::event::{KeyModifiers, MouseEventKind};
use crossterm::execute;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::Terminal;

type TerminalRes = Terminal<CrosstermBackend<std::io::Stdout>>;

pub struct TuiMaxEventsPerTick(usize);

pub struct TuiRunnerPlugin {
	pub max_events_per_tick: TuiMaxEventsPerTick,
}

impl Default for TuiRunnerPlugin {
	fn default() -> Self {
		Self {
			max_events_per_tick: TuiMaxEventsPerTick(128),
		}
	}
}

impl TuiRunnerPlugin {
	#[allow(dead_code)]
	pub fn max_events_per_tick(self, max_events_per_tick: usize) -> Self {
		let max_events_per_tick = TuiMaxEventsPerTick(max_events_per_tick);
		Self {
			max_events_per_tick,
			..self
		}
	}
}

impl Plugin for TuiRunnerPlugin {
	fn build(&self, app: &mut AppBuilder) {
		trace!("Registering TUI Plugin");

		crossterm::terminal::enable_raw_mode()
			.expect("must be able to set raw mode for stdout for the TUI");
		let stdout = std::io::stdout();
		let backend = CrosstermBackend::new(stdout);
		let mut term: TerminalRes = Terminal::new(backend).expect("unable to initialize TUI");

		let (x, y) =
			crossterm::terminal::size().expect("unable to access terminal size information");
		let window_descriptor = WindowDescriptor {
			width: x as _,
			height: y as _,
			resize_constraints: WindowResizeConstraints {
				min_width: 0.0,
				min_height: 0.0,
				max_width: u16::MAX as _,
				max_height: u16::MAX as _,
			},
			scale_factor_override: None,
			title: "".to_string(),
			vsync: false,
			resizable: true,
			decorations: false,
			cursor_visible: true,
			cursor_locked: false,
			mode: WindowMode::Fullscreen { use_size: true },
		};

		let primary_window = Window::new(
			WindowId::primary(),
			&window_descriptor,
			x as _,
			y as _,
			1.0,
			None,
		);

		app.world_mut()
			.get_resource_mut::<Windows>()
			.expect("the `Windows` resource must have been inserted before `TuiPlugin` is added")
			.add(primary_window);

		app.world_mut()
			.get_resource_or_insert_with(|| TuiMaxEventsPerTick(128));

		let (x, y) = term.get_cursor().unwrap_or((0, 0));

		app.insert_non_send_resource(term)
			.insert_resource(CursorLocation(x, y))
			.add_system_to_stage(CoreStage::First, event_poller.exclusive_system())
			.add_system_to_stage(CoreStage::Last, change_window.exclusive_system())
			.add_system_to_stage(CoreStage::Last, reset_on_exit.system());
	}
}

pub struct CursorLocation(u16, u16);

//fn event_poller(max_events_per_tick: Res<TuiMaxEventsPerTick>, mut windows: ResMut<Windows>, mut mouse: EventWriter<Mou>) {
fn event_poller(world: &mut World) {
	let world = world.cell();
	let mut max_events = world.get_resource_mut::<TuiMaxEventsPerTick>().unwrap().0;
	let mut windows = world.get_resource_mut::<Windows>().unwrap();
	let mut term = world.get_resource_mut::<TerminalRes>().unwrap();
	let mut resized: Option<(u16, u16)> = None;

	while max_events != 0 {
		max_events -= 1;
		if crossterm::event::poll(Duration::from_nanos(0)).expect("failed polling for events") {
			let event = crossterm::event::read()
				.expect("an event was ready to read but vanished before it was read?");
			use crossterm::event::Event;
			match event {
				Event::Key(key) => {
					trace!("TUI key event: {:?}", key);
					let key_code: &[KeyCode] = match key.code {
						crossterm::event::KeyCode::Backspace => &[KeyCode::Back],
						crossterm::event::KeyCode::Enter => &[KeyCode::Return],
						crossterm::event::KeyCode::Left => &[KeyCode::Left],
						crossterm::event::KeyCode::Right => &[KeyCode::Right],
						crossterm::event::KeyCode::Up => &[KeyCode::Up],
						crossterm::event::KeyCode::Down => &[KeyCode::Down],
						crossterm::event::KeyCode::Home => &[KeyCode::Home],
						crossterm::event::KeyCode::End => &[KeyCode::End],
						crossterm::event::KeyCode::PageUp => &[KeyCode::PageUp],
						crossterm::event::KeyCode::PageDown => &[KeyCode::PageDown],
						crossterm::event::KeyCode::Tab => &[KeyCode::Tab],
						crossterm::event::KeyCode::BackTab => &[KeyCode::LShift, KeyCode::Tab],
						crossterm::event::KeyCode::Delete => &[KeyCode::Delete],
						crossterm::event::KeyCode::Insert => &[KeyCode::Insert],
						crossterm::event::KeyCode::F(id) => match id {
							1 => &[KeyCode::F1],
							2 => &[KeyCode::F2],
							3 => &[KeyCode::F3],
							4 => &[KeyCode::F4],
							5 => &[KeyCode::F5],
							6 => &[KeyCode::F6],
							7 => &[KeyCode::F7],
							8 => &[KeyCode::F8],
							9 => &[KeyCode::F9],
							10 => &[KeyCode::F10],
							11 => &[KeyCode::F11],
							12 => &[KeyCode::F12],
							13 => &[KeyCode::F13],
							14 => &[KeyCode::F14],
							15 => &[KeyCode::F15],
							16 => &[KeyCode::F16],
							17 => &[KeyCode::F17],
							18 => &[KeyCode::F18],
							19 => &[KeyCode::F19],
							20 => &[KeyCode::F20],
							21 => &[KeyCode::F21],
							22 => &[KeyCode::F22],
							23 => &[KeyCode::F23],
							24 => &[KeyCode::F24],
							_ => {
								warn!("unhandled F# key ID: {}", id);
								&[]
							}
						},
						crossterm::event::KeyCode::Char(c) => match c {
							'A' | 'a' => &[KeyCode::A],
							'B' | 'b' => &[KeyCode::B],
							'C' | 'c' => &[KeyCode::C],
							'D' | 'd' => &[KeyCode::D],
							'E' | 'e' => &[KeyCode::E],
							'F' | 'f' => &[KeyCode::F],
							'G' | 'g' => &[KeyCode::G],
							'H' | 'h' => &[KeyCode::H],
							'I' | 'i' => &[KeyCode::I],
							'J' | 'j' => &[KeyCode::J],
							'K' | 'k' => &[KeyCode::K],
							'L' | 'l' => &[KeyCode::L],
							'M' | 'm' => &[KeyCode::M],
							'N' | 'n' => &[KeyCode::N],
							'O' | 'o' => &[KeyCode::O],
							'P' | 'p' => &[KeyCode::P],
							'Q' | 'q' => &[KeyCode::Q],
							'R' | 'r' => &[KeyCode::R],
							'S' | 's' => &[KeyCode::S],
							'T' | 't' => &[KeyCode::T],
							'U' | 'u' => &[KeyCode::U],
							'V' | 'v' => &[KeyCode::V],
							'W' | 'w' => &[KeyCode::W],
							'X' | 'x' => &[KeyCode::X],
							'Y' | 'y' => &[KeyCode::Y],
							'Z' | 'z' => &[KeyCode::Z],
							'0' => &[KeyCode::Key0],
							'1' => &[KeyCode::Key1],
							'2' => &[KeyCode::Key2],
							'3' => &[KeyCode::Key3],
							'4' => &[KeyCode::Key4],
							'5' => &[KeyCode::Key5],
							'6' => &[KeyCode::Key6],
							'7' => &[KeyCode::Key7],
							'8' => &[KeyCode::Key8],
							'9' => &[KeyCode::Key9],
							_ => {
								error!("unhandled keyboard char code in TUI: {}", c);
								&[]
							}
						},
						crossterm::event::KeyCode::Null => &[],
						crossterm::event::KeyCode::Esc => &[KeyCode::Escape],
					};
					// TODO:  Still need to add in modifiers as well from `key.modifiers`
					let modifier_code: &[KeyCode] = {
						if key.modifiers.contains(KeyModifiers::SHIFT) {
							if key.modifiers.contains(KeyModifiers::CONTROL) {
								if key.modifiers.contains(KeyModifiers::ALT) {
									&[KeyCode::LShift, KeyCode::LControl, KeyCode::LAlt]
								} else {
									&[KeyCode::LShift, KeyCode::LControl]
								}
							} else {
								if key.modifiers.contains(KeyModifiers::ALT) {
									&[KeyCode::LShift, KeyCode::LAlt]
								} else {
									&[KeyCode::LShift]
								}
							}
						} else {
							if key.modifiers.contains(KeyModifiers::CONTROL) {
								if key.modifiers.contains(KeyModifiers::ALT) {
									&[KeyCode::LControl, KeyCode::LAlt]
								} else {
									&[KeyCode::LControl]
								}
							} else {
								if key.modifiers.contains(KeyModifiers::ALT) {
									&[KeyCode::LAlt]
								} else {
									&[]
								}
							}
						}
					};
					let modifier_iter = modifier_code.iter().zip(std::iter::repeat(false));
					let keys_iter = key_code.iter().zip(std::iter::repeat(false));
					let keys_rev_iter = key_code.iter().rev().zip(std::iter::repeat(true));
					let modifier_rev_iter = modifier_code.iter().rev().zip(std::iter::repeat(true));
					let keys = modifier_iter
						.chain(keys_iter)
						.chain(keys_rev_iter)
						.chain(modifier_rev_iter);
					let keys = keys.map(|(code, is_released)| KeyboardInput {
						scan_code: 0,
						key_code: Some(*code),
						state: if is_released {
							ElementState::Released
						} else {
							ElementState::Pressed
						},
					});
					let keys = keys.map(|k| dbg!(k));
					let mut input = world
						.get_resource_mut::<Events<KeyboardInput>>()
						.unwrap()
						.extend(keys);
					// TODO:  Remove this test code AppExit
					if key.code == crossterm::event::KeyCode::Esc {
						let mut exit = world.get_resource_mut::<Events<AppExit>>().unwrap();
						exit.send(AppExit);
					}
				}
				Event::Mouse(mouse_event) => {
					trace!("TUI mouse event: {:?}", mouse_event);

					let x = mouse_event.column as f32;
					let y = mouse_event.row as f32;
					let mut mouse = world.get_resource_mut::<Events<MouseMotion>>().unwrap();
					let mut old_loc = world.get_resource_mut::<CursorLocation>().unwrap();
					let old_x = old_loc.0 as f32;
					let old_y = old_loc.1 as f32;
					mouse.send(MouseMotion {
						delta: Vec2::new(x - old_x, y - old_y),
					});
					old_loc.0 = mouse_event.column;
					old_loc.1 = mouse_event.row;

					match mouse_event.kind {
						MouseEventKind::Down(button) => {
							let button = match button {
								crossterm::event::MouseButton::Left => MouseButton::Left,
								crossterm::event::MouseButton::Right => MouseButton::Right,
								crossterm::event::MouseButton::Middle => MouseButton::Middle,
							};
							let mut mouse = world
								.get_resource_mut::<Events<MouseButtonInput>>()
								.unwrap();
							mouse.send(MouseButtonInput {
								button,
								state: ElementState::Pressed,
							});
						}
						MouseEventKind::Up(button) => {
							let button = match button {
								crossterm::event::MouseButton::Left => MouseButton::Left,
								crossterm::event::MouseButton::Right => MouseButton::Right,
								crossterm::event::MouseButton::Middle => MouseButton::Middle,
							};
							let mut mouse = world
								.get_resource_mut::<Events<MouseButtonInput>>()
								.unwrap();
							mouse.send(MouseButtonInput {
								button,
								state: ElementState::Released,
							});
						}
						MouseEventKind::Drag(_button) => {}
						MouseEventKind::Moved => {}
						MouseEventKind::ScrollDown => {
							let mut mouse = world.get_resource_mut::<Events<MouseWheel>>().unwrap();
							mouse.send(MouseWheel {
								unit: MouseScrollUnit::Line,
								x: 0.0,
								y: -1.0,
							})
						}
						MouseEventKind::ScrollUp => {
							let mut mouse = world.get_resource_mut::<Events<MouseWheel>>().unwrap();
							mouse.send(MouseWheel {
								unit: MouseScrollUnit::Line,
								x: 0.0,
								y: 1.0,
							})
						}
					}
				}
				Event::Resize(x, y) => resized = Some((x, y)),
			}
		} else {
			break;
		}
	}

	if let Some((x, y)) = resized {
		let window = windows
			.get_primary_mut()
			.expect("primary window was somehow missing");
		window.update_actual_size_from_backend(x as u32, y as u32);
		info!("tui resized to: {}:{}", x, y);
	}
}

fn change_window(world: &mut World) {
	let world = world.cell();
	let mut windows = world.get_resource_mut::<Windows>().unwrap();
	let mut term = world.get_non_send_mut::<TerminalRes>().unwrap();

	for bevy_window in windows.iter_mut() {
		let id = bevy_window.id();
		assert_eq!(
			id,
			WindowId::primary(),
			"only a single `primary` window is allowed in the TUI"
		);
		for command in bevy_window.drain_commands() {
			match command {
				WindowCommand::SetTitle { title } => {
					if let Err(e) = set_title(&title) {
						error!("failed to set TUI title: {:?}", e)
					}
				}
				WindowCommand::SetCursorLockMode { .. } => {
					todo!("set cursor lock mode")
				}
				WindowCommand::SetCursorVisibility { visible } => {
					if let Err(e) = if visible {
						term.show_cursor()
					} else {
						term.hide_cursor()
					} {
						error!("failed to set cursor visibility for TUI: {:?}", e)
					}
				}
				WindowCommand::SetCursorPosition { position } => {
					if let Err(e) = term.set_cursor(position.x as u16, position.y as u16) {
						error!("failed to set cursor position for TUI: {:?}", e)
					}
				}
				unsupported_cmd => {
					error!(
						"unsupported WindowCommand on Window `{}`: {:?}",
						id, unsupported_cmd
					)
				}
			}
		}
	}
}

fn reset_on_exit(mut exiting: EventReader<AppExit>, mut term: NonSendMut<TerminalRes>) {
	if exiting.iter().next().is_some() {
		// Ignore the results of these in case `stdout` is already dead
		let _ignore_show_mouse = term.show_cursor();
		let _ignore_disable_raw_mode = crossterm::terminal::disable_raw_mode();
		let _ignore_erase_title = execute!(std::io::stdout(), crossterm::terminal::SetTitle(""));
	}
}

// Helpers

fn set_title(title: &str) -> Result<(), crossterm::ErrorKind> {
	execute!(std::io::stdout(), crossterm::terminal::SetTitle(title))
}
