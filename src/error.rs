use failure::Error;
use failure_derive::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum DotError {
    #[fail(display = "git dir is not configured")]
    NoGitDir,
    #[fail(display = "{}: {}", msg, error)]
    Wrap { msg: String, error: Error },
    #[fail(display = "no file have matched for path {}", 0)]
    NoMatch(String),
    #[fail(display = "cannot checksum directory")]
    ChecksumDir,
    #[fail(display = "path {} does not exists", 0)]
    NotFound(String),
    #[fail(display = "cannot copy different file type")]
    InvalidCopy,
}

impl DotError {
    pub fn wrap<S: AsRef<str>, E: Into<Error>>(msg: S, error: E) -> DotError {
        DotError::Wrap {
            msg: msg.as_ref().to_string(),
            error: error.into(),
        }
    }
}
