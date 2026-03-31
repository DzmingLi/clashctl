#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("{0}")]
    InteractiveError(#[from] crate::interactive::InteractiveError),

    #[error("{0}")]
    TuiError(#[from] crate::ui::TuiError),

    #[error("{0}")]
    ClashCtl(#[from] clashctl_core::Error),

    #[error("Requestty error")]
    RequesttyError(#[from] requestty::ErrorKind),

    #[error("{0}")]
    Custom(String),
}
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(Box<ErrorKind>);

impl<E> From<E> for Error
where
    ErrorKind: From<E>,
{
    fn from(err: E) -> Self {
        Error(Box::new(ErrorKind::from(err)))
    }
}

impl Error {
    pub fn custom(msg: String) -> Self {
        Error(Box::new(ErrorKind::Custom(msg)))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
