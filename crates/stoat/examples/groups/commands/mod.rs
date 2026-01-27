use stoat::{
    async_trait,
    commands::{Command, CommandEventHandler},
};

use crate::{CmdCtx, Error};

mod ban;

#[derive(Clone)]
pub struct Commands;

#[async_trait]
impl CommandEventHandler for Commands {
    type State = ();
    type Error = Error;

    async fn get_prefix(&self, _ctx: CmdCtx) -> Result<Vec<String>, Error> {
        Ok(vec!["!".to_string()])
    }
}

pub fn commands() -> Vec<Command<Error, ()>> {
    vec![ban::command()]
}
