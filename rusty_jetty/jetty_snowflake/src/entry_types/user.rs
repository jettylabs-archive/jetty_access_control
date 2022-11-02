use anyhow::Result;
use serde::{de, Deserialize, Serialize};

/// Snowflake User entry.
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct User {
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub login_name: String,
    pub display_name: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub disabled: bool,
}

impl User {
    pub fn new(
        name: String,
        first_name: String,
        last_name: String,
        email: String,
        login_name: String,
        display_name: String,
        disabled: bool,
    ) -> Self {
        Self {
            name,
            first_name,
            last_name,
            email,
            login_name,
            display_name,
            disabled,
        }
    }
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => panic!("unknown value for disabled field"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_disabled_is_deserializable() -> Result<()> {
        let user_json = json! {
            {
                "name": "name",
                "first_name": "first",
                "last_name": "last",
                "email": "elliot@allsafe.com",
                "login_name": "hax0rz",
                "display_name": "honeypot",
                "disabled": "false",
            }
        };

        let user: User = serde_json::from_value(user_json)?;
        assert!(!user.disabled);

        let user_json = json! {
            {
                "name": "name",
                "first_name": "first",
                "last_name": "last",
                "email": "elliot@allsafe.com",
                "login_name": "hax0rz",
                "display_name": "honeypot",
                "disabled": "true",
            }
        };

        let user: User = serde_json::from_value(user_json)?;
        assert!(user.disabled);
        Ok(())
    }
}
