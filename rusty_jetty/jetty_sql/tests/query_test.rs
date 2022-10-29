use anyhow::Result;
use jetty_core::logging::{debug, error};

// Tests that queries from the dataset are valid. Does not currently check table output.
#[test]
fn try_all_queries() -> Result<()> {
    let mut parse_failures = 0;
    let mut table_failures = 0;
    for i in 1..100 {
        print!("Query: #{i} - ");
        let q = std::fs::read_to_string(format!("tests/queries/query_{i}.sql"));
        match q {
            Ok(query) => {
                let tables = jetty_sql::get_tables(&query, jetty_sql::DbType::Generic);
                match tables {
                    Ok(t) => debug!(
                        "{}",
                        t.iter().map(|u| u.join(".")).collect::<Vec<_>>().join(", ")
                    ),
                    Err(e) => {
                        if e.to_string().contains("sql parser error") {
                            error!("Failed to parse query: {}", e);
                            parse_failures += 1;
                        } else {
                            error!("Failed to get tables: {}", e);
                            table_failures += 1;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read query: {}", e);
            }
        }
    }
    error!(
        "\n--------------\nFailed to parse {} queries",
        parse_failures
    );
    error!(
        "Failed to extract tables from {} queries\n--------------\n",
        table_failures
    );
    Ok(())
}
