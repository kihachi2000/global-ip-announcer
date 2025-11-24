use ::log::info;
use ::regex::Regex;
use crate::error::DnsError;

#[cfg_attr(test, ::mockall::automock)]
trait Command {
    async fn execute(&self) -> Result<String, DnsError>;
}

pub struct DigCommand;

impl DigCommand {
    pub fn new() -> Self {
        Self
    }

    fn gen_command(&self) -> ::tokio::process::Command {
        let mut command = ::tokio::process::Command::new("dig");
        command.arg("-4");
        command.arg("@ns1.google.com");
        command.arg("o-o.myaddr.l.google.com");
        command.arg("TXT");
        command.arg("+short");
        command
    }
}

impl Command for DigCommand {
    async fn execute(&self) -> Result<String, DnsError> {
        let output = self
            .gen_command()
            .output()
            .await
            .map_err(|_| DnsError::CommandFailed("dig".to_owned()))?;

        if output.status.success() {
            String::from_utf8(output.stdout.clone())
                .map_err(|_| DnsError::CommandFailed("dig".to_owned()))
        } else {
            Err(DnsError::CommandFailed("dig".to_owned()))
        }
    }
}

#[cfg_attr(test, ::mockall::automock)]
pub trait DnsClient {
    async fn fetch(&self) -> Result<String, DnsError>;

    async fn get_ip_addr(&self) -> Result<String, DnsError> {
        let ip_addr = self.fetch().await?;

        if self.validate(&ip_addr) {
            Ok(ip_addr)
        } else {
            Err(DnsError::IpNotValid(ip_addr))
        }
    }

    fn validate(&self, ip_addr: &str) -> bool {
        let re = Regex::new(r"^((25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])$")
            .expect("the regular expression should be valid.");

        re.is_match(ip_addr)
    }
}

pub struct Dig<C: Command> {
    command: C,
}

impl<C: Command> Dig<C> {
    pub fn new(command: C) -> Self {
        Self {
            command
        }
    }

    fn drop_double_quotes(source: String) -> String {
        fn is_num_or_period(c: &char) -> bool {
            ('0' <= *c && *c <= '9') || *c == '.'
        }

        source
            .chars()
            .skip(1)
            .take_while(is_num_or_period)
            .collect()
    }
}

impl<C: Command> DnsClient for Dig<C> {
    async fn fetch(&self) -> Result<String, DnsError> {
        self.command
            .execute()
            .await
            .map(|output| Self::drop_double_quotes(output))
            .inspect(|ip_addr| info!("{ip_addr}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[::tokio::test]
    async fn can_execute_dig() {
        let mut command = MockCommand::new();
        command
            .expect_execute()
            .return_const(Ok(r#""0.0.0.0""#.to_owned()))
            .times(1);
        let dig_client = Dig::new(command);
        let result = dig_client.get_ip_addr().await;
        assert_eq!(result, Ok("0.0.0.0".to_owned()));
    }

    #[::tokio::test]
    async fn dig_is_not_executable() {
        let mut command = MockCommand::new();
        command
            .expect_execute()
            .return_const(Err(DnsError::CommandFailed("dig".to_owned())))
            .times(1);
        let dig_client = Dig::new(command);
        let result = dig_client.get_ip_addr().await;
        assert_eq!(result, Err(DnsError::CommandFailed("dig".to_owned())));
    }

    #[::tokio::test]
    async fn invalid_ip_addr() {
        let mut command = MockCommand::new();
        command
            .expect_execute()
            .return_const(Ok(r#""0000""#.to_owned()))
            .times(1);
        let dig_client = Dig::new(command);
        let result = dig_client.get_ip_addr().await;
        assert_eq!(result, Err(DnsError::IpNotValid("0000".to_owned())));

        let mut command = MockCommand::new();
        command
            .expect_execute()
            .return_const(Ok(r#""255.255.255.256""#.to_owned()))
            .times(1);
        let dig_client = Dig::new(command);
        let result = dig_client.get_ip_addr().await;
        assert_eq!(result, Err(DnsError::IpNotValid("255.255.255.256".to_owned())));
    }
}