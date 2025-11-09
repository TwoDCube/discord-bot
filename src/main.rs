use std::env;

use serenity::builder::CreateChannel;
use serenity::model::channel::{Channel, ChannelType};
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use serenity::model::voice::VoiceState;
use serenity::{async_trait, model::gateway::Ready, prelude::*};

const VOICE_CHANNELS_CATEGORY_ID: ChannelId = ChannelId::new(941469281730838578);
const VOICE_CHANNEL_NAME_PREFIX: &str = "Voice - ";

#[derive(Debug, thiserror::Error)]
enum HandlerError {
    #[error("some serenity error")]
    SerenityError(#[from] SerenityError),
}

struct VoiceChatData {
    next_channel_id: u128,
    last_channel_id: ChannelId,
}
struct VoiceChat;

impl TypeMapKey for VoiceChat {
    type Value = VoiceChatData;
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

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        let mut largest_number: Option<u128> = None;

        for (_id, c) in guild
            .channels(&ctx.http)
            .await
            .expect("cannot get channels")
            .iter()
        {
            if c.parent_id != Some(VOICE_CHANNELS_CATEGORY_ID) || c.kind != ChannelType::Voice {
                continue;
            }

            if c.members(&ctx.cache).unwrap().is_empty() {
                c.delete((&ctx.cache, ctx.http.as_ref()))
                    .await
                    .expect("delete failed");
                continue;
            }

            let Some(Ok(number)) = c
                .name()
                .strip_prefix(VOICE_CHANNEL_NAME_PREFIX)
                .map(|n| n.parse::<u128>())
            else {
                continue;
            };

            largest_number.get_or_insert(number);
            largest_number.map(|n| n.max(number));
        }

        let new_number = largest_number.map(|n| n + 1).unwrap_or(0);

        let last_channel_id = guild
            .create_channel((&ctx.cache, ctx.http.as_ref()), channel_creator(new_number))
            .await
            .expect("cannot create channel")
            .id;

        let mut data = ctx.data.write().await;
        data.insert::<VoiceChat>(VoiceChatData {
            next_channel_id: new_number + 1,
            last_channel_id,
        });
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let result = || async {
            if old.is_none() || old.as_ref().unwrap().channel_id != new.channel_id {
                if let Some(channel_id) = new.channel_id {
                    if let Channel::Guild(_) = channel_id
                        .to_channel((&ctx.cache, ctx.http.as_ref()))
                        .await?
                    {
                        let mut data = ctx.data.write().await;
                        let voice_data = data
                            .get_mut::<VoiceChat>()
                            .expect("somehow didn't find, developer is stupid");

                        if voice_data.last_channel_id == channel_id {
                            let channel = new
                                .guild_id
                                .unwrap()
                                .create_channel(
                                    (&ctx.cache, ctx.http.as_ref()),
                                    channel_creator(voice_data.next_channel_id),
                                )
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
                if let Channel::Guild(gc) = channel_id
                    .to_channel((&ctx.cache, ctx.http.as_ref()))
                    .await?
                {
                    if gc.parent_id != Some(VOICE_CHANNELS_CATEGORY_ID) {
                        return Ok(());
                    }

                    let members = gc.members(&ctx.cache)?;

                    if members.is_empty() {
                        gc.delete((&ctx.cache, ctx.http.as_ref())).await?;
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

fn channel_creator<'a>(id: u128) -> CreateChannel<'a> {
    CreateChannel::new(format!("{VOICE_CHANNEL_NAME_PREFIX}{id}"))
        .kind(ChannelType::Voice)
        .category(VOICE_CHANNELS_CATEGORY_ID)
        .bitrate(128000)
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(
        &token,
        GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
    )
    .event_handler(Handler)
    .await
    .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
