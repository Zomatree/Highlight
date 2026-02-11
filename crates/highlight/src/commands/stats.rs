use stoat::commands::{Command, Context};

use crate::{Error, State};

async fn stats(ctx: Context<Error, State>) -> Result<(), Error> {
    let mut system = ctx.state.system.lock().await;

    system.refresh_all();

    let cpu_usage = system.global_cpu_usage();
    let used_mem = system.used_memory() / 1024 / 1024;
    let total_mem = system.total_memory() / 1024 / 1024;
    let mem_perc = (used_mem as f32 / total_mem as f32) * 100.;

    ctx.send()
        .content(format!(
            "\
```
CPU: {cpu_usage:.2}%
Memory: {used_mem}MB / {total_mem}MB ({mem_perc:.2}%)
```"
        ))
        .build()
        .await?;

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("stats", stats)
        .hidden()
        .description("Bot statistics.")
}
