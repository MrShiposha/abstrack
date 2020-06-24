use std::{
    ops::{Add, Sub, Neg},
    fmt::Debug,
};

pub trait TrackKey<RHS=Self>: Debug + Ord + Default + Clone {
    type Distance: TrackKeyDistance;

    fn distance(&self, rhs: &RHS) -> Self::Distance;

    fn add_distance(&self, distance: &Self::Distance) -> Self;
}

pub trait TrackKeyDistance: Debug 
    + Add<Output=Self> 
    + Sub<Output=Self>
    + Neg<Output=Self> 
    + Ord
    + Default 
    + Clone 
{
    fn abs(&self) -> Self;

    fn scale(&self, factor: usize) -> Self;

    fn div_floor(&self, other: &Self) -> usize;
}