pub mod hex;

use super::tile::Tile;

pub trait TileMap {
	type CoordType: super::coord::Coord;

	fn get_tile(&self, coord: Self::CoordType) -> Option<&Tile>;
	fn get_tile_mut(&mut self, coord: Self::CoordType) -> Option<&mut Tile>;
}
