mod ctrl_c;
mod discord_bot;
mod dns_client;
mod error;
mod scheduler;

use ::log::info;
use ::std::env;
use ::std::env::VarError;
use ::tokio::spawn;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;

use discord_bot::DiscordBot;
use dns_client::Dig;
use error::Error;
use error::Result;
use scheduler::Scheduler;


fn env_var(key: &str) -> Result<String> {
    env::var(key)
        .map_err(|e| {
            match e {
                VarError::NotPresent => Error::VarNotPresent(key.to_owned()),
                VarError::NotUnicode(_) => Error::VarNotValid(key.to_owned()),
            }
        })
}

fn env_var_u64(key: &str) -> Result<u64> {
    let parse = |value: String| -> Result<u64> {
        value.parse().map_err(|_| Error::VarNotValid(key.to_owned()))
    };

    env_var(key).and_then(parse)
}

fn env_var_vec_u64(key: &str) -> Result<Vec<u64>> {
    let values = env_var(key)?;
    let values = values
        .split(',')
        .flat_map(str::parse)
        .collect::<Vec<_>>();
    
    if values.len() == 0 {
        Err(Error::VarNotValid(key.to_owned()))
    } else {
        Ok(values)
    }
}

#[::tokio::main]
async fn main() -> Result<()> {
    ::env_logger::init();
    let check_frequency = env_var_u64("CHECK_FREQUENCY").unwrap_or(120);
    let discord_token = env_var("DISCORD_TOKEN")?;
    let channel_ids = env_var_vec_u64("CHANNEL_ID")?;

    let (kill_tx, kill_rx) = oneshot::channel();
    let (schedule_tx, schedule_rx) = mpsc::channel(8);
    let (ip_addr_tx, ip_addr_rx) = mpsc::channel(8);

    let scheduler_handle = spawn(async move {
        let mut scheduler = Scheduler::new(
            std::time::Duration::from_secs(check_frequency),
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
        let mut bot = DiscordBot::new(ip_addr_rx, &discord_token, channel_ids).await.unwrap();
        bot.run().await;
    });

    ctrl_c::detect(kill_tx).await;
    let _ = discord_bot_handle.await;
    let _ = scheduler_handle.await;
    let _ = dns_client_handle.await;

    info!("graceful stop!!");
    Ok(())
}
