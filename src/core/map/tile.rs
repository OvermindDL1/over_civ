use std::collections::HashSet;

use bevy::ecs::prelude::Entity;

use super::coord::Coord;

pub struct Tile {
	entities: HashSet<Entity>,
}

impl Tile {
	pub fn new() -> Tile {
		Tile {
			entities: HashSet::default(),
		}
	}
}
