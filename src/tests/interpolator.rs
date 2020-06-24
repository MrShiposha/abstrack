use {
    std::ops::Deref,
    crate::*
};

type Key = i64;

type Distance = i64;

type Data = usize;

type Node = node::TrackNode<Key, Data, DataNA>;

pub struct Interpolator;

impl TrackInterpolator for Interpolator {
    type Key = Key;
    type Data = Data;
    type NotAlignedData = DataNA;
    type Output = Output;
    
    fn interpolate(
        &mut self, 
        requested_key: &Self::Key, 
        begin_key: Self::Key,
        begin_node: &Node, 
        end_key: Self::Key,
        end_node: &Node
    ) -> Self::Output { 
        Self::Output {
            requested_key: requested_key.clone(),
            begin_key,
            begin_node: begin_node.clone(),
            end_key,
            end_node: end_node.clone()
        }
    }
}

impl TrackKey for Key {
    type Distance = Distance;
    
    fn distance(&self, rhs: &Key) -> Self::Distance { 
        rhs - self
    }
    
    fn add_distance(&self, rhs: &Self::Distance) -> Self { 
        self + rhs
    }
}

impl TrackKeyDistance for Distance {
    fn abs(&self) -> Self {
        Distance::abs(*self)
    }

    fn scale(&self, factor: usize) -> Self {
        *self * factor as Distance
    }

    fn div_floor(&self, other: &Self) -> usize {
        (*self / *other) as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataNA(pub Data);

impl Deref for DataNA {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Output {
    pub requested_key: Key,
    pub begin_key: Key,
    pub begin_node: Node,
    pub end_key: Key,
    pub end_node: Node,
}