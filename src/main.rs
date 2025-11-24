mod config;
mod ctrl_c;
mod discord_bot;
mod dns_client;
mod error;
mod message;
mod scheduler;

use ::log::info;
use ::log::warn;
use ::tokio::spawn;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;

use crate::config::Config;
use crate::discord_bot::DiscordBot;
use crate::discord_bot::DiscordClient;
use crate::dns_client::Dig;
use crate::dns_client::DigCommand;
use crate::dns_client::DnsClient;
use crate::error::Result;
use crate::message::Message;
use crate::scheduler::Scheduler;


#[::tokio::main]
async fn main() -> Result<()> {
    ::env_logger::init();
    let config = Config::from_env()?;

    let (kill_tx, kill_rx) = oneshot::channel();
    let (schedule_tx, schedule_rx) = mpsc::channel(8);

    let scheduler_handle = spawn(async move {
        let mut scheduler = Scheduler::new(
            std::time::Duration::from_secs(config.check_frequency),
            kill_rx,
        );

        scheduler.run(schedule_tx).await;
    });

    let runner_handle = spawn(async move {
        run(config, schedule_rx).await;
    });

    ctrl_c::detect(kill_tx).await;
    let _ = scheduler_handle.await;
    let _ = runner_handle.await;

    info!("graceful stop!!");
    Ok(())
}

async fn run(config: Config, mut schedule_rx: mpsc::Receiver<()>) {
    let command = DigCommand::new();
    let dig = Dig::new(command);

    let client = DiscordClient::new(&config.discord_token).await.unwrap();
    let bot = DiscordBot::new(client, config.channel_id);

    while let Some(_) = schedule_rx.recv().await {
        if let Err(e) = handle(&dig, &bot).await {
            warn!("{e}");
        }
    }
}

async fn handle(
    dns_client: &impl DnsClient,
    discord_bot: &DiscordBot<DiscordClient>
) -> std::result::Result<(), String> {
    let ip_addr = dns_client
        .get_ip_addr()
        .await
        .map_err(|e| e.to_string())?;

    let message = Message::Change(ip_addr);
    discord_bot.send(&message).await;

    Ok(())
}