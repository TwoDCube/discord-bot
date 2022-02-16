mod voice_manager;

use std::{env};
use serenity::model::channel::{Channel, ChannelType};
use serenity::builder::CreateChannel;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::voice::VoiceState;
use serenity::{async_trait, Client, model::gateway::Ready};
use serenity::client::{Context, EventHandler};
use serenity::prelude::{SerenityError};
use crate::voice_manager::{VoiceManager, VoiceChat};

#[derive(Debug, thiserror::Error)]
enum HandlerError {
    #[error("some serenity error")]
    SerenityError(#[from] SerenityError),
    #[error("something else: {0}")]
    SomethingElse(String),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        let result = || async {
            if old.is_none() || old.as_ref().unwrap().channel_id != new.channel_id {
                if let Some(channel_id) = new.channel_id {
                    if let Channel::Guild(_) = channel_id.to_channel(&ctx.http).await? {
                        let mut data = ctx.data.write().await;
                        let voice_data = data
                            .get_mut::<VoiceChat>()
                            .expect("somehow didn't find, developer is stupid");

                        if voice_data.last_channel_id == channel_id {
                            let channel = guild
                                .unwrap()
                                .create_channel(&ctx.http, channel_creator(voice_data.next_channel_id))
                                .await?;

                            println!("joined");
                            voice_data.next_channel_id += 1;
                            voice_data.last_channel_id = channel.id;
                        }
                    }
                }
            }

            if let Some(VoiceState {
                channel_id: Some(channel_id),
                ..
            }) = old
            {
                if let Channel::Guild(gc) = channel_id.to_channel(&ctx.http).await? {
                    if gc.category_id != Some(ChannelId(941469281730838578)) {
                        return Ok(());
                    }

                    let members = gc.members(&ctx.cache).await?;

                    if members.is_empty() {
                        gc.delete(&ctx.http).await?;
                        println!("removed");
                    }
                }
            }
            Ok::<_, HandlerError>(())
        };

        if let Err(err) = result().await {
            todo!("implement error: {}", err);
        }
    }
}

fn channel_creator(id: u128) -> impl FnOnce(&mut CreateChannel) -> &mut CreateChannel {
    move |c| {
        c.name(format!("Voice - {}", id))
            .kind(ChannelType::Voice)
            .category(941469281730838578)
            .bitrate(128000)
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let voice_manager = VoiceManager::new(&client);

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
