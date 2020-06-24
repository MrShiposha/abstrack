use {
    std::{
        ops::Deref,
        fmt::Debug,
    },
    crate::{
        TrackKey,
        TrackNode
    }
}; 

pub trait TrackInterpolator {
    type Key: TrackKey;
    type Data: Debug + Default + Clone;
    type NotAlignedData: Debug + Deref<Target=Self::Data> + Clone;
    type Output;

    fn interpolate(
        &mut self, 
        key: &Self::Key,
        lhs_key: Self::Key,
        lhs: &TrackNode<Self::Key, Self::Data, Self::NotAlignedData>,
        rhs_key: Self::Key,
        rhs: &TrackNode<Self::Key, Self::Data, Self::NotAlignedData>,
    ) -> Self::Output;
}