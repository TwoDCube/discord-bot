use std::{env, time};
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use serenity::model::channel::{Channel, ChannelType};

use serenity::model::guild::Guild;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::voice::VoiceState;
use serenity::{async_trait, model::gateway::Ready, prelude::*};
use serenity::builder::CreateChannel;
use serenity::model::channel::ChannelType::Voice;

struct VoiceChatData {
    next_channel_id: u128,
    last_channel_id: ChannelId
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

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if old.is_none() || old.as_ref().expect("").channel_id != new.channel_id {
            if let Some(channel_id) = new.channel_id {
                match channel_id.to_channel(&ctx.http).await {
                    Ok(c) => {
                        match c {
                            Channel::Guild(_gc) => {
                                let mut data = ctx.data.write().await;
                                let voice_data = data.get_mut::<VoiceChat>().expect("somehow didnt find, developer is stupid");
                                if voice_data.last_channel_id == channel_id {
                                    match guild.expect("guild nil").create_channel(&ctx.http, channel_creator(voice_data.next_channel_id)).await {
                                        Ok(channel) => {
                                            println!("joined");
                                            voice_data.next_channel_id+=1;
                                            voice_data.last_channel_id = channel.id;
                                        }
                                        Err(_) => {todo!()}
                                    }

                                }

                            }
                            Channel::Private(_) => {todo!()}
                            Channel::Category(_) => {todo!()}
                            _ => {todo!()}
                        }
                    }
                    Err(_) => { todo!()                    }
                }
            }
        }
        if let Some(VoiceState{channel_id: Some(channel_id), ..}) = old {
            match channel_id.to_channel(&ctx.http).await {
                Ok(c) => {
                    match c {
                        Channel::Guild(gc) => {
                            if gc.category_id.unwrap_or_default().0 != 941469281730838578 {
                                return
                            }
                            match gc.members(&ctx.cache).await {
                                Ok(members) => {
                                    if members.len() == 0 {
                                        match gc.delete(&ctx.http).await {
                                            Ok(_) => {println!("removed")}
                                            Err(_) => {todo!()}
                                        }
                                    }
                                }
                                Err(_) => {todo!()}
                            }
                        }
                        Channel::Private(_) => {todo!()}
                        Channel::Category(_) => {todo!()}
                        _ => {todo!()}
                    }
                }
                Err(_) => {todo!()}
            }
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

    let guild = Guild::get(&client.cache_and_http.http, 941431307269963877)
        .await
        .expect("cannot get guild");
    for (_id, c) in guild
        .channels(&client.cache_and_http.http)
        .await
        .expect("cannot get channels")
        .iter()
    {
        if c.category_id.unwrap_or_default().0 == 941469281730838578 && c.kind == ChannelType::Voice {
            c.delete(&client.cache_and_http.http)
                .await
                .expect("delete failed");
        }
    }

    let last_channel_id = guild
        .create_channel(&client.cache_and_http.http, channel_creator(0))
        .await
        .expect("cannot create channel").id;
   {
        let mut data = client.data.write().await;

        data.insert::<VoiceChat>(VoiceChatData {
            next_channel_id: 1,
            last_channel_id,
        });
    }

    let cache_and_http = Arc::clone(&client.cache_and_http);
    tokio::spawn(async move {
        loop {
            for (_id, c) in guild
                .channels(&cache_and_http.http)
                .await
                .expect("cannot get channels")
                .iter()
            {
                if c.category_id.unwrap_or_default().0 == 941469281730838578 && c.kind == ChannelType::Voice {
                    match c.members(&cache_and_http.cache).await {
                        Ok(members) => {
                            if members.len() == 0 {
                                c.delete(&cache_and_http.http)
                                    .await
                                    .expect("delete failed");
                            }
                        }
                        Err(_) => {todo!()}
                    }

                }
            }

            tokio::time::sleep(time::Duration::from_secs(600)).await;
        }
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
