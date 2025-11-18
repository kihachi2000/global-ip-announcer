use ::std::env;
use crate::error::Error;
use crate::error::Result;

#[derive(Debug, Default)]
pub struct Config {
    pub check_frequency: u64,
    pub discord_token: String,
    pub channel_id: Vec<u64>,
}

impl Config {
    pub fn new(check_frequency: u64, discord_token: String, channel_id: Vec<u64>) -> Self {
        Self {
            check_frequency,
            discord_token,
            channel_id,
        }
    }

    pub fn from_env() -> Result<Self> {
        let check_frequency_key = "CHECK_FREQUENCY";
        let check_frequency = get_env(check_frequency_key)
            .and_then(|value| parse_check_frequency(check_frequency_key, &value))
            .unwrap_or(300);

        let discord_token_key = "DISCORD_TOKEN";
        let discord_token = get_env(discord_token_key)?;

        let channel_id_key = "CHANNEL_ID";
        let channel_id = get_env(channel_id_key)
            .and_then(|value| parse_channel_id(channel_id_key, &value))?;

        Ok(Config {
            check_frequency,
            discord_token,
            channel_id
        })
    }
}

fn get_env(key: &str) -> Result<String> {
    env::var(key).map_err(|error| Error::from_var_error(error, key))
}

fn parse_check_frequency(key: &str, value: &str) -> Result<u64> {
    value
        .parse()
        .map_err(|_| Error::VarNotValid(key.to_owned()))
}

fn parse_channel_id(key: &str, value: &str) -> Result<Vec<u64>> {
    value
        .split(',')
        .map(|id| id.parse().map_err(|_| Error::VarNotValid(key.to_owned())))
        .collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_check_frequency_with_valid_value() {
        let result = parse_check_frequency("key", "100");
        assert_eq!(result, Ok(100));

        let result = parse_check_frequency("key", "200");
        assert_eq!(result, Ok(200));
    }

    #[test]
    fn parse_check_frequency_with_invalid_value() {
        let result = parse_check_frequency("key", "");
        assert_eq!(result, Err(Error::VarNotValid("key".to_owned())));
    }

    #[test]
    fn parse_channel_id_with_one_id() {
        let result = parse_channel_id("key", "100");
        assert_eq!(result, Ok(vec![100]));

        let result = parse_channel_id("key", "200");
        assert_eq!(result, Ok(vec![200]));
    }

    #[test]
    fn parse_channel_id_with_multiple_ids() {
        let result = parse_channel_id("key", "100,200");
        assert_eq!(result, Ok(vec![100, 200]));
    }

    #[test]
    fn parse_channel_id_with_invalid_id() {
        let result = parse_channel_id("key", "");
        assert_eq!(result, Err(Error::VarNotValid("key".to_owned())));

        let result = parse_channel_id("key", "100,invalid,300");
        assert_eq!(result, Err(Error::VarNotValid("key".to_owned())));
    }
}
