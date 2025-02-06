#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),


    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}
