use derive_more::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
#[display(fmt = "{self:?}")]
pub struct DieselError<'a> {
    pub table: &'a str,
    pub function: &'a str,
    pub error: diesel::result::Error,
}
