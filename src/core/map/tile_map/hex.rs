use super::super::coord::hex;
use super::super::coord::Coord;
use super::super::tile::Tile;

#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct HexMapContext {
	pub width: u8,
	pub height: u8,
	pub wrap_x: bool,
}

pub struct HexMap {
	context: HexMapContext,
	tiles: Vec<Tile>,
}

impl HexMap {
	pub fn new(width: u8, height: u8, wrap_x: bool) -> HexMap {
		HexMap {
			context: HexMapContext {
				width: width,
				height: height,
				wrap_x: wrap_x,
			},
			tiles: std::iter::repeat(())
				.take(width as usize * height as usize)
				.map(|()| Tile::new())
				.collect(),
		}
	}
}

impl super::TileMap for HexMap {
	type CoordType = hex::HexCoord;

	fn get_tile(&self, coord: Self::CoordType) -> Option<&Tile> {
		Some(&self.tiles[coord.idx(&self.context)?])
	}

	fn get_tile_mut(&mut self, coord: Self::CoordType) -> Option<&mut Tile> {
		Some(&mut self.tiles[coord.idx(&self.context)?])
	}
}
