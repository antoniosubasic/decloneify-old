use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Deserializer};

pub struct Token {
    pub access_token: String,
    pub expiration: DateTime<Utc>,
}

#[derive(Deserialize)]
struct TokenData {
    access_token: String,
    expires_in: u64,
}

impl<'de> Deserialize<'de> for Token {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let token_data = TokenData::deserialize(deserializer)?;
        let expiration = Utc::now() + Duration::seconds(token_data.expires_in as i64 - 1);

        Ok(Token {
            access_token: token_data.access_token,
            expiration,
        })
    }
}
