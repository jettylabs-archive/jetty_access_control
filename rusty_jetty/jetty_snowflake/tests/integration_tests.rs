use std::collections::HashMap;

use std::{boxed::Box, collections::HashSet};

use jetty_core::connectors::NewConnector;
use jetty_core::{
    connectors::{
        nodes::{self, RawGroup},
        ConnectorClient,
    },
    jetty::ConnectorConfig,
    logging::debug,
    Connector,
};
use jetty_snowflake::{RoleName, SnowflakeConnector};

use anyhow::Context;
use serde::Serialize;
use serde_json::Value;

use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct WiremockServer {
    pub server: Option<MockServer>,
}

#[derive(Serialize)]
struct SnowflakeField {
    name: String,
}
#[derive(Serialize)]
struct SnowflakeRowTypeFields {
    #[serde(rename = "rowType")]
    row_type: Vec<SnowflakeField>,
}
#[derive(Serialize)]
struct SnowflakeResult {
    #[serde(rename = "resultSetMetaData")]
    result_set_metadata: SnowflakeRowTypeFields,
    data: Vec<Vec<Option<String>>>,
}

/// Make a json body for the types from the input with the given pattern.
macro_rules! body_for {
    ($entry_type: pat, $input: expr, $($field:ident),+) => {
        serde_json::to_string(&SnowflakeResult {
            result_set_metadata: SnowflakeRowTypeFields {
                row_type: vec![$(SnowflakeField {
                    name: stringify!($field).to_owned(),
                }),+],
            },
            // Snowflake returns objects as arrays, so we need to do the same for testing.
            // Example: https://tinyurl.com/object-to-string-rust
            data: $input
                .entries
                .iter()
                .filter_map(|entry| {
                    // Only keep entries of this type.
                    if !matches!(entry, $entry_type){
                        return None;
                    }

                    if let Value::Object(obj) = serde_json::to_value(entry).unwrap(){
                        let vals = obj.values().cloned().map(|i|{

                            if let serde_json::Value::String(v) = i {
                                // Snowflake returns Option<String>.
                                Some(v)
                            } else {
                                // Shouldn't happen
                                panic!("bad entry field for snowflake body")
                            }
                        }).collect::<Vec<_>>();
                        Some(vals)
                    }else{
                        // Shouldn't happen
                        panic!("bad entry for snowflake body")
                    }
                })
                .collect(),
            })
        .context("building json body").unwrap()
    };
}

impl WiremockServer {
    pub fn new() -> Self {
        Self { server: None }
    }

    async fn init(mut self, input: &TestInput) -> Self {
        let mock_server = MockServer::start().await;
        self.server = Some(mock_server);

        let roles_body = body_for!(jetty_snowflake::Entry::Role(_), input, name);
        debug!("roles: {}", roles_body);
        let users_body = body_for!(
            jetty_snowflake::Entry::User(_),
            input,
            name,
            first_name,
            last_name,
            email,
            login_name,
            login_name,
            display_name
        );
        let grants_body = body_for!(
            jetty_snowflake::Entry::Grant(_),
            input,
            name,
            privilege,
            granted_on
        );
        let objects_body = body_for!(
            jetty_snowflake::Entry::Asset(jetty_snowflake::Asset::Object(_)),
            input,
            name,
            schema_name,
            database_name
        );
        debug!("objects body: {}", objects_body);
        let schemas_body = body_for!(
            jetty_snowflake::Entry::Asset(jetty_snowflake::Asset::Schema(_)),
            input,
            name,
            database_name
        );
        debug!("body: {}", schemas_body);
        let databases_body = body_for!(
            jetty_snowflake::Entry::Asset(jetty_snowflake::Asset::Database(_)),
            input,
            name
        );

        // Mount mocks for each query.
        // Mount mock for roles
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW ROLES"))
            .respond_with(ResponseTemplate::new(200).set_body_string(roles_body))
            .named("roles query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // Mount mock for users
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW USERS"))
            .respond_with(ResponseTemplate::new(200).set_body_string(users_body))
            .named("users query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // Mount mock for grants
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW GRANTS"))
            .respond_with(ResponseTemplate::new(200).set_body_string(grants_body))
            .named("grants query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // Mount mock for tables
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW TABLES"))
            .respond_with(ResponseTemplate::new(200).set_body_string(objects_body.clone()))
            .named("grants query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // Mount mock for views
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW VIEWS"))
            .respond_with(ResponseTemplate::new(200).set_body_string(objects_body))
            .named("grants query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // Mount mock for schemas
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW SCHEMAS"))
            .respond_with(ResponseTemplate::new(200).set_body_string(schemas_body))
            .named("grants query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // Mount mock for databases
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW DATABASES"))
            .respond_with(ResponseTemplate::new(200).set_body_string(databases_body))
            .named("grants query")
            .mount(self.server.as_ref().unwrap())
            .await;
        self
    }
}

struct TestHarness<T: Connector> {
    _input: TestInput,
    _mock_server: WiremockServer,
    connector: Box<T>,
}

#[derive(Clone)]
struct TestInput {
    entries: Vec<jetty_snowflake::Entry>,
}

/// Get a mocked-out connector that will ingest the input.
async fn construct_connector_from(input: &TestInput) -> TestHarness<SnowflakeConnector> {
    let wiremock_server = WiremockServer::new().init(input).await;
    let creds = HashMap::from([
        ("account".to_owned(), "my_account".to_owned()),
        ("role".to_owned(), "role".to_owned()),
        ("user".to_owned(), "user".to_owned()),
        ("warehouse".to_owned(), "warehouse".to_owned()),
        ("private_key".to_owned(), "private_key".to_owned()),
        ("public_key_fp".to_owned(), "fp".to_owned()),
        (
            "url".to_owned(),
            format!(
                "{}/api/v2/statements",
                wiremock_server.server.as_ref().unwrap().uri()
            ),
        ),
    ]);
    TestHarness {
        _input: input.clone(),
        _mock_server: wiremock_server,
        connector: SnowflakeConnector::new(
            &ConnectorConfig::default(),
            &creds,
            Some(ConnectorClient::Test),
            None,
        )
        .await
        .unwrap(),
    }
}

#[tokio::test]
async fn input_produces_correct_results() {
    let expected_groups: Vec<RawGroup> = vec![RawGroup {
        name: "my_role".to_owned(),
        metadata: HashMap::new(),
        member_of: HashSet::new(),
        includes_users: HashSet::new(),
        includes_groups: HashSet::new(),
        granted_by: HashSet::new(),
    }];
    let input = TestInput {
        entries: vec![jetty_snowflake::Entry::Role(jetty_snowflake::Role {
            name: RoleName("my_role".to_owned()),
        })],
        // users: vec![jetty_snowflake::User {
        //     name: "my_user".to_owned(),
        //     first_name: "my".to_owned(),
        //     last_name: "user".to_owned(),
        //     email: "myuser@jettylabs.io".to_owned(),
        //     login_name: "myuser".to_owned(),
        //     display_name: "my user".to_owned(),
        // }],
        // grants: vec![jetty_snowflake::Grant {
        //     name: "my_grant".to_owned(),
        //     privilege: "my_priv".to_owned(),
        //     granted_on: "granted_on".to_owned(),
        // }],
        // assets: vec![
        //     Asset::Database(jetty_snowflake::Database {
        //         name: "db1".to_owned(),
        //     }),
        //     Asset::Schema(jetty_snowflake::Schema {
        //         name: "schema1".to_owned(),
        //         database_name: "db1".to_owned(),
        //     }),
        // ],
    };

    // Create the simulated client.
    let mut harness = construct_connector_from(&input).await;

    // Query the Snowflake connector
    let data: nodes::ConnectorData = harness.connector.get_data().await;
    debug!("data: {:#?}", data);

    // Do some assertion on the resulting data.
    assert_eq!(data.groups, expected_groups);
    // assert_eq!(data.users, EXPECTED_USERS);
    // assert_eq!(data.assets, EXPECTED_ASSETS);
    // assert_eq!(data.tags, EXPECTED_TAGS);
    // assert_eq!(data.policies, EXPECTED_POLICIES);
}
