mod buf;
mod key;
mod node;
mod interpolator;
mod search;

#[cfg(test)]
mod tests;

use {
    std::{
        ops::Index,
        marker::PhantomData
    },
    buf::Buffer,
    search::search,
};

pub use {
    key::{
        TrackKey,
        TrackKeyDistance,
    },
    node::TrackNode,
    interpolator::TrackInterpolator,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Overflow,
    KeyNotInRange,
    KeyIsNotInInnerRange,
}

type Key<I> = <I as TrackInterpolator>::Key;

type KeyDistance<I> = <Key<I> as TrackKey>::Distance;

type Data<I> = <I as TrackInterpolator>::Data;

type NotAlignedData<I> = <I as TrackInterpolator>::NotAlignedData;

type Node<I> = TrackNode<Key<I>, Data<I>, NotAlignedData<I>>;

type NotAlignedNode<I> = node::NotAlignedNode::<Key<I>, Data<I>, NotAlignedData<I>>;

type Output<I> = <I as TrackInterpolator>::Output;

type TrackRange = (usize, usize);

struct NearbyNodes<I: TrackInterpolator> {
    begin_index: usize,
    begin_key: Key<I>,
    end_index: usize,
    end_key: Key<I>
}

pub struct Track<I: TrackInterpolator> {
    interpolator: I,
    ranges: Buffer<TrackRange>,
    buf: Buffer<Node<I>>,
    aligned_step: KeyDistance<I>,
    next_step: KeyDistance<I>,
    key_start: Key<I>,
    key_end: Key<I>,
}

impl<I: TrackInterpolator> Track<I> {
    pub fn new(interpolator: I, track_size: usize, aligned_step: KeyDistance<I>) -> Self {
        assert!(track_size > 1);

        Self {
            interpolator,
            ranges: Buffer::new(track_size - 1),
            buf: Buffer::new(track_size),
            aligned_step: aligned_step.clone(),
            next_step: aligned_step,
            key_start: Key::<I>::default(),
            key_end: Key::<I>::default(),
        }
    }

    pub fn key_start(&self) -> &Key<I> {
        &self.key_start
    }

    pub fn key_end(&self) -> &Key<I> {
        &self.key_end
    }

    pub fn reset_track(&mut self, new_key_start: Key<I>) -> buf::Truncated<Node<I>> {
        self.ranges.clear();
        self.key_start = new_key_start.clone();
        self.next_step = self.aligned_step.clone();
        self.key_end = new_key_start;
        self.buf.clear()
    }

    pub fn interpolate(&mut self, key: &Key<I>) -> Result<Output<I>> {
        if *key < self.key_start || *key >= self.key_end {
            return Err(Error::KeyNotInRange);
        }

        let range_index = self.range_index(key);
        let nodes = self.find_nearby_nodes_in_range(range_index, key);

        let output = self.interpolator.interpolate(
            key, 
            nodes.begin_key, 
            &self.buf[nodes.begin_index], 
            nodes.end_key, 
            &self.buf[nodes.end_index]
        );

        Ok(output)
    }

    pub fn truncate_back(&mut self, key: &Key<I>) {
        // If `key` is behind the `self.key_start` -- distance will be negative.
        // Negative distance might cause too big `range_index`.
        //
        // Also, those key value is meaningless for `truncate_back`
        if !self.is_forward_key(&key) || self.ranges.len() < 2 {
            return;
        }

        let old_begin = self.ranges.first().unwrap().0;
        let range_index = self.range_index(&key);
        let removed_ranges = self.ranges.truncate_back(range_index);
        let removed_ranges = removed_ranges.len();

        self.key_start = self.increase_key_by_step(&self.key_start, removed_ranges);

        let (begin, _) = self.ranges.first().unwrap();
        self.buf.truncate_back(
            self.wrap_buf_index(
                *begin, old_begin
            )
        );
    }

    pub fn cancel_forward(&mut self, key: &Key<I>) -> buf::Truncated<Node<I>> {
        if *key <= self.key_start {
            return self.reset_track(
                Key::<I>::default()
            );
        } else if *key > self.key_end || self.is_empty() {
            return buf::Truncated::empty(&mut self.buf)
        }
        
        let index;
        let mut range_index;

        if *key == self.key_end {
            range_index = self.ranges.len() - 1;
            let (_, end) = self.ranges.last_mut().unwrap();
            *end -= 1;

            index = *end;
        } else {
            range_index = self.range_index(key);
            let nearby_nodes = self.find_nearby_nodes_in_range(range_index, key);
            let mut node_index = nearby_nodes.begin_index;
            let finded_key = nearby_nodes.begin_key;
            if  *key == finded_key {
                node_index -= 1;
            }

            index = node_index;
    
            if *key == self.range_index_to_key(range_index) {
                range_index -= 1;
            }
    
            self.ranges.truncate_forward(range_index);
            self.ranges.last_mut().unwrap().1 = index;
        }

        let (begin, end) = self.ranges.last().unwrap();
        if begin == end {
            if self.ranges.len() > 1 {
                self.ranges.truncate_forward(range_index - 1);
            } else {
                self.ranges.clear();
            }
        }

        match self.buf[index] {
            TrackNode::Aligned(_) => {
                self.next_step = self.aligned_step.clone();
                self.key_end = self.increase_key_by_step(&self.key_start, self.ranges.len());
            },
            TrackNode::NotAligned(ref node) => {
                let range_index = self.range_index(&node.key);

                let nearest_aligned_key = self.increase_key_by_step(
                    &self.key_start, 
                    range_index
                );

                self.key_end = node.key.clone();
                let key_distance = nearest_aligned_key.distance(&self.key_end);
                self.next_step = self.aligned_step.clone() - key_distance;
            }
        };

        self.buf.truncate_forward(index)
    }

    pub fn push_aligned(&mut self, node: Data<I>) -> Result<()> {
        if self.is_empty() {
            self.buf.try_push(node.into()).unwrap();
            return Ok(());
        }

        if self.ranges.is_empty() {
            debug_assert!(matches![self.node_end().unwrap(), Node::<I>::Aligned(_)]);

            self.buf.try_push(node.into()).unwrap();
            self.ranges.try_push((0, 1)).unwrap();
        } else {
            self.push_helper(node.into())?;
        }

        self.key_end = self.key_end.add_distance(&self.next_step);
        self.next_step = self.aligned_step.clone();

        Ok(())
    }

    pub fn insert_not_aligned<Handler>(
        &mut self, 
        key: Key<I>, 
        node: NotAlignedData<I>,
        mut handler: Handler
    ) -> Result<()> 
    where
        Handler: FnMut(&mut Node<I>)
    {
        if !self.is_key_in_inner_range(&key) {
            return Err(Error::KeyIsNotInInnerRange);
        }

        let mut canceled_nodes = self.cancel_forward(&key);
        let nearest_canceled_node = canceled_nodes.nth(0).unwrap().clone();
        for node in canceled_nodes {
            handler(node)
        }

        let nearest_canceled_key = match nearest_canceled_node {
            Node::<I>::Aligned(_) => self.key_end.add_distance(&self.next_step),
            Node::<I>::NotAligned(ref node) => node.key.clone()
        };
        let nearest_canceled_node = (*nearest_canceled_node).clone();

        let node_key = key.clone();
        let not_aligned_node = NotAlignedNode::<I> {
            node,
            key,
            canceled_node: nearest_canceled_node,
            canceled_key: nearest_canceled_key,
            phantom: PhantomData
        };

        if self.ranges.is_empty() {
            debug_assert!(matches![self.node_end().unwrap(), Node::<I>::Aligned(_)]);

            self.buf.try_push(not_aligned_node.into()).unwrap();
            self.ranges.try_push((0, 1)).unwrap();
        } else {
            self.push_helper(not_aligned_node.into())?;
        }

        let key_distance = self.key_end.distance(&node_key);
        self.next_step = self.next_step.clone() - key_distance;
        self.key_end = node_key;

        Ok(())
    }

    fn push_helper(&mut self, node: Node<I>) -> Result<()> {
        match self.node_end().unwrap() {
            Node::<I>::Aligned(_) => {
                self.try_push(node)?;

                let (_, last_end) = self.ranges.last().unwrap();

                let new_begin = *last_end;
                let new_end = new_begin + 1;

                self.ranges.try_push((new_begin, new_end)).unwrap();
            },
            Node::<I>::NotAligned(_) => {
                self.try_push(node)?;

                let (_, last_end) = self.ranges.last_mut().unwrap();
                *last_end += 1;
            }
        }

        Ok(())
    }

    fn try_push(&mut self, node: Node::<I>) -> Result<()> {
        if let Err(buf::Error::Overflow(node)) = self.buf.try_push(node) {
            self.force_push(node)?;
        }

        Ok(())
    }

    fn force_push(&mut self, node: Node::<I>) -> Result<()> {
        debug_assert!(!self.ranges.is_empty());

        if self.ranges.len() == 1 {
            self.buf.grow();
            self.buf.try_push(node).unwrap();
            
            Ok(())
        } else {
            Err(Error::Overflow)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn node_start(&self) -> Option<&Node<I>> {
        self.buf.first()
    }

    pub fn node_end(&self) -> Option<&Node<I>> {
        self.buf.last()
    }

    fn find_nearby_nodes_in_range(&self, range_index: usize, key: &Key<I>) -> NearbyNodes<I> {
        let range = self.ranges[range_index];

        let begin_key = self.range_index_to_key(range_index);
        let end_key = match self.buf[range.1] {
            Node::<I>::Aligned(_) => begin_key.add_distance(&self.aligned_step),
            Node::<I>::NotAligned(ref node) => node.key.clone()
        };

        assert!(begin_key <= *key && *key <= end_key);

        let range_adapter = TrackRangeAdapter::<I>::new(
            &self.buf, 
            range, 
            begin_key, 
            end_key.clone()
        );

        let (begin_index, begin_key) = search(&range_adapter, range.0, range.1, key);
        let end_index = begin_index + 1;
        let begin_key = begin_key.clone();
        let end_key = match self.buf[end_index] {
            Node::<I>::Aligned(_) => end_key,
            Node::<I>::NotAligned(ref node) => node.key.clone()
        };

        NearbyNodes::<I> {
            begin_index,
            begin_key,
            end_index,
            end_key
        }
    }

    fn is_key_in_inner_range(&self, key: &Key<I>) -> bool {
        self.key_start < *key && * key < self.key_end
    }

    fn is_forward_key(&self, key: &Key<I>) -> bool {
        self.key_start <= *key
    }

    fn range_index(&self, key: &Key<I>) -> usize {
        self.key_start.distance(key)
            .div_floor(&self.aligned_step)
    }

    fn increase_key_by_step(&self, key: &Key<I>, steps: usize) -> Key<I> {
        key.add_distance(
            &self.aligned_step.scale(steps)
        )
    }

    fn range_index_to_key(&self, range_index: usize) -> Key<I> {
        assert!(range_index < self.ranges.len());

        self.increase_key_by_step(&self.key_start, range_index)
    }

    fn wrap_buf_index(&self, buf_index: usize, begin_index: usize) -> usize {
        assert!(buf_index >= begin_index);
        let index = buf_index - begin_index;

        self.buf.wrap_raw_index(index)
    }
}

struct TrackRangeAdapter<'b, I: TrackInterpolator> {
    buf: &'b Buffer<Node<I>>,
    range: TrackRange,
    left_key: Key<I>,
    right_key: Key<I>
}

impl<'b, I: TrackInterpolator> TrackRangeAdapter<'b, I> {
    fn new(
        buf: &'b Buffer<Node<I>>, 
        range: TrackRange, 
        left_key: Key<I>,
        right_key: Key<I>
    ) -> Self {
        Self {
            buf,
            range,
            left_key,
            right_key
        }
    }
}

impl<'b, I: TrackInterpolator> Index<usize> for TrackRangeAdapter<'b, I> {
    type Output = Key<I>;

    fn index(&self, index: usize) -> &Self::Output { 
        if index == self.range.0 {
            &self.left_key
        } else if index == self.range.1 {
            &self.right_key
        } else {
            match self.buf[index] {
                TrackNode::NotAligned(ref node) => &node.key,
                TrackNode::Aligned(_) => panic!("unexpected aligned node")
            }
        }
    }
}