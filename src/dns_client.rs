use ::tokio::process::Command;
use ::tokio::sync::mpsc;

pub struct Dig {
    schedule_rx: mpsc::Receiver<()>,
    addr_tx: mpsc::Sender<String>,
}

impl Dig {
    pub fn new(schedule_rx: mpsc::Receiver<()>, addr_tx: mpsc::Sender<String>) -> Self {
        Self {
            schedule_rx,
            addr_tx,
        }
    }

    pub async fn run(&mut self) {
        fn gen_command() -> Command {
            let mut command = Command::new("dig");
            command.arg("-4");
            command.arg("@ns1.google.com");
            command.arg("o-o.myaddr.l.google.com");
            command.arg("TXT");
            command.arg("+short");
            command
        }

        fn extract_ip_addr(source: String) -> String {
            fn is_num_or_period(c: &char) -> bool {
                ('0' <= *c && *c <= '9') || *c == '.'
            }

            source
                .chars()
                .skip_while(|c| !is_num_or_period(c))
                .take_while(|c| is_num_or_period(c))
                .collect()
        }

        while let Some(_) = self.schedule_rx.recv().await {
            let mut command = gen_command();

            if let Ok(output) = command.output().await {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
                    let ip_addr = extract_ip_addr(stdout);
                    self.addr_tx.send(ip_addr).await.unwrap();
                }
            }
        }
    }
}
