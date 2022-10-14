use std::collections::HashSet;

use anyhow::Result;
use serde::Deserialize;

use super::FlowDoc;

#[derive(Deserialize)]
struct ConnectionAttributes {
    schema: String,
    dbname: String,
}

fn get_server_info(doc: &FlowDoc, connection_id: &String) -> Result<String> {
    Ok(doc.connections[connection_id]
        .get("connectionAttributes")
        .and_then(|v| v.get("server"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_owned())
}

pub(super) fn get_input_table_ids(
    doc: &FlowDoc,
    node: &serde_json::Value,
) -> Result<HashSet<(TableauAssetType, String)>> {
    #[derive(Deserialize)]
    struct TableRelation {
        table: String,
    }

    #[derive(Deserialize)]
    struct TableInfo {
        #[serde(rename = "connectionAttributes")]
        connection_attributes: ConnectionAttributes,
        #[serde(rename = "connectionId")]
        connection_id: String,
        relation: TableRelation,
    }

    let table_info: TableInfo = serde_json::from_value(node.to_owned())?;
    let server = get_server_info(doc, &table_info.connection_id)?;

    let snowflake_table = crate::file_parse::snowflake_common::SnowflakeTableInfo {
        table: table_info.relation.table,
        db: table_info.connection_attributes.dbname,
        server,
        schema: table_info.connection_attributes.schema,
    };

    Ok(HashSet::from_iter(
        snowflake_table.to_cuals()?.iter().cloned(),
    ))
}

pub(super) fn get_output_table_ids(
    doc: &FlowDoc,
    node: &serde_json::Value,
) -> Result<HashSet<String>> {
    #[derive(Deserialize)]
    struct OutputDbAttributes {
        schema: String,
        dbname: String,
        warehouse: String,
        tablename: String,
    }

    #[derive(Deserialize)]
    struct TableInfo {
        attributes: OutputDbAttributes,
        #[serde(rename = "connectionId")]
        connection_id: String,
    }

    let table_info: TableInfo = serde_json::from_value(node.to_owned())?;
    let server = get_server_info(doc, &table_info.connection_id)?;

    let table = table_info.attributes.tablename;

    // It turns out that when writing tables to the db, flows actually use the tableau name as if it
    // were quoted, which is what we want to do, so this code double-escapes quotes causes errors:

    // // Fix up the table name:
    // if table.starts_with('"') {
    //     // this uses a simple wrapper around strip to make
    //     // it work like trim_matches, but only with single match
    //     table = strip_start_and_end(table, '"', '"');
    // } else if table.starts_with('\'') {
    //     table = strip_start_and_end(table, '\'', '\'');
    // } else if table.starts_with('[') {
    //     table = strip_start_and_end(table, '[', ']');
    // } else if table.starts_with('`') {
    //     table = strip_start_and_end(table, '`', '`');
    // } else {
    //     table = table.to_uppercase();
    // }

    let snowflake_table = crate::file_parse::snowflake_common::SnowflakeTableInfo {
        table,
        db: table_info.attributes.dbname,
        server,
        schema: table_info.attributes.schema,
    };

    Ok(HashSet::from_iter(
        snowflake_table.to_cuals()?.iter().cloned(),
    ))
}

pub(super) fn get_input_query_ids(
    doc: &FlowDoc,
    node: &serde_json::Value,
) -> Result<HashSet<String>> {
    #[derive(Deserialize)]
    struct QueryRelation {
        query: String,
    }

    #[derive(Deserialize)]
    struct QueryInfo {
        #[serde(rename = "connectionAttributes")]
        connection_attributes: ConnectionAttributes,
        #[serde(rename = "connectionId")]
        connection_id: String,
        relation: QueryRelation,
    }

    let mut relations = HashSet::new();

    let table_info: QueryInfo = serde_json::from_value(node.to_owned())?;
    let server = get_server_info(doc, &table_info.connection_id)?;

    let snowflake_table = crate::file_parse::snowflake_common::SnowflakeQueryInfo {
        query: table_info.relation.query,
        db: table_info.connection_attributes.dbname,
        server,
        schema: table_info.connection_attributes.schema,
    };
    relations.extend(snowflake_table.to_cuals()?);
    Ok(relations)
}
