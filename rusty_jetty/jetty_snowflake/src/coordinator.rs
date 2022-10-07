use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use futures::future::join_all;
use futures::future::BoxFuture;
use futures::StreamExt;

use crate::entry_types;

/// Number of metadata request to run currently (e.g. permissions).
/// 15 seems to give the best performance. In some circumstances, we may want to bump this up.
const CONCURRENT_METADATA_FETCHES: usize = 15;

/// Environment is a collection of objects pulled right out of Snowflake.
/// We process them to make jetty nodes and edges.
#[derive(Default, Debug)]
struct Environment {
    databases: Vec<entry_types::Database>,
    schemas: Vec<entry_types::Schema>,
    objects: Vec<entry_types::Object>,
    users: Vec<entry_types::User>,
    roles: Vec<entry_types::Role>,
    standard_grants: Vec<entry_types::StandardGrant>,
    future_grants: Vec<entry_types::FutureGrant>,
    role_grants: Vec<entry_types::GrantOf>,
}

// Now lets start filling up the environment

pub(super) struct Coordinator<'a> {
    env: Environment,
    conn: &'a super::SnowflakeConnector,
}

impl<'a> Coordinator<'a> {
    pub(super) fn new(conn: &'a super::SnowflakeConnector) -> Self {
        Self {
            env: Default::default(),
            conn,
        }
    }

    pub(super) async fn get_data(&mut self) {
        let now = Instant::now();
        // // Run in one group
        // Get all databases
        // Get all the schemas
        // Get all the users
        // Get all the roles

        let hold: Vec<BoxFuture<_>> = vec![
            Box::pin(self.conn.get_databases_future(&mut self.env.databases)),
            Box::pin(self.conn.get_schemas_future(&mut self.env.schemas)),
            Box::pin(self.conn.get_users_future(&mut self.env.users)),
            Box::pin(self.conn.get_roles_future(&mut self.env.roles)),
        ];

        let results = join_all(hold).await;

        println!("Fetched first batch: {:?}", now.elapsed());

        for res in results {
            match res {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }
        }

        // try one object:
        let mut hold: Vec<BoxFuture<_>> = vec![];

        // for each schema, get objects
        let objects_mutex = Arc::new(Mutex::new(&mut self.env.objects));
        for schema in &self.env.schemas {
            let m = Arc::clone(&objects_mutex);
            hold.push(Box::pin(self.conn.get_objects_futures(schema, m)));
        }

        // for each role, get grants to that role
        let grants_to_role_mutex = Arc::new(Mutex::new(&mut self.env.standard_grants));
        for role in &self.env.roles {
            let m = Arc::clone(&grants_to_role_mutex);
            hold.push(Box::pin(self.conn.get_grants_to_role_future(role, m)));
        }

        // for each role, get grants of
        let target_arc = Arc::new(Mutex::new(&mut self.env.role_grants));
        for role in &self.env.roles {
            let m = Arc::clone(&target_arc);
            hold.push(Box::pin(self.conn.get_grants_of_role_future(role, m)));
        }

        // for each schema, get future grants
        let futre_grants_arc = Arc::new(Mutex::new(&mut self.env.future_grants));
        for schema in &self.env.schemas {
            let m = Arc::clone(&futre_grants_arc);
            hold.push(Box::pin(
                self.conn.get_future_grants_of_schema_future(schema, m),
            ));
        }

        // for database, get future grants, using the same Arc<Mutex>
        for database in &self.env.databases {
            let m = Arc::clone(&futre_grants_arc);
            hold.push(Box::pin(
                self.conn.get_future_grants_of_database_future(database, m),
            ));
        }

        let t2 = now.elapsed();
        println!("Starting second batch: {:?}", now.elapsed());
        let object_fetch_results = futures::stream::iter(hold)
            .buffer_unordered(CONCURRENT_METADATA_FETCHES)
            .collect::<Vec<_>>()
            .await;

        println!(
            "Finished second batch: {:?} ({:?})",
            now.elapsed(),
            now.elapsed() - t2
        );
        for res in object_fetch_results {
            match res {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }
        }
    }
}
