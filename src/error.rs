use ::std::env::VarError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, ::thiserror::Error)]
pub enum Error {
    #[error("{0} should be set.")]
    VarNotPresent(String),

    #[error("{0} should be valid.")]
    VarNotValid(String),
}

impl Error {
    pub fn from_var_error(error: VarError, key: &str) -> Self {
        match error {
            VarError::NotPresent => Self::VarNotPresent(key.to_owned()),
            VarError::NotUnicode(_) => Self::VarNotValid(key.to_owned()),
        }
    }
}