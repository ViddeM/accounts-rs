use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IdToken {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub")]
    pub subject: String,
    // Can also be a list of strings.
    #[serde(rename = "aud")]
    pub audience: String,
    // Seconds since 1970-01-01T00:00:00Z measured in UTC.
    #[serde(rename = "exp")]
    pub expires_at: i128,
    // Seconds since 1970-01-01T00:00:00Z measured in UTC.
    #[serde(rename = "iat")]
    pub issued_at: i128,
}
