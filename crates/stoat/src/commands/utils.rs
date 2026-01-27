use std::fmt::Debug;

use crate::{Error, commands::Context};

pub fn when_mentioned<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>(
    context: &Context<E, S>,
) -> Vec<String> {
    vec![
        format!(
            "<@{}> ",
            &context
                .cache
                .current_user_id
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
        )
    ]
}

pub fn when_mentioned_or<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>(
    context: &Context<E, S>,
    prefixes: &[String]
) -> Vec<String> {
    let mut v = vec![
        format!(
            "<@{}> ",
            &context
                .cache
                .current_user_id
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
        )
    ];

    v.extend_from_slice(prefixes);
    v
}
