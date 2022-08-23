use crate::{
    connectors::Connector,
    jetty::{ConnectorConfig, CredentialsBlob},
};

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, EncodingKey, Header};
use reqwest;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

pub struct Snowflake {
    credentials: SnowflakeCredentials,
}
#[derive(Deserialize, Debug, Default)]
struct SnowflakeCredentials {
    account: String,
    password: String,
    role: String,
    user: String,
    warehouse: String,
    private_key: String,
    public_key_fp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    sub: String, // Optional. Subject (whom token refers to)
}

#[async_trait]
impl Connector for Snowflake {
    fn new(config: &ConnectorConfig, credentials: &CredentialsBlob) -> Result<Box<Self>> {
        let mut conn = SnowflakeCredentials::default();
        let mut required_fields: HashSet<_> = vec![
            "account",
            "password",
            "role",
            "user",
            "warehouse",
            "private_key",
            "public_key_fp",
        ]
        .into_iter()
        .collect();

        for (k, v) in credentials.iter() {
            match k.as_ref() {
                "account" => conn.account = v.to_string(),
                "password" => conn.password = v.to_string(),
                "role" => conn.role = v.to_string(),
                "user" => conn.user = v.to_string(),
                "warehouse" => conn.warehouse = v.to_string(),
                "private_key" => conn.private_key = v.to_string(),
                "public_key_fp" => conn.public_key_fp = v.to_string(),
                _ => (),
            }

            required_fields.remove::<str>(k);
        }

        if !required_fields.is_empty() {
            Err(anyhow![format![
                "Snowflake config missing required fields: {:#?}",
                required_fields
            ]])
        } else {
            Ok(Box::new(Snowflake { credentials: conn }))
        }
    }

    async fn check(&self) -> bool {
        let qualified_username = format![
            "{}.{}",
            self.credentials.account.to_uppercase(),
            self.credentials.user.to_uppercase()
        ];

        // Generate jwt
        let claims = Claims {
            exp: (get_current_timestamp() + 3600) as usize,
            iat: get_current_timestamp() as usize,
            iss: format!["{}.{}", qualified_username, self.credentials.public_key_fp],
            sub: qualified_username,
        };

        println!("{}", self.credentials.private_key);

        let token = encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(std::include_bytes!("/Users/jk/rsa_key.p8")).unwrap(),
        )
        .unwrap();

        // request

        // This will POST a body of `{"lang":"rust","body":"json"}`
        let mut body = HashMap::new();
        body.insert("statement", "SELECT 1");
        body.insert("warehouse", "main");
        body.insert("role", &self.credentials.role);

        let client = reqwest::Client::new();

        let res = client
            .post(format![
                "https://{}.snowflakecomputing.com/api/v2/statements",
                self.credentials.account
            ])
            .json(&body)
            .header("Authorization", format!["Bearer {}", token])
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Snowflake-Authorization-Token-Type", "KEYPAIR_JWT")
            .header("User-Agent", "test")
            .send()
            .await
            // each response is wrapped in a `Result` type
            // we'll unwrap here for simplicity
            .unwrap()
            .text()
            .await;
        println!["{:#?}", res];
        true
    }
}
