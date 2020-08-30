use thiserror::Error;

/// An error within Skia and its interactions with OpenGL.
#[derive(Error, Debug)]
pub enum SkiaError {
    #[error("the OpenGL target {0} is invalid")]
    InvalidTarget(String),
    #[error("invalid OpenGL context")]
    InvalidContext,
    #[error("unknown skia error")]
    UnknownError,
}
