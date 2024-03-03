use derive_more::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
#[display(fmt = "{self:?}")]
pub struct FrameError {
    pub frame_no: usize,
}

#[derive(Debug, Display, Error)]
pub struct EmptyAttackerPathError;

#[derive(Debug, Display, Error)]
pub struct EmptyDefenderPathError;

#[derive(Debug, Display, Error)]
#[display(fmt = "{self:?}")]
pub struct KeyError {
    pub key: i32,
    pub hashmap: String,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "{self:?}")]
pub struct MapSpaceRotationError {
    pub map_space_id: i32,
}
