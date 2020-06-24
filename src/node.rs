use {
    std::{
        ops::Deref,
        marker::PhantomData,
        fmt::Debug,
    },
    crate::TrackKey,
};

#[derive(Debug, Clone)]
pub enum TrackNode<Key, Data, NA> 
where
    Key: TrackKey,
    Data: Debug + Default + Clone,
    NA: Debug + Deref<Target=Data> + Clone
{
    Aligned(Data),
    NotAligned(NotAlignedNode<Key, Data, NA>)
}

impl<Key, Data, NA> Default for TrackNode<Key, Data, NA>
where
    Key: TrackKey,
    Data: Debug + Default + Clone,
    NA: Debug + Deref<Target=Data> + Clone
{
    fn default() -> Self {
        Self::Aligned(Data::default())
    }
}

impl<Key, Data, NA> From<Data> for TrackNode<Key, Data, NA>
where
    Key: TrackKey,
    Data: Debug + Default + Clone,
    NA: Debug + Deref<Target=Data> + Clone
{
    fn from(node: Data) -> Self {
        Self::Aligned(node)
    }
}

impl<Key, Data, NA> From<NotAlignedNode<Key, Data, NA>> for TrackNode<Key, Data, NA>
where
    Key: TrackKey,
    Data: Debug + Default + Clone,
    NA: Debug + Deref<Target=Data> + Clone
{
    fn from(node: NotAlignedNode<Key, Data, NA>) -> Self {
        Self::NotAligned(node)
    }
}

impl<Key, Data, NA> Deref for TrackNode<Key, Data, NA>
where
    Key: TrackKey,
    Data: Debug + Default + Clone,
    NA: Debug + Deref<Target=Data> + Clone
{
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Aligned(aligned) => aligned,
            Self::NotAligned(not_aligned) => not_aligned.deref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotAlignedNode<Key, C, NA>
where
    Key: TrackKey,
    C: Debug + Clone,
    NA: Debug + Clone,
{
    pub(crate) node: NA,
    pub(crate) key: Key,
    pub(crate) canceled_node: C,
    pub(crate) canceled_key: Key,

    pub(crate) phantom: PhantomData<C>
}

impl<Key, C, NA> NotAlignedNode<Key, C, NA>
where
    Key: TrackKey,
    C: Debug + Clone,
    NA: Debug + Clone,
{
    pub fn canceled_node(&self) -> &C {
        &self.canceled_node
    }

    pub fn canceled_key(&self) -> &Key {
        &self.canceled_key
    }
}

impl<Key, C, NA> Deref for NotAlignedNode<Key, C, NA> 
where
    Key: TrackKey,
    C: Debug + Clone,
    NA: Debug + Clone,
{
    type Target = NA;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}