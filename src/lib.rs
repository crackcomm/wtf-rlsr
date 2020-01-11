#![feature(str_strip, try_trait)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

pub(crate) mod cmd;
pub(crate) mod git;
pub(crate) mod parser;
pub(crate) mod ui;
pub(crate) mod updater;
pub(crate) mod util;
pub(crate) mod ws;

pub use self::cmd::execute;
