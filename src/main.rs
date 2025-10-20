mod ctrl_c;
mod scheduler;

use ::tokio::spawn;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;

use scheduler::Scheduler;

#[::tokio::main]
async fn main() {
    let (kill_tx, kill_rx) = oneshot::channel();
    let (schedule_tx, schedule_rx) = mpsc::channel(5);

    let scheduler_handle = spawn(async move {
        let mut scheduler = Scheduler::new(
            std::time::Duration::from_secs(5),
            kill_rx,
        );

        scheduler.run(schedule_tx).await;
    });

    ctrl_c::detect(kill_tx).await;

    let _ = scheduler_handle.await;
}
