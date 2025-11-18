mod config;
mod ctrl_c;
mod discord_bot;
mod dns_client;
mod error;
mod message;
mod scheduler;

use ::log::info;
use ::tokio::spawn;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;

use crate::config::Config;
use crate::discord_bot::DiscordBot;
use crate::discord_bot::DiscordClient;
use crate::dns_client::Dig;
use crate::error::Result;
use crate::message::Message;
use crate::scheduler::Scheduler;


#[::tokio::main]
async fn main() -> Result<()> {
    ::env_logger::init();
    let config = Config::from_env()?;

    let (kill_tx, kill_rx) = oneshot::channel();
    let (schedule_tx, schedule_rx) = mpsc::channel(8);
    let (ip_addr_tx, mut ip_addr_rx) = mpsc::channel(8);

    let scheduler_handle = spawn(async move {
        let mut scheduler = Scheduler::new(
            std::time::Duration::from_secs(config.check_frequency),
            kill_rx,
        );

        scheduler.run(schedule_tx).await;
    });

    let dns_client_handle = spawn(async move {
        let mut client = Dig::new(
            schedule_rx,
            ip_addr_tx,
        );

        client.run().await;
    });

    let discord_bot_handle = spawn(async move {
        let client = DiscordClient::new(&config.discord_token).await.unwrap();
        let bot = DiscordBot::new(client, config.channel_id);

        while let Some(ip_addr) = ip_addr_rx.recv().await {
            bot.send(&Message::Change(ip_addr)).await;
        }
    });

    ctrl_c::detect(kill_tx).await;
    let _ = discord_bot_handle.await;
    let _ = scheduler_handle.await;
    let _ = dns_client_handle.await;

    info!("graceful stop!!");
    Ok(())
}
