use ::std::time::Duration;
use ::tokio::sync::mpsc;
use ::tokio::sync::oneshot;
use ::tokio::time::interval;

pub struct Scheduler {
    period: Duration,
    kill_switch: oneshot::Receiver<()>,
}

impl Scheduler {
    pub fn new(period: Duration, kill_switch: oneshot::Receiver<()>) -> Self {
        Self {
            period,
            kill_switch,
        }
    }

    pub async fn run(&mut self, tx: mpsc::Sender<()>) {
        let mut interval = interval(self.period);

        loop {
            tokio::select! {
                _ = &mut self.kill_switch => {
                    break;
                },

                _ = interval.tick() => {
                    tx.send(()).await.unwrap();
                },
            }
        }
    }
}