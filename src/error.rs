use failure::Error;
use failure_derive::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum DotError {
    #[fail(display = "invalid config path: {}", path)]
    InvalidConfig { path: String },
}
