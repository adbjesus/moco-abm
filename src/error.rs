#[derive(Debug)]
pub enum ErrorKind {
    EmptyApproximation,
    EmptyRegion,
    UnsortedSegment,
    UnsortedSegments,
    WrongDimensions,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind: kind,
            message: None,
        }
    }

    pub fn with_message(kind: ErrorKind, message: String) -> Self {
        Error {
            kind: kind,
            message: Some(message),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.message {
            Some(m) => writeln!(f, "{:?} - {}", self.kind, m),
            None => writeln!(f, "{:?}", self.kind),
        }
    }
}
