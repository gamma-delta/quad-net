#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    Misc(String),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IOError(error)
    }
}
