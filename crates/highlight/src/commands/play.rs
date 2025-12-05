use stoat::{
    ChannelExt, FFmpegPCMAudio, VoiceConnection,
    commands::{Command, Context, server_only},
    types::{Channel, DataJoinCall},
};

use crate::{Error, State};

async fn play(ctx: Context<Error, State>, channel: Channel) -> Result<(), Error> {
    if !matches!(channel, Channel::TextChannel { voice: Some(_), .. }) {
        ctx.get_current_channel()
            .await?
            .send(&ctx.http)
            .content("Not a voice channel".to_string())
            .build()
            .await?;

        return Ok(());
    };

    let server = ctx.get_current_server().await?;

    if channel.server().is_none_or(|s| s != server.id) {
        ctx.get_current_channel()
            .await?
            .send(&ctx.http)
            .content("Channel not in this server".to_string())
            .build()
            .await?;

        return Ok(());
    }

    let data = ctx
        .http
        .join_call(
            channel.id(),
            &DataJoinCall {
                node: Some(
                    ctx.cache
                        .api_config
                        .features
                        .livekit
                        .nodes
                        .first()
                        .unwrap()
                        .name
                        .clone(),
                ),
                force_disconnect: None,
                recipients: None,
            },
        )
        .await?;

    ctx.get_current_channel()
        .await?
        .send(&ctx.http)
        .content(format!("Connecting to <#{}>!", channel.id()))
        .build()
        .await?;

    let conn = VoiceConnection::connect(&ctx.cache, &data.url, &data.token).await?;

    conn.play(FFmpegPCMAudio::new("audio.wav")).await?;

    conn.disconnect().await?;

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("play", play)
        .description("Plays an audio file in a voice channel")
        .signature("<channel>")
        .check(server_only)
}
