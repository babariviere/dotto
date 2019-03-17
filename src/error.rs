use failure::Error;
use failure_derive::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum DotError {
    #[fail(display = "git dir is not configured")]
    NoGitDir,
}
