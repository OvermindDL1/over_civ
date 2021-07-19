pub mod hex;

pub trait Coord
where
	Self: std::marker::Sized,
{
	type MapContext;

	type RelativeCoord;

	type NeighborIteratorFull: Iterator<Item = Self>;
	type NeighborIteratorRing: Iterator<Item = Self>;

	fn from_linear(x: f32, y: f32) -> Self;
	fn to_linear(&self) -> (f32, f32);

	fn relative_position(&self, other: Self, map_context: &Self::MapContext)
		-> Self::RelativeCoord;
	fn distance_to(&self, other: &Self, map_context: &Self::MapContext) -> u8;
	fn offset_by(
		&self,
		offset: &Self::RelativeCoord,
		map_context: &Self::MapContext,
	) -> Option<Self>;

	fn is_technically_valid(&self, map_context: &Self::MapContext) -> bool;
	fn is_fully_valid(&self, map_context: &Self::MapContext) -> bool;

	// attempt to convert a coordinate that is technically correct to one that is fully correct
	// (such as through wrapping)
	fn revalidate(&self, map_context: &Self::MapContext) -> Option<Self>;

	// convert a coordinate into a number suitable as an index in an array
	fn idx(&self, map_context: &Self::MapContext) -> Option<usize>;

	fn iter_neighbors_full(
		&self,
		size: u8,
		map_context: &Self::MapContext,
	) -> Self::NeighborIteratorFull;
	fn iter_neighbors_ring(
		&self,
		size: u8,
		map_context: &Self::MapContext,
	) -> Self::NeighborIteratorRing;
}
