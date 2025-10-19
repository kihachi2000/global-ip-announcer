mod scheduler;
use scheduler::Scheduler;

use ::tokio::spawn;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;

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

    let _ = scheduler_handle.await;
}
