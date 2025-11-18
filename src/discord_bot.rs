use ::log::info;
use ::serenity::all::ChannelId;
use crate::error::Error;
use crate::error::Result;
use crate::message::Message;

#[cfg_attr(test, ::mockall::automock)]
trait Client {
    async fn send(&self, channel_id: &ChannelId, message: &Message);
}

pub struct DiscordClient {
    inner: serenity::Client,
}

impl DiscordClient {
    pub async fn new(token: &str) -> Result<Self> {
        let intents = ::serenity::all::GatewayIntents::GUILD_MESSAGES;
        let client = ::serenity::Client::builder(token, intents)
            .await
            .map_err(|_| Error::VarNotValid("DISCORD_TOKEN".to_owned()))?;

        Ok(Self {
            inner: client
        })
    }
}

impl Client for DiscordClient {
    async fn send(&self, channel_id: &ChannelId, message: &Message) {
        let result = channel_id.say(&self.inner.http, message.to_string()).await;

        match result {
            Ok(_) => info!("Succeeded to send message to {}.", &channel_id),
            Err(_) => info!("Failed to send message to {}.", &channel_id),
        }
    }
}

pub struct DiscordBot<C: Client> {
    client: C,
    channel_id_list: Vec<ChannelId>,
}

impl<C> DiscordBot<C> where C: Client {
    pub fn new(client: C, channel_id_list: Vec<u64>) -> Self {
        let channel_id_list = channel_id_list
            .into_iter()
            .map(|channel_id| ChannelId::new(channel_id))
            .collect();

        Self {
            client,
            channel_id_list,
        }
    }

    pub async fn send(&self, message: &Message) {
        let futures = self
            .channel_id_list
            .iter()
            .map(|channel_id| self.client.send(channel_id, message))
            .collect::<Vec<_>>();

        for future in futures {
            future.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use ::mockall::predicate;
    use super::*;

    #[::tokio::test]
    async fn send_to_single_channel() {
        let mut client = MockClient::new();
        client
            .expect_send()
            .return_const(())
            .with(
                predicate::eq(ChannelId::new(100)),
                predicate::eq(Message::Reboot("".to_owned())),
            )
            .times(1);
        let channel_id_list = vec![100];
        let bot = DiscordBot::new(client, channel_id_list);
        bot.send(&Message::Reboot("".to_owned())).await;
    }

    #[::tokio::test]
    async fn send_to_multiple_channels() {
        let mut client = MockClient::new();
        client
            .expect_send()
            .return_const(())
            .times(3);
        let channel_id_list = vec![100, 200, 300];
        let bot = DiscordBot::new(client, channel_id_list);
        bot.send(&Message::Reboot("".to_owned())).await;
    }
}