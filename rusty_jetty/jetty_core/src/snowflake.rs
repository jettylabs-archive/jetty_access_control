use crate::ConnectorConfig;

struct SnowflakeCredentials{
    connector_type:String,
    account:String,
    password:String,
    role:String,
    user:String,
    warehouse:String,
}



impl ConnectorCredentials for SnowflakeCredentials{}