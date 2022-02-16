use std::sync::Arc;
use std::time;
use serenity::{Client};
use serenity::builder::CreateChannel;
use serenity::model::channel::ChannelType;
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use serenity::prelude::TypeMapKey;

pub struct VoiceChatData {
    pub next_channel_id: u128,
    pub last_channel_id: ChannelId,
}
pub struct VoiceChat;

impl TypeMapKey for VoiceChat {
    type Value = VoiceChatData;
}

pub struct VoiceManager<'a> {
    client: &'a Client
}

impl<'a> VoiceManager<'a> {
    pub async fn new(client: &'a Client) -> VoiceManager<'a> {
        let vm = VoiceManager{ client };

        let guild = Guild::get(&client.cache_and_http.http, 941431307269963877)
            .await
            .expect("cannot get guild");
        for (_id, c) in guild
            .channels(&client.cache_and_http.http)
            .await
            .expect("cannot get channels")
            .iter()
        {
            if c.category_id == Some(ChannelId(941469281730838578)) && c.kind == ChannelType::Voice {
                c.delete(&client.cache_and_http.http)
                    .await
                    .expect("delete failed");
            }
        }

        let last_channel_id = guild
            .create_channel(&client.cache_and_http.http, channel_creator(0))
            .await
            .expect("cannot create channel")
            .id;
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
                    if c.category_id == Some(ChannelId(941469281730838578))
                        && c.kind == ChannelType::Voice
                    {
                        match c.members(&cache_and_http.cache).await {
                            Ok(members) => {
                                if members.is_empty() {
                                    c.delete(&cache_and_http.http).await.expect("delete failed");
                                }
                            }
                            Err(_) => {
                                todo!()
                            }
                        }
                    }
                }

                tokio::time::sleep(time::Duration::from_secs(600)).await;
            }
        });

        vm
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