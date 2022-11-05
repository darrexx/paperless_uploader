use std::fmt::Display;

use log::error;

pub(crate) trait LogError {
    fn log_if_error(self, message: impl Display) -> Self;
}

impl<T, E> LogError for Result<T, E>
where
    E: Display,
{
    fn log_if_error(self, message: impl Display) -> Self {
        match self {
            Ok(_) => self,
            Err(ref err) => {
                error!("{} : {}", message, err);
                self
            }
        }
    }
}

impl<T> LogError for Option<T> {
    fn log_if_error(self, message: impl Display) -> Self {
        match self {
            Some(_) => self,
            None => {
                error!("{}", message);
                self
            }
        }
    }
}
