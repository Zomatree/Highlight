use std::{collections::HashMap, fmt::Debug};

use crate::{
    Error,
    commands::{CommandReturn, Context},
};

#[derive(Debug, Clone)]
pub struct Command<E: From<Error> + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> {
    pub name: String,
    pub handle: for<'a> fn(&'a mut Context<'_, E, S>) -> CommandReturn<'a, E>,
    pub children: HashMap<String, Command<E, S>>,
    pub description: Option<String>,
}
