use std::boxed::Box;
use std::collections::HashMap;

use jetty_core::{
    connectors::{nodes, ConnectorClient},
    jetty::{ConnectorConfig, CredentialsBlob},
    Connector,
};
use jetty_snowflake::{Asset, Snowflake};

use serde::Serialize;
use serde_json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockGuard, MockServer, ResponseTemplate};

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
    #[serde(rename = "resultSetMetadata")]
    result_set_metadata: SnowflakeRowTypeFields,
    data: Vec<Vec<String>>,
}

impl WiremockServer {
    pub fn new() -> Self {
        Self { server: None }
    }

    async fn init(mut self, input: &TestInput) -> Self {
        let mock_server = MockServer::start().await;
        self.server = Some(mock_server);

        let body_obj = SnowflakeResult {
            result_set_metadata: SnowflakeRowTypeFields {
                row_type: vec![
                    SnowflakeField {
                        name: "Field 1".to_owned(),
                    },
                    SnowflakeField {
                        name: "Field 2".to_owned(),
                    },
                ],
            },
            data: vec![vec!["one".to_owned(), "two".to_owned()]],
        };

        let body = serde_json::to_string(&body_obj).unwrap();
        println!("body: {}", body);

        // Mount mocks for each query.
        // Mount mock for roles
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .and(body_string_contains("SHOW ROLES"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .named("roles query")
            .mount(self.server.as_ref().unwrap())
            .await;

        // // Mount mock for users
        // Mock::given(method("POST"))
        //     .and(path("/api/v2/statements"))
        //     .and(body_string_contains("SHOW USERS"))
        //     .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"text": "wiremock"}"#))
        //     .named("users query")
        //     .mount(self.server.as_ref().unwrap())
        //     .await;
        self
    }
}

struct TestHarness<T: Connector> {
    input: TestInput,
    mock_server: WiremockServer,
    connector: Box<T>,
}

#[derive(Clone)]
struct TestInput {
    roles: Vec<jetty_snowflake::Role>,
    users: Vec<jetty_snowflake::User>,
    grants: Vec<jetty_snowflake::Grant>,
    assets: Vec<jetty_snowflake::Asset>,
}

/// Get a mocked-out connector that will ingest the input.
async fn construct_connector_from(input: &TestInput) -> TestHarness<Snowflake> {
    let wiremock_server = WiremockServer::new().init(&input).await;
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
        input: input.clone(),
        mock_server: wiremock_server,
        connector: Snowflake::new(
            &ConnectorConfig::default(),
            &creds,
            Some(ConnectorClient::Test),
        )
        .unwrap(),
    }
}

// #[ignore]
#[tokio::test]
async fn input_produces_correct_results() {
    let input = TestInput {
        roles: vec![jetty_snowflake::Role {
            name: "my_role".to_owned(),
        }],
        users: vec![jetty_snowflake::User {
            name: "my_user".to_owned(),
            first_name: "my".to_owned(),
            last_name: "user".to_owned(),
            email: "myuser@jettylabs.io".to_owned(),
            login_name: "myuser".to_owned(),
            display_name: "my user".to_owned(),
        }],
        grants: vec![jetty_snowflake::Grant {
            name: "my_grant".to_owned(),
            privilege: "my_priv".to_owned(),
            granted_on: "granted_on".to_owned(),
        }],
        assets: vec![
            Asset::Database(jetty_snowflake::Database {
                name: "db1".to_owned(),
            }),
            Asset::Schema(jetty_snowflake::Schema {
                name: "schema1".to_owned(),
                database_name: "db1".to_owned(),
            }),
        ],
    };

    // Create the simulated client.
    let harness = construct_connector_from(&input).await;

    // Query the Snowflake connector
    let data: nodes::ConnectorData = harness.connector.get_data().await;
    println!("data: {:#?}", data);

    // Do some assertion on the resulting data.
    // assert_eq!(data.groups, EXPECTED_GROUPS);
    // assert_eq!(data.users, EXPECTED_USERS);
    // assert_eq!(data.assets, EXPECTED_ASSETS);
    // assert_eq!(data.tags, EXPECTED_TAGS);
    // assert_eq!(data.policies, EXPECTED_POLICIES);
}
