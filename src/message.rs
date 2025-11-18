#[derive(Debug, PartialEq)]
pub enum Message {
    Reboot(String),
    Change(String),
}

impl Message {
    pub fn to_string(&self) -> String {
        match self {
            Self::Reboot(ip_addr) => format!("System rebooted. Current IP address is: ```{}```", ip_addr),
            Self::Change(ip_addr) => format!("IP address changed. ```{}```", ip_addr),
        }
    }
}