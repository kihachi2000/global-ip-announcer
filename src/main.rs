mod ctrl_c;
mod discord_bot;
mod dns_client;
mod scheduler;

use ::log::info;
use ::std::env;
use ::tokio::spawn;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;

use discord_bot::DiscordBot;
use dns_client::Dig;
use scheduler::Scheduler;

#[::tokio::main]
async fn main() {
    ::env_logger::init();

    let (kill_tx, kill_rx) = oneshot::channel();
    let (schedule_tx, schedule_rx) = mpsc::channel(8);
    let (ip_addr_tx, ip_addr_rx) = mpsc::channel(8);

    let scheduler_handle = spawn(async move {
        let mut scheduler = Scheduler::new(
            std::time::Duration::from_secs(10),
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

    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN should be exported.");
    let channel_id = env::var("CHANNEL_ID").expect("CHANNEL_ID should be exported.");
    let channel_id = channel_id.parse::<u64>().expect("CHANNEL_ID should be number.");
    let discord_bot_handle = spawn(async move {
        let mut bot = DiscordBot::new(ip_addr_rx, &discord_token, channel_id).await.unwrap();
        bot.run().await;
    });

    ctrl_c::detect(kill_tx).await;
    let _ = discord_bot_handle.await;
    let _ = scheduler_handle.await;
    let _ = dns_client_handle.await;

    info!("graceful stop!!");
}
