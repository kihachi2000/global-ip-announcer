use ::log::info;
use ::serenity::all::ChannelId;
use ::serenity::prelude::*;
use ::tokio::sync::mpsc::Receiver;

pub struct DiscordBot {
    client: Client,
    channel_id: u64,
    ip_addr_rx: Receiver<String>,
    current_ip_addr: String,
}

impl DiscordBot {
    pub async fn new(ip_addr_rx: Receiver<String>, token: &str, channel_id: u64) -> Result<Self, String> {
        let intents = GatewayIntents::GUILD_MESSAGES;
        let client = Client::builder(token, intents)
            .await
            .map_err(|_| "DISCORD_TOKEN should be valid.")?;

        Ok(Self {
            client,
            channel_id,
            ip_addr_rx,
            current_ip_addr: "".to_owned(),
        })
    }

    pub async fn run(&mut self) {
        while let Some(ip_addr) = self.ip_addr_rx.recv().await {
            if self.current_ip_addr != ip_addr {
                let channel_id = ChannelId::new(self.channel_id);

                let result = channel_id.say(
                    &self.client.http,
                    format!("Current global ip is {}", ip_addr)
                ).await;

                match result {
                    Ok(_) => {
                        info!("Succeeded to send message to the discord server.");
                        self.current_ip_addr = ip_addr;
                    },

                    Err(_) => info!("Failed to send message to the discord server."),
                }
            }
        }
    }
}