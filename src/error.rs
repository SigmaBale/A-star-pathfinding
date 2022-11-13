#![allow(dead_code)]
use std::fmt::Display;

/// [`Error`] type that is defined specifically for [`crate::Maze`] type
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Error { kind: value }
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    InvalidFilePath,
    InvalidCharacters,
    MazeIsNotSolvable,
    MazeNotSolved,
    MazeIsNotSet,
    StartEndNotSet,
}

impl ErrorKind {
    pub fn as_str(&self) -> &str {
        use ErrorKind::*;
        match *self {
            InvalidFilePath => "Invalid file path",
            InvalidCharacters => "Characters are not unique. (start, end, wall...)",
            MazeIsNotSet => "Maze is not set (loaded), consider using `set` method on `Maze`.",
            MazeIsNotSolvable => "This maze is unsolvable.",
            MazeNotSolved => "Could not retrieve path, maze is not yet solved.",
            StartEndNotSet => "Start/End are not set.",
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
