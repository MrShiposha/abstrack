mod interpolator;

use crate::{
    *, 
    tests::interpolator::*
};

macro_rules! test_not_aligned {
    (
        $buf_node:expr, 
        key: $key:expr, 
        node: $node:expr, 
        canceled_key: $ckey:expr, 
        canceled_node: $cnode: expr
    ) => {
        match $buf_node {
            TrackNode::NotAligned(ref node) => {
                assert_eq!(node.key, $key);
                assert_eq!(***node, $node);
                assert_eq!(node.canceled_key, $ckey);
                assert_eq!(node.canceled_node, $cnode);
            },
            _ => panic!("expected not aligned node")
        }
    };
}

#[test]
fn test_track_new() {
    let track_size = 5;
    let step = 1;

    let track = Track::new(Interpolator, track_size, step);

    assert!(track.is_empty());
    assert!(track.buf.is_empty());
    assert_eq!(track.buf.capacity(), track_size);
    assert!(track.ranges.is_empty());
    assert_eq!(track.aligned_step, step);
    assert_eq!(track.key_start, Default::default());
    assert_eq!(track.key_end, Default::default());
}

#[test]
#[should_panic]
fn test_invalid_track_new() {
    Track::new(Interpolator, 1, 1);
}

#[test]
fn test_reset_track() {
    let track_size = 5;
    let step = 1;
    let key_start = 40;

    let mut track = Track::new(Interpolator, track_size, step);
    track.reset_track(key_start);

    assert!(track.is_empty());
    assert!(track.buf.is_empty());
    assert_eq!(track.buf.capacity(), track_size);
    assert!(track.ranges.is_empty());
    assert_eq!(track.ranges.capacity(), track_size - 1);
    assert_eq!(track.aligned_step, step);
    assert_eq!(track.key_start, key_start);
    assert_eq!(track.key_end, key_start);
}

#[test]
fn test_push() -> Result<()> {
    test_aligned_full_push()?;
    test_one_range_push()?;
    test_two_ranges_push()?;
    test_grow_push()?;

    Ok(())
}

#[test]
fn test_insert_not_aligned() -> Result<()> {
    let track_size = 8;
    let track_step = 10;
    let mut track = Track::new(Interpolator, track_size, track_step);
    test_insert_not_inner_range(&mut track);

    let result = track.insert_not_aligned(1, DataNA(1), |_| {});

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::KeyIsNotInInnerRange);

    track.push_aligned(0)?;
    assert_eq!(track.next_step, 10);

    test_insert_not_inner_range(&mut track);
    let result = track.insert_not_aligned(1, DataNA(1), |_| {});

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::KeyIsNotInInnerRange);

    track.push_aligned(10)?;
    assert_eq!(track.next_step, 10);
    assert_eq!(track.key_end, 10);
    test_insert_not_inner_range(&mut track);
    let mut canceled = vec![];

    track.insert_not_aligned(5, DataNA(5), |node| canceled.push(**node))?;
    test_not_aligned_node(track.buf.last().unwrap(), 5, DataNA(5), 10, 10);
    assert_eq!(canceled, vec![10]);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.next_step, 5);
    assert_eq!(track.key_end, 5);

    test_insert_not_inner_range(&mut track);
    let mut canceled = vec![];
    track.insert_not_aligned(2, DataNA(2), |node| canceled.push(node.clone()))?;
    test_not_aligned_node(track.buf.last().unwrap(), 2, DataNA(2), 10, 10);
    assert_eq!(canceled.len(), 1);
    test_not_aligned_node(&canceled[0], 5, DataNA(5), 10, 10);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.next_step, 8);
    assert_eq!(track.key_end, 2);

    track.push_aligned(10)?;
    assert_eq!(track.next_step, 10);
    test_insert_not_inner_range(&mut track);
    let mut canceled = vec![];
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.key_end, 10);

    track.insert_not_aligned(5, DataNA(5), |node| canceled.push(**node))?;
    test_not_aligned_node(track.buf.last().unwrap(), 5, DataNA(5), 10, 10);
    assert_eq!(canceled, vec![10]);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.next_step, 5);
    assert_eq!(track.key_end, 5);

    track.push_aligned(10)?;
    assert_eq!(track.next_step, 10);
    assert_eq!(track.key_end, 10);
    test_insert_not_inner_range(&mut track);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 3));

    track.push_aligned(20)?;
    assert_eq!(track.next_step, 10);
    assert_eq!(track.key_end, 20);
    test_insert_not_inner_range(&mut track);
    let mut canceled = vec![];
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.ranges[1], (3, 4));

    track.insert_not_aligned(15, DataNA(15), |node| canceled.push(**node))?;
    test_not_aligned_node(track.buf.last().unwrap(), 15, DataNA(15), 20, 20);
    assert_eq!(canceled, vec![20]);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.ranges[1], (3, 4));
    assert_eq!(track.next_step, 5);
    assert_eq!(track.key_end, 15);

    track.push_aligned(20)?;
    assert_eq!(track.next_step, 10);
    assert_eq!(track.key_end, 20);
    test_insert_not_inner_range(&mut track);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.ranges[1], (3, 5));

    track.push_aligned(30)?;
    assert_eq!(track.next_step, 10);
    assert_eq!(track.key_end, 30);
    test_insert_not_inner_range(&mut track);
    let mut canceled = vec![];
    assert_eq!(track.ranges.len(), 3);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.ranges[1], (3, 5));
    assert_eq!(track.ranges[2], (5, 6));

    track.insert_not_aligned(20, DataNA(20), |node| canceled.push(**node))?;
    test_not_aligned_node(track.buf.last().unwrap(), 20, DataNA(20), 20, 20);
    assert_eq!(canceled, vec![20, 30]);
    assert_eq!(track.key_end, 20);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.ranges[1], (3, 5));
    assert_eq!(track.next_step, 0);
    assert_eq!(track.key_end, 20);

    track.push_aligned(20)?;
    assert_eq!(track.next_step, 10);
    assert_eq!(track.key_end, 20);
    test_insert_not_inner_range(&mut track);
    assert_eq!(track.key_end, 20);
    let mut canceled = vec![];
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.ranges[1], (3, 6));

    track.insert_not_aligned(3, DataNA(3), |node| canceled.push(node.clone()))?;
    assert_eq!(canceled.len(), 5);
    test_not_aligned_node(&canceled[0], 5, DataNA(5), 10, 10);
    assert_eq!(*canceled[1], 10);
    test_not_aligned_node(&canceled[2], 15, DataNA(15), 20, 20);
    test_not_aligned_node(&canceled[3], 20, DataNA(20), 20, 20);
    assert_eq!(*canceled[4], 20);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.next_step, 7);
    assert_eq!(track.key_end, 3);

    Ok(())
}

#[test]
fn test_truncate_back() -> Result<()> {
    test_truncate_back_one_range()?;
    test_truncate_back_two_ranges()?;
    test_truncate_back_two_ranges_not_aligned()?;
    test_truncate_back_seq()?;

    Ok(())
}

#[test]
fn test_cancel_forward() -> Result<()> {
    test_cancel_forward_one_range()?;
    test_cancel_two_ranges()?;
    test_cancel_aligned()?;

    Ok(())
}

#[test]
fn test_find_nearby_nodes_in_range() -> Result<()> {
    let track_size = 5;
    let track_step = 30;
    let mut track = Track::new(Interpolator, track_size, track_step);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(1)?;
    track.push_aligned(31)?;
    track.insert_not_aligned(8, DataNA(8), |_| {})?;
    track.push_aligned(31)?;
    track.insert_not_aligned(16, DataNA(16), |_| {})?;
    track.push_aligned(31)?;
    track.insert_not_aligned(24, DataNA(24), |_| {})?;
    track.push_aligned(31)?;
    track.push_aligned(61)?;
    track.insert_not_aligned(41, DataNA(41), |_| {})?;
    track.push_aligned(61)?;
    track.insert_not_aligned(52, DataNA(52), |_| {})?;

    let nodes = track.find_nearby_nodes_in_range(0, &1);
    assert_eq!(nodes.begin_index, 0);
    assert_eq!(nodes.begin_key, 1);
    assert_eq!(nodes.end_index, 1);
    assert_eq!(nodes.end_key, 8);

    let nodes = track.find_nearby_nodes_in_range(0, &5);
    assert_eq!(nodes.begin_index, 0);
    assert_eq!(nodes.begin_key, 1);
    assert_eq!(nodes.end_index, 1);
    assert_eq!(nodes.end_key, 8);

    let nodes = track.find_nearby_nodes_in_range(0, &8);
    assert_eq!(nodes.begin_index, 1);
    assert_eq!(nodes.begin_key, 8);
    assert_eq!(nodes.end_index, 2);
    assert_eq!(nodes.end_key, 16);

    let nodes = track.find_nearby_nodes_in_range(0, &14);
    assert_eq!(nodes.begin_index, 1);
    assert_eq!(nodes.begin_key, 8);
    assert_eq!(nodes.end_index, 2);
    assert_eq!(nodes.end_key, 16);

    let nodes = track.find_nearby_nodes_in_range(0, &16);
    assert_eq!(nodes.begin_index, 2);
    assert_eq!(nodes.begin_key, 16);
    assert_eq!(nodes.end_index, 3);
    assert_eq!(nodes.end_key, 24);

    let nodes = track.find_nearby_nodes_in_range(0, &20);
    assert_eq!(nodes.begin_index, 2);
    assert_eq!(nodes.begin_key, 16);
    assert_eq!(nodes.end_index, 3);
    assert_eq!(nodes.end_key, 24);

    let nodes = track.find_nearby_nodes_in_range(0, &24);
    assert_eq!(nodes.begin_index, 3);
    assert_eq!(nodes.begin_key, 24);
    assert_eq!(nodes.end_index, 4);
    assert_eq!(nodes.end_key, 31);

    let nodes = track.find_nearby_nodes_in_range(0, &31);
    assert_eq!(nodes.begin_index, 3);
    assert_eq!(nodes.begin_key, 24);
    assert_eq!(nodes.end_index, 4);
    assert_eq!(nodes.end_key, 31);

    let nodes = track.find_nearby_nodes_in_range(1, &31);
    assert_eq!(nodes.begin_index, 4);
    assert_eq!(nodes.begin_key, 31);
    assert_eq!(nodes.end_index, 5);
    assert_eq!(nodes.end_key, 41);

    let nodes = track.find_nearby_nodes_in_range(1, &38);
    assert_eq!(nodes.begin_index, 4);
    assert_eq!(nodes.begin_key, 31);
    assert_eq!(nodes.end_index, 5);
    assert_eq!(nodes.end_key, 41);

    let nodes = track.find_nearby_nodes_in_range(1, &41);
    assert_eq!(nodes.begin_index, 5);
    assert_eq!(nodes.begin_key, 41);
    assert_eq!(nodes.end_index, 6);
    assert_eq!(nodes.end_key, 52);

    let nodes = track.find_nearby_nodes_in_range(1, &48);
    assert_eq!(nodes.begin_index, 5);
    assert_eq!(nodes.begin_key, 41);
    assert_eq!(nodes.end_index, 6);
    assert_eq!(nodes.end_key, 52);

    let nodes = track.find_nearby_nodes_in_range(1, &52);
    assert_eq!(nodes.begin_index, 5);
    assert_eq!(nodes.begin_key, 41);
    assert_eq!(nodes.end_index, 6);
    assert_eq!(nodes.end_key, 52);

    Ok(())
}

#[test]
fn test_interpolate() -> Result<()> {
    let track_size = 8;
    let track_step = 10;
    let mut track = Track::new(Interpolator, track_size, track_step);

    track.push_aligned(0)?;
    track.push_aligned(10)?;
    track.insert_not_aligned(3, DataNA(3), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(5, DataNA(5), |_| {})?;
    track.push_aligned(10)?;
    track.push_aligned(20)?;
    track.insert_not_aligned(12, DataNA(12), |_| {})?;
    track.push_aligned(20)?;
    track.push_aligned(30)?;
    track.push_aligned(40)?;
    track.insert_not_aligned(34, DataNA(34), |_| {})?;

    let out = track.interpolate(&-1);
    assert!(out.is_err());
    assert_eq!(out.unwrap_err(), Error::KeyNotInRange);

    let out = track.interpolate(&0)?;
    assert_eq!(out.requested_key, 0);
    assert_eq!(out.begin_key, 0);
    assert_eq!(*out.begin_node, 0);
    assert_eq!(out.end_key, 3);
    assert_eq!(*out.end_node, 3);

    let out = track.interpolate(&1)?;
    assert_eq!(out.requested_key, 1);
    assert_eq!(out.begin_key, 0);
    assert_eq!(*out.begin_node, 0);
    assert_eq!(out.end_key, 3);
    assert_eq!(*out.end_node, 3);

    let out = track.interpolate(&2)?;
    assert_eq!(out.requested_key, 2);
    assert_eq!(out.begin_key, 0);
    assert_eq!(*out.begin_node, 0);
    assert_eq!(out.end_key, 3);
    assert_eq!(*out.end_node, 3);

    let out = track.interpolate(&3)?;
    assert_eq!(out.requested_key, 3);
    assert_eq!(out.begin_key, 3);
    assert_eq!(*out.begin_node, 3);
    assert_eq!(out.end_key, 5);
    assert_eq!(*out.end_node, 5);

    let out = track.interpolate(&4)?;
    assert_eq!(out.requested_key, 4);
    assert_eq!(out.begin_key, 3);
    assert_eq!(*out.begin_node, 3);
    assert_eq!(out.end_key, 5);
    assert_eq!(*out.end_node, 5);

    let out = track.interpolate(&5)?;
    assert_eq!(out.requested_key, 5);
    assert_eq!(out.begin_key, 5);
    assert_eq!(*out.begin_node, 5);
    assert_eq!(out.end_key, 10);
    assert_eq!(*out.end_node, 10);

    let out = track.interpolate(&7)?;
    assert_eq!(out.requested_key, 7);
    assert_eq!(out.begin_key, 5);
    assert_eq!(*out.begin_node, 5);
    assert_eq!(out.end_key, 10);
    assert_eq!(*out.end_node, 10);

    let out = track.interpolate(&10)?;
    assert_eq!(out.requested_key, 10);
    assert_eq!(out.begin_key, 10);
    assert_eq!(*out.begin_node, 10);
    assert_eq!(out.end_key, 12);
    assert_eq!(*out.end_node, 12);

    let out = track.interpolate(&11)?;
    assert_eq!(out.requested_key, 11);
    assert_eq!(out.begin_key, 10);
    assert_eq!(*out.begin_node, 10);
    assert_eq!(out.end_key, 12);
    assert_eq!(*out.end_node, 12);

    let out = track.interpolate(&12)?;
    assert_eq!(out.requested_key, 12);
    assert_eq!(out.begin_key, 12);
    assert_eq!(*out.begin_node, 12);
    assert_eq!(out.end_key, 20);
    assert_eq!(*out.end_node, 20);

    let out = track.interpolate(&15)?;
    assert_eq!(out.requested_key, 15);
    assert_eq!(out.begin_key, 12);
    assert_eq!(*out.begin_node, 12);
    assert_eq!(out.end_key, 20);
    assert_eq!(*out.end_node, 20);

    let out = track.interpolate(&20)?;
    assert_eq!(out.requested_key, 20);
    assert_eq!(out.begin_key, 20);
    assert_eq!(*out.begin_node, 20);
    assert_eq!(out.end_key, 30);
    assert_eq!(*out.end_node, 30);

    let out = track.interpolate(&25)?;
    assert_eq!(out.requested_key, 25);
    assert_eq!(out.begin_key, 20);
    assert_eq!(*out.begin_node, 20);
    assert_eq!(out.end_key, 30);
    assert_eq!(*out.end_node, 30);

    let out = track.interpolate(&30)?;
    assert_eq!(out.requested_key, 30);
    assert_eq!(out.begin_key, 30);
    assert_eq!(*out.begin_node, 30);
    assert_eq!(out.end_key, 34);
    assert_eq!(*out.end_node, 34);

    let out = track.interpolate(&32)?;
    assert_eq!(out.requested_key, 32);
    assert_eq!(out.begin_key, 30);
    assert_eq!(*out.begin_node, 30);
    assert_eq!(out.end_key, 34);
    assert_eq!(*out.end_node, 34);

    let out = track.interpolate(&33)?;
    assert_eq!(out.requested_key, 33);
    assert_eq!(out.begin_key, 30);
    assert_eq!(*out.begin_node, 30);
    assert_eq!(out.end_key, 34);
    assert_eq!(*out.end_node, 34);

    let out = track.interpolate(&34);
    assert!(out.is_err());
    assert_eq!(out.unwrap_err(), Error::KeyNotInRange);

    Ok(())
}

#[test]
fn test_range_index() {
    let track_size = 5;
    let step = 2;

    let track = Track::new(Interpolator, track_size, step);

    assert_eq!(track.range_index(&0), 0);
    assert_eq!(track.range_index(&1), 0);
    assert_eq!(track.range_index(&2), 1);
    assert_eq!(track.range_index(&3), 1);
    assert_eq!(track.range_index(&4), 2);
    assert_eq!(track.range_index(&5), 2);
    assert_eq!(track.range_index(&6), 3);
    assert_eq!(track.range_index(&7), 3);
    assert_eq!(track.range_index(&8), 4);
    assert_eq!(track.range_index(&9), 4);
}

fn test_insert_not_inner_range(track: &mut Track<Interpolator>) {
    let key_start = track.key_start().clone();
    let key_end = track.key_end().clone();
    let test_node = DataNA(42);

    let result = track.insert_not_aligned(key_start, test_node.clone(), |_| {});
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::KeyIsNotInInnerRange);

    let result = track.insert_not_aligned(key_start.add_distance(&-1), test_node.clone(), |_| {});
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::KeyIsNotInInnerRange);

    let result = track.insert_not_aligned(key_end, test_node.clone(), |_| {});
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::KeyIsNotInInnerRange);

    let result = track.insert_not_aligned(key_end.add_distance(&1), test_node.clone(), |_| {});
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::KeyIsNotInInnerRange);
}

fn test_not_aligned_node(
    node: &Node<Interpolator>,
    key: Key<Interpolator>, 
    data: DataNA, 
    canceled_key: Key<Interpolator>,
    canceled_data: Data<Interpolator>
) {
    let node = match node {
        TrackNode::NotAligned(node) => node,
        _ => unreachable!()
    };

    assert_eq!(node.key, key);
    assert_eq!(**node, data);
    assert_eq!(node.canceled_key, canceled_key);
    assert_eq!(node.canceled_node, canceled_data);
}

fn test_aligned_full_push() -> Result<()> {
    let track_size = 5;
    let step = 2;

    let mut track = Track::new(Interpolator, track_size, step);

    track.push_aligned(0)?;
    assert!(track.ranges.is_empty());

    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default()
    );

    track.push_aligned(1)?;
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() 
        + step as <Interpolator as TrackInterpolator>::Key
    );

    track.push_aligned(2)?;
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.ranges[1], (1, 2));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() 
        + step * 2 as <Interpolator as TrackInterpolator>::Key
    );

    track.push_aligned(3)?;
    assert_eq!(track.ranges.len(), 3);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.ranges[1], (1, 2));
    assert_eq!(track.ranges[2], (2, 3));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() 
        + step * 3 as <Interpolator as TrackInterpolator>::Key
    );

    track.push_aligned(4)?;
    assert_eq!(track.ranges.len(), 4);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.ranges[1], (1, 2));
    assert_eq!(track.ranges[2], (2, 3));
    assert_eq!(track.ranges[3], (3, 4));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() 
        + step * 4 as <Interpolator as TrackInterpolator>::Key
    );

    assert!(track.push_aligned(track_size).is_err());

    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() 
        + step * (track_size - 1) as <Interpolator as TrackInterpolator>::Key
    );

    Ok(())
}

fn test_one_range_push() -> Result<()> {
    let track_size = 5;
    let step = 4;

    let mut track = Track::new(Interpolator, track_size, step);
    track.push_aligned(0)?;
    assert!(track.ranges.is_empty());
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default()
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(1, DataNA(10), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));

    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 1
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(2, DataNA(20), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 2));

    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 2
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(3, DataNA(30), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 3));

    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 3
    );

    track.push_aligned(4)?;
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 4));

    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + step
    );

    assert_eq!(*track.buf[0], 0);
    test_not_aligned! {
        track.buf[1],
        key: 1,
        node: 10,
        canceled_key: 4,
        canceled_node: 4
    };
    test_not_aligned! {
        track.buf[2],
        key: 2,
        node: 20,
        canceled_key: 4,
        canceled_node: 4
    };
    test_not_aligned! {
        track.buf[3],
        key: 3,
        node: 30,
        canceled_key: 4,
        canceled_node: 4
    };
    assert_eq!(*track.buf[4], 4);

    Ok(())
}

fn test_two_ranges_push() -> Result<()> {
    let track_size = 6;
    let step = 4;

    let mut track = Track::new(Interpolator, track_size, step);
    
    track.push_aligned(0)?;
    assert!(track.ranges.is_empty());
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default()
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(1, DataNA(10), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 1
    );

    track.push_aligned(4)?;
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + step
    );

    track.push_aligned(8)?;
    track.insert_not_aligned(step + 2, DataNA(20), |_| {})?;

    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.ranges[1], (2, 3));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + step + 2
    );

    track.push_aligned(8)?;
    track.insert_not_aligned(step + 3, DataNA(30), |_| {})?;

    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.ranges[1], (2, 4));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + step + 3
    );

    track.push_aligned(8)?;
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.ranges[1], (2, 5));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + step + step
    );

    Ok(())
}

fn test_grow_push() -> Result<()> {
    let track_size = 4;
    let step = 4;

    let mut track = Track::new(Interpolator, track_size, step);
    track.push_aligned(0)?;
    assert!(track.ranges.is_empty());
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default()
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(1, DataNA(10), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 1
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(2, DataNA(20), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 2));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 2
    );

    track.push_aligned(4)?;
    track.insert_not_aligned(3, DataNA(30), |_| {})?;

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + 3
    );

    let old_capacity = track.buf.capacity();
    track.push_aligned(4)?;

    assert!(old_capacity < track.buf.capacity());

    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 4));
    assert_eq!(track.key_start, Default::default());
    assert_eq!(
        track.key_end, 
        <Interpolator as TrackInterpolator>::Key::default() + step
    );

    assert_eq!(*track.buf[0], 0);
    test_not_aligned! {
        track.buf[1],
        key: 1,
        node: 10,
        canceled_key: 4,
        canceled_node: 4
    };
    test_not_aligned! {
        track.buf[2],
        key: 2,
        node: 20,
        canceled_key: 4,
        canceled_node: 4
    };
    test_not_aligned! {
        track.buf[3],
        key: 3,
        node: 30,
        canceled_key: 4,
        canceled_node: 4
    };
    assert_eq!(*track.buf[4], 4);

    Ok(())
}

fn test_truncate_back_one_range() -> Result<()> {
    let mut track = Track::new(Interpolator, 2, 1);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(0)?;
    track.push_aligned(1)?;

    track.truncate_back(&0);
    assert_eq!(track.key_start, key_start);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 0);
    assert_eq!(*track.buf[1], 1);

    track.truncate_back(&1);
    assert_eq!(track.key_start, key_start);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 0);
    assert_eq!(*track.buf[1], 1);

    track.truncate_back(&2);
    assert_eq!(track.key_start, key_start);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 0);
    assert_eq!(*track.buf[1], 1);

    Ok(())
}

fn test_truncate_back_two_ranges() -> Result<()> {
    let mut track = Track::new(Interpolator, 3, 1);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(0)?;
    track.push_aligned(1)?;
    track.push_aligned(2)?;

    track.truncate_back(&0);
    assert_eq!(track.key_start, key_start);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.ranges[1], (1, 2));
    assert_eq!(track.buf.len(), 3);
    assert_eq!(*track.buf[0], 0);
    assert_eq!(*track.buf[1], 1);
    assert_eq!(*track.buf[2], 2);

    track.truncate_back(&1);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.ranges[1], (1, 2));
    assert_eq!(track.buf.len(), 3);
    assert_eq!(*track.buf[0], 0);
    assert_eq!(*track.buf[1], 1);
    assert_eq!(*track.buf[2], 2);

    track.truncate_back(&2);
    assert_eq!(track.key_start, 2);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (1, 2));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 2);

    track.truncate_back(&20);
    assert_eq!(track.key_start, 2);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (1, 2));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 2);

    Ok(())
}

fn test_truncate_back_two_ranges_not_aligned() -> Result<()> {
    let key_step = 100;
    let key_start = 100;
    let mut track = Track::new(Interpolator, 3, key_step);

    track.reset_track(key_start);
    track.push_aligned(100)?;
    track.push_aligned(200)?;
    track.insert_not_aligned(125, DataNA(125), |_| {})?;
    track.push_aligned(200)?;
    track.insert_not_aligned(150, DataNA(150), |_| {})?;
    track.push_aligned(200)?;
    track.insert_not_aligned(175, DataNA(175), |_| {})?;
    track.push_aligned(200)?;
    track.push_aligned(300)?;

    track.truncate_back(&0);
    assert_eq!(track.key_start, 100);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 4));
    assert_eq!(track.ranges[1], (4, 5));
    assert_eq!(track.buf.len(), 6);
    assert_eq!(*track.buf[0], 100);
    assert_eq!(*track.buf[1], 125);
    assert_eq!(*track.buf[2], 150);
    assert_eq!(*track.buf[3], 175);
    assert_eq!(*track.buf[4], 200);
    assert_eq!(*track.buf[5], 300);

    track.truncate_back(&100);
    assert_eq!(track.key_start, 100);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (0, 4));
    assert_eq!(track.ranges[1], (4, 5));
    assert_eq!(track.buf.len(), 6);
    assert_eq!(*track.buf[0], 100);
    assert_eq!(*track.buf[1], 125);
    assert_eq!(*track.buf[2], 150);
    assert_eq!(*track.buf[3], 175);
    assert_eq!(*track.buf[4], 200);
    assert_eq!(*track.buf[5], 300);

    track.truncate_back(&200);
    assert_eq!(track.key_start, 200);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (4, 5));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 200);
    assert_eq!(*track.buf[1], 300);

    track.truncate_back(&2000);
    assert_eq!(track.key_start, 200);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (4, 5));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 200);
    assert_eq!(*track.buf[1], 300);

    Ok(())
}

fn test_truncate_back_seq() -> Result<()> {
    let key_step = 1;
    let key_start = 1;
    let mut track = Track::new(Interpolator, 6, key_step);

    track.reset_track(key_start);
    track.push_aligned(0)?;
    track.push_aligned(1)?;
    track.push_aligned(2)?;
    track.push_aligned(3)?;
    track.push_aligned(4)?;
    track.push_aligned(5)?;

    track.truncate_back(&3);
    assert_eq!(track.key_start, 3);
    assert_eq!(track.ranges.len(), 3);
    assert_eq!(track.ranges[0], (2, 3));
    assert_eq!(track.ranges[1], (3, 4));
    assert_eq!(track.ranges[2], (4, 5));
    assert_eq!(track.buf.len(), 4);
    assert_eq!(*track.buf[0], 2);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(*track.buf[2], 4);
    assert_eq!(*track.buf[3], 5);
    
    track.push_aligned(6)?;
    track.push_aligned(7)?;
    assert_eq!(track.ranges.len(), 5);
    assert_eq!(track.ranges[0], (2, 3));
    assert_eq!(track.ranges[1], (3, 4));
    assert_eq!(track.ranges[2], (4, 5));
    assert_eq!(track.ranges[3], (5, 6));
    assert_eq!(track.ranges[4], (6, 7));
    assert_eq!(track.buf.len(), 6);
    assert_eq!(*track.buf[0], 2);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(*track.buf[2], 4);
    assert_eq!(*track.buf[3], 5);
    assert_eq!(*track.buf[4], 6);
    assert_eq!(*track.buf[5], 7);

    track.truncate_back(&5);
    assert_eq!(track.key_start, 5);
    assert_eq!(track.ranges.len(), 3);
    assert_eq!(track.ranges[0], (4, 5));
    assert_eq!(track.ranges[1], (5, 6));
    assert_eq!(track.ranges[2], (6, 7));
    assert_eq!(track.buf.len(), 4);
    assert_eq!(*track.buf[0], 4);
    assert_eq!(*track.buf[1], 5);
    assert_eq!(*track.buf[2], 6);
    assert_eq!(*track.buf[3], 7);

    track.push_aligned(8)?;
    track.push_aligned(9)?;
    assert_eq!(track.ranges.len(), 5);
    assert_eq!(track.ranges[0], (4, 5));
    assert_eq!(track.ranges[1], (5, 6));
    assert_eq!(track.ranges[2], (6, 7));
    assert_eq!(track.ranges[3], (7, 8));
    assert_eq!(track.ranges[4], (8, 9));
    assert_eq!(track.buf.len(), 6);
    assert_eq!(*track.buf[0], 4);
    assert_eq!(*track.buf[1], 5);
    assert_eq!(*track.buf[2], 6);
    assert_eq!(*track.buf[3], 7);
    assert_eq!(*track.buf[4], 8);
    assert_eq!(*track.buf[5], 9);

    track.truncate_back(&8);
    assert_eq!(track.key_start, 8);
    assert_eq!(track.ranges.len(), 2);
    assert_eq!(track.ranges[0], (7, 8));
    assert_eq!(track.ranges[1], (8, 9));
    assert_eq!(track.buf.len(), 3);
    assert_eq!(*track.buf[0], 7);
    assert_eq!(*track.buf[1], 8);
    assert_eq!(*track.buf[2], 9);

    Ok(())
}

fn test_cancel_forward_one_range() -> Result<()> {
    let track_size = 5;
    let track_step = 10;
    let mut track = Track::new(Interpolator, track_size, track_step);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(1)?;
    track.push_aligned(10)?;
    track.insert_not_aligned(3, DataNA(3), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(4, DataNA(4), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(5, DataNA(5), |_| {})?;
    track.push_aligned(10)?;

    let key_end = *track.key_end();

    let canceled = track.cancel_forward(&(key_end + 1));
    assert!(canceled.is_empty());

    let mut canceled = track.cancel_forward(&key_end);
    assert_eq!(canceled.len(), 1);
    assert_eq!(**canceled.nth(0).unwrap(), 10);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.buf.len(), 4);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(*track.buf[2], 4);
    assert_eq!(*track.buf[3], 5);
    assert_eq!(track.key_end, 5);

    let mut canceled = track.cancel_forward(&4);
    assert_eq!(canceled.len(), 2);
    assert_eq!(**canceled.nth(0).unwrap(), 4);
    assert_eq!(**canceled.nth(1).unwrap(), 5);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(track.key_end, 3);

    let mut canceled = track.cancel_forward(&3);
    assert_eq!(canceled.len(), 1);
    assert_eq!(**canceled.nth(0).unwrap(), 3);
    assert!(track.ranges.is_empty());
    assert_eq!(track.buf.len(), 1);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(track.key_end, 1);

    let mut canceled = track.cancel_forward(&1);
    assert_eq!(canceled.len(), 1);
    assert_eq!(**canceled.nth(0).unwrap(), 1);
    assert!(track.ranges.is_empty());
    assert!(track.buf.is_empty());
    assert_eq!(track.key_end, Default::default());

    Ok(())
}

fn test_cancel_two_ranges() -> Result<()> {
    let track_size = 8;
    let track_step = 10;
    let mut track = Track::new(Interpolator, track_size, track_step);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(1)?;
    track.push_aligned(10)?;
    track.insert_not_aligned(3, DataNA(3), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(4, DataNA(4), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(5, DataNA(5), |_| {})?;
    track.push_aligned(10)?;
    track.push_aligned(20)?;
    track.insert_not_aligned(14, DataNA(14), |_| {})?;
    track.push_aligned(20)?;
    track.insert_not_aligned(15, DataNA(15), |_| {})?;
    track.push_aligned(20)?;

    let mut canceled = track.cancel_forward(&4);
    assert_eq!(canceled.len(), 6);
    assert_eq!(**canceled.nth(0).unwrap(), 4);
    assert_eq!(**canceled.nth(1).unwrap(), 5);
    assert_eq!(**canceled.nth(2).unwrap(), 10);
    assert_eq!(**canceled.nth(3).unwrap(), 14);
    assert_eq!(**canceled.nth(4).unwrap(), 15);
    assert_eq!(**canceled.nth(5).unwrap(), 20);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 1));
    assert_eq!(track.buf.len(), 2);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(track.key_end, 3);


    let track_size = 8;
    let track_step = 10;
    let mut track = Track::new(Interpolator, track_size, track_step);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(1)?;
    track.push_aligned(10)?;
    track.insert_not_aligned(3, DataNA(3), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(4, DataNA(4), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(5, DataNA(5), |_| {})?;
    track.push_aligned(10)?;
    track.push_aligned(20)?;
    track.insert_not_aligned(14, DataNA(14), |_| {})?;
    track.push_aligned(20)?;
    track.insert_not_aligned(15, DataNA(15), |_| {})?;
    track.push_aligned(20)?;

    let mut canceled = track.cancel_forward(&11);
    assert_eq!(canceled.len(), 4);
    assert_eq!(**canceled.nth(0).unwrap(), 10);
    assert_eq!(**canceled.nth(1).unwrap(), 14);
    assert_eq!(**canceled.nth(2).unwrap(), 15);
    assert_eq!(**canceled.nth(3).unwrap(), 20);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.buf.len(), 4);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(*track.buf[2], 4);
    assert_eq!(*track.buf[3], 5);
    assert_eq!(track.key_end, 5);

    Ok(())
}

fn test_cancel_aligned() -> Result<()> {
    let track_size = 8;
    let track_step = 10;
    let mut track = Track::new(Interpolator, track_size, track_step);
    let key_start = 1;

    track.reset_track(key_start);
    track.push_aligned(1)?;
    track.push_aligned(10)?;
    track.insert_not_aligned(3, DataNA(3), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(4, DataNA(4), |_| {})?;
    track.push_aligned(10)?;
    track.insert_not_aligned(5, DataNA(5), |_| {})?;
    track.push_aligned(10)?;
    track.push_aligned(20)?;
    track.insert_not_aligned(14, DataNA(14), |_| {})?;
    track.push_aligned(20)?;
    track.insert_not_aligned(15, DataNA(15), |_| {})?;
    track.push_aligned(20)?;

    let mut canceled = track.cancel_forward(&10);
    assert_eq!(canceled.len(), 4);
    assert_eq!(**canceled.nth(0).unwrap(), 10);
    assert_eq!(**canceled.nth(1).unwrap(), 14);
    assert_eq!(**canceled.nth(2).unwrap(), 15);
    assert_eq!(**canceled.nth(3).unwrap(), 20);
    assert_eq!(track.ranges.len(), 1);
    assert_eq!(track.ranges[0], (0, 3));
    assert_eq!(track.buf.len(), 4);
    assert_eq!(*track.buf[0], 1);
    assert_eq!(*track.buf[1], 3);
    assert_eq!(*track.buf[2], 4);
    assert_eq!(*track.buf[3], 5);
    assert_eq!(track.key_end, 5);

    Ok(())
}

// fn test_truncate_forward_one_range() -> Result<()> {
//     let track_size = 2usize;
//     let track_step = 1i64;
//     let mut track = Track::new(Interpolator, track_size, track_step);

//     track.reset_track(1);
//     track.push_aligned(0)?;
//     track.push_aligned(1)?;

//     let key_end = *track.key_end();

//     track.truncate_forward(2);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.buf.len(), 2);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);

//     track.truncate_forward(1);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.buf.len(), 2);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);

//     track.truncate_forward(0);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.buf.len(), 2);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);

//     Ok(())
// }

// fn test_truncate_forward_two_ranges() -> Result<()> {
//     let track_size = 3usize;
//     let track_step = 1i64;
//     let mut track = Track::new(Interpolator, track_size, track_step);
//     track.reset_track(1);
//     track.push_aligned(0)?;
//     track.push_aligned(1)?;
//     track.push_aligned(2)?;

//     let key_end = *track.key_end();

//     track.truncate_forward(20);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 2);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.ranges[1], (1, 2));
//     assert_eq!(track.buf.len(), 3);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);
//     assert_eq!(*track.buf[2], 2);

//     track.truncate_forward(2);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 2);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.ranges[1], (1, 2));
//     assert_eq!(track.buf.len(), 3);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);
//     assert_eq!(*track.buf[2], 2);

//     track.truncate_forward(1);
//     assert_eq!(track.key_end, 2);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.buf.len(), 2);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);

//     track.truncate_forward(0);
//     assert_eq!(track.key_end, 2);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 1));
//     assert_eq!(track.buf.len(), 2);
//     assert_eq!(*track.buf[0], 0);
//     assert_eq!(*track.buf[1], 1);

//     Ok(())
// }

// fn test_truncate_forward_two_ranges_not_aligned() -> Result<()> {
//     let mut track = Track::new(Interpolator, 3, 100);
//     track.reset_track(100);
//     track.push_aligned(100)?;
//     track.push_not_aligned(125, DataNA(125), 200, 200)?;
//     track.push_not_aligned(150, DataNA(150), 200, 200)?;
//     track.push_not_aligned(175, DataNA(175), 200, 200)?;
//     track.push_aligned(200)?;
//     track.push_aligned(300)?;

//     let key_end = *track.key_end();

//     track.truncate_forward(2000);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 2);
//     assert_eq!(track.ranges[0], (0, 4));
//     assert_eq!(track.ranges[1], (4, 5));
//     assert_eq!(track.buf.len(), 6);
//     assert_eq!(*track.buf[0], 100);
//     assert_eq!(*track.buf[1], 125);
//     assert_eq!(*track.buf[2], 150);
//     assert_eq!(*track.buf[3], 175);
//     assert_eq!(*track.buf[4], 200);
//     assert_eq!(*track.buf[5], 300);

//     track.truncate_forward(200);
//     assert_eq!(track.key_end, key_end);
//     assert_eq!(track.ranges.len(), 2);
//     assert_eq!(track.ranges[0], (0, 4));
//     assert_eq!(track.ranges[1], (4, 5));
//     assert_eq!(track.buf.len(), 6);
//     assert_eq!(*track.buf[0], 100);
//     assert_eq!(*track.buf[1], 125);
//     assert_eq!(*track.buf[2], 150);
//     assert_eq!(*track.buf[3], 175);
//     assert_eq!(*track.buf[4], 200);
//     assert_eq!(*track.buf[5], 300);

//     track.truncate_forward(100);
//     assert_eq!(track.key_end, 200);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 4));
//     assert_eq!(track.buf.len(), 5);
//     assert_eq!(*track.buf[0], 100);
//     assert_eq!(*track.buf[1], 125);
//     assert_eq!(*track.buf[2], 150);
//     assert_eq!(*track.buf[3], 175);
//     assert_eq!(*track.buf[4], 200);

//     track.truncate_forward(0);
//     assert_eq!(track.key_end, 200);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (0, 4));
//     assert_eq!(track.buf.len(), 5);
//     assert_eq!(*track.buf[0], 100);
//     assert_eq!(*track.buf[1], 125);
//     assert_eq!(*track.buf[2], 150);
//     assert_eq!(*track.buf[3], 175);
//     assert_eq!(*track.buf[4], 200);

//     Ok(())
// }

// fn test_truncate_forward_seq() -> Result<()> {
//     let key_step = 1;
//     let key_start = 1;
//     let mut track = Track::new(Interpolator, 6, key_step);

//     track.reset_track(key_start);
//     track.push_aligned(0)?;
//     track.push_aligned(1)?;
//     track.push_aligned(2)?;
//     track.push_aligned(3)?;
//     track.push_aligned(4)?;
//     track.push_aligned(5)?;

//     track.truncate_back(5);
//     assert_eq!(track.ranges.len(), 1);
//     assert_eq!(track.ranges[0], (4, 5));
//     assert_eq!(track.buf.len(), 2);
//     assert_eq!(*track.buf[0], 4);
//     assert_eq!(*track.buf[1], 5);
    
//     track.push_aligned(6)?;
//     track.push_aligned(7)?;
//     track.push_aligned(8)?;
//     track.push_aligned(9)?;

//     track.truncate_forward(8);
//     assert_eq!(track.key_end, 9);
//     assert_eq!(track.ranges.len(), 4);
//     assert_eq!(track.ranges[0], (4, 5));
//     assert_eq!(track.ranges[1], (5, 6));
//     assert_eq!(track.ranges[2], (6, 7));
//     assert_eq!(track.ranges[3], (7, 8));
//     assert_eq!(track.buf.len(), 5);
//     assert_eq!(*track.buf[0], 4);
//     assert_eq!(*track.buf[1], 5);
//     assert_eq!(*track.buf[2], 6);
//     assert_eq!(*track.buf[3], 7);
//     assert_eq!(*track.buf[4], 8);

//     Ok(())
// }