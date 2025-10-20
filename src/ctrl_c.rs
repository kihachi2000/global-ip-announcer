use ::tokio::signal::ctrl_c;
use ::tokio::sync::oneshot;

pub async fn detect(tx: oneshot::Sender<()>) {
    ctrl_c().await.unwrap();
    tx.send(()).unwrap();
}
