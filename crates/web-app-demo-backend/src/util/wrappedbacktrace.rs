use std::{backtrace::Backtrace, fmt::Display};

// To prevent thiserror::Error trying to
// smart but nightly things for us.
#[derive(Debug)]
pub struct WrappedBacktrace(pub Backtrace);

impl Display for WrappedBacktrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
