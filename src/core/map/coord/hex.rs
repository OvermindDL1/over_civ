use std::ops::{Add, Neg, Sub};

use super::super::tile_map::hex;
use super::Coord;

#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct HexCoord(u8, u8);

impl HexCoord {
	pub const CENTER_TO_POINT: f32 = 0.57735026; // 0.5773502691896258; //0.5/(TAU/12.0).cos(); // `cos` is not const capable for some reason...;
	const SQRT3: f32 = 1.7320508; // 1.732050807568877; //3.0f32.sqrt(); // `sqrt` is not const capable either, why?!

	pub fn new_axial(q: u8, r: u8) -> HexCoord {
		HexCoord(q, r)
	}

	pub fn q(&self) -> u8 {
		self.0
	}

	pub fn r(&self) -> u8 {
		self.1
	}

	pub fn to_axial_tuple(&self) -> (u8, u8) {
		(self.q(), self.r())
	}

	pub fn x(&self) -> i16 {
		self.0 as i16
	}

	pub fn y(&self) -> i16 {
		self.x().wrapping_neg().wrapping_sub(self.z())
	}

	pub fn z(&self) -> i16 {
		self.1 as i16
	}

	pub fn to_cubic_tuple(&self) -> (i16, i16, i16) {
		(self.x(), self.y(), self.z())
	}
}

impl Coord for HexCoord {
	type MapContext = hex::HexMapContext;

	type RelativeCoord = RelativeHexCoord;

	type NeighborIteratorFull = HexCoordIteratorFull;
	type NeighborIteratorRing = HexCoordIteratorRing;

	fn from_linear(x: f32, y: f32) -> Self {
		let s3 = 3.0f32.sqrt();
		let segment = (x + s3 * y + 1.0).floor();
		let q = (((2.0 * x + 1.0).floor() + segment) / 3.0).floor();
		let r = ((segment + (-x + s3 * y + 1.0).floor()) / 3.0).floor();
		HexCoord::new_axial((q - r) as i16 as u8, r as i16 as u8)
	}

	fn to_linear(&self) -> (f32, f32) {
		let q = self.0 as f32;
		let r = self.1 as f32;
		let x = Self::CENTER_TO_POINT * (Self::SQRT3 * q + Self::SQRT3 / 2.0 * r);
		let y = Self::CENTER_TO_POINT * (3.0 / 2.0 * r);
		(x, y)
	}

	fn relative_position(
		&self,
		other: Self,
		map_context: &Self::MapContext,
	) -> Self::RelativeCoord {
		unimplemented!()
	}

	fn distance_to(&self, other: &Self, map_context: &Self::MapContext) -> u8 {
		let (dx, dy, dz) = (*self - *other).to_cubic_tuple();
		std::cmp::max(
			std::cmp::max(dx.abs() as u8, dy.abs() as u8),
			dz.abs() as u8,
		)
	}

	fn offset_by(
		&self,
		offset: &Self::RelativeCoord,
		map_context: &Self::MapContext,
	) -> Option<Self> {
		let width = map_context.width as isize + 1;
		let height = map_context.height as isize + 1;
		let mut q = self.0 as isize + offset.0 as isize;
		let r = self.1 as isize + offset.1 as isize;
		if r < 0 || r > height {
			return None;
		}
		let r = r as u8;
		if map_context.wrap_x {
			q = q.rem_euclid(width);
		} else if q < 0 || q > width {
			return None;
		}
		let q = q as u8;
		Some(HexCoord::new_axial(q, r))
	}

	fn is_technically_valid(&self, map_context: &Self::MapContext) -> bool {
		!(self.0 < 0
			|| (self.0 > map_context.width && !map_context.wrap_x)
			|| self.1 < 0
			|| self.1 > map_context.height)
	}

	fn is_fully_valid(&self, map_context: &Self::MapContext) -> bool {
		!(self.0 < 0 || self.0 > map_context.width || self.1 < 0 || self.1 > map_context.height)
	}

	// attempt to convert a coordinate that is technically correct to one that is fully correct
	// (such as through wrapping)
	fn revalidate(&self, map_context: &Self::MapContext) -> Option<Self> {
		if self.is_fully_valid(map_context) {
			Some(*self)
		} else if self.is_technically_valid(map_context) && map_context.wrap_x {
			let (q, r) = self.to_axial_tuple();
			let nq = (q as u16).rem_euclid(map_context.width as u16 + 1) as u8;
			Some(HexCoord(nq, r))
		} else {
			None
		}
	}

	// convert a coordinate into a number suitable as an index in an array
	fn idx(&self, map_context: &Self::MapContext) -> Option<usize> {
		let valid_tile = self.revalidate(map_context)?;
		Some((valid_tile.0 as usize) + (valid_tile.1 as usize) * map_context.width as usize)
	}

	fn iter_neighbors_full(
		&self,
		size: u8,
		map_context: &Self::MapContext,
	) -> Self::NeighborIteratorFull {
		Self::NeighborIteratorFull::new(*self, size)
	}

	fn iter_neighbors_ring(
		&self,
		size: u8,
		map_context: &Self::MapContext,
	) -> Self::NeighborIteratorRing {
		Self::NeighborIteratorRing::new(*self, size)
	}
}

impl Add<RelativeHexCoord> for HexCoord {
	type Output = HexCoord;

	fn add(self, rhs: RelativeHexCoord) -> Self::Output {
		HexCoord(
			(self.0 as i8).wrapping_add(rhs.0) as u8,
			(self.1 as i8).wrapping_add(rhs.1) as u8,
		)
	}
}

impl Sub<RelativeHexCoord> for HexCoord {
	type Output = HexCoord;

	fn sub(self, rhs: RelativeHexCoord) -> Self::Output {
		HexCoord(
			(self.0 as i8).wrapping_sub(rhs.0) as u8,
			(self.1 as i8).wrapping_sub(rhs.1) as u8,
		)
	}
}

impl Sub<HexCoord> for HexCoord {
	type Output = RelativeHexCoord;

	fn sub(self, rhs: HexCoord) -> Self::Output {
		RelativeHexCoord(
			self.0.wrapping_sub(rhs.0) as i8,
			self.1.wrapping_sub(rhs.1) as i8,
		)
	}
}

pub struct HexCoordIteratorRing {
	center: HexCoord,
	offset: RelativeHexCoordIteratorRing,
}

impl HexCoordIteratorRing {
	fn new(center: HexCoord, distance: u8) -> HexCoordIteratorRing {
		HexCoordIteratorRing {
			center,
			offset: RelativeHexCoordIteratorRing::new(distance),
		}
	}
}

impl Iterator for HexCoordIteratorRing {
	type Item = HexCoord;

	fn next(&mut self) -> Option<Self::Item> {
		let offset = self.offset.next()?;
		Some(self.center + offset)
	}
}

pub struct HexCoordIteratorFull {
	center: HexCoord,
	offset: RelativeHexCoordIteratorFull,
}

impl HexCoordIteratorFull {
	fn new(center: HexCoord, distance: u8) -> HexCoordIteratorFull {
		HexCoordIteratorFull {
			center,
			offset: RelativeHexCoordIteratorFull::new(distance),
		}
	}
}

impl Iterator for HexCoordIteratorFull {
	type Item = HexCoord;

	fn next(&mut self) -> Option<Self::Item> {
		let offset = self.offset.next()?;
		Some(self.center + offset)
	}
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct RelativeHexCoord(i8, i8);

impl RelativeHexCoord {
	pub fn new_axial(q: i8, r: i8) -> RelativeHexCoord {
		RelativeHexCoord(q, r)
	}

	pub fn q(&self) -> i8 {
		self.0
	}

	pub fn r(&self) -> i8 {
		self.1
	}

	pub fn to_axial_tuple(&self) -> (i8, i8) {
		(self.q(), self.r())
	}

	pub fn x(&self) -> i8 {
		self.0
	}

	pub fn y(&self) -> i8 {
		self.0.wrapping_neg().wrapping_sub(self.1)
	}

	pub fn z(&self) -> i8 {
		self.1
	}

	pub fn to_cubic_tuple(&self) -> (i8, i8, i8) {
		(self.x(), self.y(), self.z())
	}

	pub fn to_linear(self) -> (f32, f32) {
		let q = self.0 as f32;
		let r = self.1 as f32;
		let x = HexCoord::CENTER_TO_POINT * (HexCoord::SQRT3 * q + HexCoord::SQRT3 / 2.0 * r);
		let y = HexCoord::CENTER_TO_POINT * (3.0 / 2.0 * r);
		(x, y)
	}

	pub fn distance_to(self, other: RelativeHexCoord) -> u8 {
		let (dx, dy, dz) = (self - other).to_cubic_tuple();
		std::cmp::max(
			std::cmp::max(dx.abs() as u8, dy.abs() as u8),
			dz.abs() as u8,
		)
	}

	pub fn scale(self, scale: i8) -> RelativeHexCoord {
		RelativeHexCoord(self.0.wrapping_mul(scale), self.1.wrapping_mul(scale))
	}

	pub fn cw(self) -> RelativeHexCoord {
		let (_x, y, z) = (-self).to_cubic_tuple();
		// RelativeHexCoord::new_cubic(z, x, y)
		RelativeHexCoord::new_axial(z, y)
	}

	pub fn ccw(self) -> RelativeHexCoord {
		let (x, y, _z) = (-self).to_cubic_tuple();
		// RelativeHexCoord::new_cubic(y, z, x)
		RelativeHexCoord::new_axial(y, x)
	}

	// pub fn as_coord(self) -> Coord {
	// 	Coord(self.0, self.1)
	// }

	pub fn iter_neighbors_ring(distance: u8) -> RelativeHexCoordIteratorRing {
		RelativeHexCoordIteratorRing::new(distance)
	}

	pub fn iter_neighbors(distance: u8) -> RelativeHexCoordIteratorFull {
		RelativeHexCoordIteratorFull::new(distance)
	}
}

impl Add for RelativeHexCoord {
	type Output = RelativeHexCoord;

	fn add(self, rhs: Self) -> Self::Output {
		RelativeHexCoord(self.0.wrapping_add(rhs.0), self.1.wrapping_add(rhs.1))
	}
}

impl Sub for RelativeHexCoord {
	type Output = RelativeHexCoord;

	fn sub(self, rhs: Self) -> Self::Output {
		RelativeHexCoord(self.0.wrapping_sub(rhs.0), self.1.wrapping_sub(rhs.1))
	}
}

impl Add<HexCoord> for RelativeHexCoord {
	type Output = HexCoord;

	fn add(self, rhs: HexCoord) -> Self::Output {
		HexCoord(
			(self.0 as u8).wrapping_add(rhs.0),
			(self.1 as u8).wrapping_add(rhs.1),
		)
	}
}

impl Sub<HexCoord> for RelativeHexCoord {
	type Output = HexCoord;

	fn sub(self, rhs: HexCoord) -> Self::Output {
		HexCoord(
			(self.0 as u8).wrapping_sub(rhs.0),
			(self.1 as u8).wrapping_sub(rhs.1),
		)
	}
}

impl Neg for RelativeHexCoord {
	type Output = RelativeHexCoord;

	fn neg(self) -> Self::Output {
		RelativeHexCoord(self.0.wrapping_neg(), self.1.wrapping_neg())
	}
}

pub struct RelativeHexCoordIteratorRing {
	side: RelativeHexCoord,
	side_count: u8,
	distance: u8,
	offset: u8,
}

impl RelativeHexCoordIteratorRing {
	pub fn new(distance: u8) -> RelativeHexCoordIteratorRing {
		assert!(distance <= 127);
		if distance == 0 {
			RelativeHexCoordIteratorRing {
				side_count: 5,
				side: RelativeHexCoord(0, 0),
				distance: 0,
				offset: 0,
			}
		} else {
			RelativeHexCoordIteratorRing {
				side_count: 0,
				side: RelativeHexCoord(1, 0),
				distance,
				offset: 0,
			}
		}
	}
}

impl Iterator for RelativeHexCoordIteratorRing {
	type Item = RelativeHexCoord;

	fn next(&mut self) -> Option<Self::Item> {
		if self.side_count > 5 {
			return None;
		}
		let side = self.side.scale(self.distance as i8);
		let offset = (-self.side).ccw().scale(self.offset as i8);
		self.offset += 1;
		if self.offset >= self.distance {
			self.offset = 0;
			self.side = self.side.cw();
			self.side_count += 1;
		}
		Some(side + offset)
	}
}

pub struct RelativeHexCoordIteratorFull {
	ring_iter: RelativeHexCoordIteratorRing,
	distance: u8,
}

impl RelativeHexCoordIteratorFull {
	pub fn new(distance: u8) -> RelativeHexCoordIteratorFull {
		RelativeHexCoordIteratorFull {
			ring_iter: RelativeHexCoordIteratorRing::new(0),
			distance,
		}
	}
}

impl Iterator for RelativeHexCoordIteratorFull {
	type Item = RelativeHexCoord;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(coord) = self.ring_iter.next() {
			return Some(coord);
		}
		if self.distance <= self.ring_iter.distance {
			return None;
		}
		self.ring_iter = RelativeHexCoordIteratorRing::new(self.ring_iter.distance + 1);
		self.ring_iter.next()
	}
}

#[cfg(test)]
mod coord_tests {
	use super::*;
	use proptest::prelude::*;
	use std::collections::HashSet;

	fn rand_coord_strategy() -> BoxedStrategy<HexCoord> {
		(any::<u8>(), any::<u8>())
			.prop_map(|(q, r)| HexCoord::new_axial(q, r))
			.boxed()
	}

	fn rand_coord_orientation_strategy() -> BoxedStrategy<RelativeHexCoord> {
		(any::<i8>(), any::<i8>())
			.prop_map(|(q, r)| RelativeHexCoord::new_axial(q, r))
			.boxed()
	}

	fn rand_map_context() -> BoxedStrategy<hex::HexMapContext> {
		(any::<u8>(), any::<u8>(), any::<bool>())
			.prop_map(|(w, h, wx)| hex::HexMapContext {
				width: w,
				height: h,
				wrap_x: wx,
			})
			.boxed()
	}

	#[test]
	fn coord_orientation_ring_iterator_small_count() {
		{
			let mut iter = RelativeHexCoordIteratorRing::new(0);
			assert_eq!(iter.next(), Some(RelativeHexCoord::new_axial(0, 0)));
			assert_eq!(iter.next(), None);
		}
		{
			let coords =
				RelativeHexCoordIteratorRing::new(1).collect::<HashSet<RelativeHexCoord>>();
			assert!(coords.contains(&RelativeHexCoord::new_axial(1, 0)));
			assert!(coords.contains(&RelativeHexCoord::new_axial(0, 1)));
			assert!(coords.contains(&RelativeHexCoord::new_axial(-1, 1)));
			assert!(coords.contains(&RelativeHexCoord::new_axial(-1, 0)));
			assert!(coords.contains(&RelativeHexCoord::new_axial(0, -1)));
			assert!(coords.contains(&RelativeHexCoord::new_axial(1, -1)));
			assert_eq!(coords.len(), 6);
		}
	}

	proptest!(
		#![proptest_config(ProptestConfig::with_cases(30))]
		#[test]
		fn coord_orientation_ring_iterator_big_count(distance in 2..128u8) {
			let around = RelativeHexCoordIteratorRing::new(distance)
				.collect::<HashSet<RelativeHexCoord>>();
			assert_eq!(
				around.len(),
				(distance as usize * 6),
				"Distance {} missing/extra values, generated: {:?}",
				distance,
				around
			);
		}
	);

	#[test]
	fn coord_orientation_neighbor_iterator_small_count() {
		let mut iter = RelativeHexCoordIteratorFull::new(0);
		assert_eq!(iter.next(), Some(RelativeHexCoord::new_axial(0, 0)));
		assert_eq!(iter.next(), None);
	}

	proptest!(
		#![proptest_config(ProptestConfig::with_cases(30))]
		#[test]
		fn coord_orientation_neighbor_iterator_big_count(distance in 1..128u8) {
			let around = RelativeHexCoordIteratorFull::new(distance)
				.collect::<HashSet<RelativeHexCoord>>();
			assert_eq!(
				around.len(),
				3 * ((distance as usize).pow(2) + distance as usize) + 1,
				"Distance {} missing/extra values, generated: {:?}",
				distance,
				around
			);
		}
	);

	proptest!(
		#[test]
		fn coord_iterator_should_give_equal_dists(
			coord in rand_coord_strategy(),
			ctx in rand_map_context(),
			distance in 0..128u8
		) {
			for i in coord.iter_neighbors_ring(distance, &ctx) {
				prop_assert_eq!(
					coord.distance_to(&i, &ctx),
					distance,
					"other Coord: {:?}",
					i
				);
			}
		}
	);

	proptest!(
		#[test]
		fn coord_to_linear_from_linear(axial in rand_coord_strategy()) {
			let (x, y) = axial.to_linear();
			let axial_to_from = Coord::from_linear(x, y);

			prop_assert_eq!((x, y, axial), (x, y, axial_to_from));
		}
	);

	proptest!(
		#[test]
		fn sum_xyz(coord in rand_coord_strategy()) {
			prop_assert_eq!(
				coord.x().wrapping_add(
				coord.y().wrapping_add(
				coord.z())),
				0
			);
		}

		#[test]
		fn sum_xyz_orientation(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.x().wrapping_add(
				coord.y().wrapping_add(
				coord.z())),
				0
			);
		}
	);

	proptest!(
		#[test]
		fn wrapping_get_always_returns_when_wrapping(
			coord in rand_coord_strategy(),
			max_x: u8
		) {
			let ctx = hex::HexMapContext{
				width: max_x,
				height: 255,
				wrap_x: true,
			};
			prop_assert_ne!(coord.idx(&ctx), None);
		}
	);

	proptest!(
		#[test]
		fn six_rights_make_itself(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.cw().cw().cw().cw().cw().cw(),
				coord
			);
		}
	);

	proptest!(
		#[test]
		fn six_lefts_make_itself(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.ccw().ccw().ccw().ccw().ccw().ccw(),
				coord
			);
		}
	);

	proptest!(
		#[test]
		fn three_lefts_make_three_rights(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.ccw().ccw().ccw(),
				coord.cw().cw().cw()
			);
		}
	);

	proptest!(
		#[test]
		fn three_rights_negate(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(coord.cw().cw().cw(), -coord);
		}
	);
}
