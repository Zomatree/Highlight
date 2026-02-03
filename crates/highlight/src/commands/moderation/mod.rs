use stoat::commands::Command;

use crate::{Error, State};

mod purge;
mod timeout;

pub fn commands() -> Vec<Command<Error, State>> {
    vec![timeout::command(), purge::command()]
}
