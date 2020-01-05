//! Cargo shell logger trait.

use std::fmt::Display;

/// Logger trait.
pub trait Logger {
    fn warn<T: Display>(&self, message: T);
    fn error<T: Display>(&self, message: T);
    fn status<S: Display, T: Display>(&self, status: S, message: T);
}

impl Logger for cargo::core::Workspace<'_> {
    fn warn<T: Display>(&self, message: T) {
        self.config().shell().warn(message).unwrap();
    }

    fn error<T: Display>(&self, message: T) {
        self.config().shell().error(message).unwrap();
    }

    fn status<S: Display, T: Display>(&self, status: S, message: T) {
        self.config().shell().status(status, message).unwrap();
    }
}
