use crate::Error;

pub trait Config: virtual_voting::Config + blockdag::Config<ErrorType = Error> {}
