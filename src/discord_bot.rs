use ::log::info;
use ::serenity::all::ChannelId;
use ::serenity::prelude::*;
use ::tokio::sync::mpsc::Receiver;

pub struct DiscordBot {
    client: Client,
    ip_addr_rx: Receiver<String>,
}

impl DiscordBot {
    pub async fn new(ip_addr_rx: Receiver<String>, token: &str) -> Result<Self, String> {
        let intents = GatewayIntents::GUILD_MESSAGES;
        let client = Client::builder(token, intents)
            .await
            .map_err(|_| "discord token should be valid.")?;

        Ok(Self {
            client,
            ip_addr_rx,
        })
    }

    pub async fn run(&mut self) {
        while let Some(ip_addr) = self.ip_addr_rx.recv().await {
            let channel_id: u64 = 0;
            let channel_id = ChannelId::new(channel_id);

            let result = channel_id.say(
                &self.client.http,
                format!("Current global ip is {}", ip_addr)
            ).await;

            match result {
                Ok(_) => info!("Succeeded to send message to the discord server."),
                Err(_) => info!("Failed to send message to the discord server."),
            }
        }
    }
}