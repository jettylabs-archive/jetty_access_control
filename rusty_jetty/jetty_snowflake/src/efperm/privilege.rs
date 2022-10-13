pub(crate) const TABLE_PRIVILEGES: &[&str] = &[
    "SELECT",     // Enables executing a SELECT statement on a table.
    "INSERT", // Enables executing an INSERT command on a table. Also enables using the ALTER TABLE command with a RECLUSTER clause to manually recluster a table with a clustering key.
    "UPDATE", // Enables executing an UPDATE command on a table.
    "TRUNCATE", // Enables executing a TRUNCATE TABLE command on a table.
    "DELETE", // Enables executing a DELETE command on a table.
    "REFERENCES", // Enables referencing a table as the unique/primary key table for a foreign key constraint. Also enables viewing the structure of a table (but not the data) via the DESCRIBE or SHOW command or by querying the Information Schema.
    "OWNERSHIP", // Grants full control over the table. Required to alter most properties of a table, with the exception of reclustering. Only a single role can hold this privilege on a specific object at a time. Note that in a managed access schema, only the schema owner (i.e. the role with the OWNERSHIP privilege on the schema) or a role with the MANAGE GRANTS privilege can grant or revoke privileges on objects in the schema, including future grants.
                 // All is materialized as every other privilege granted instead of being its own privilege.
                 // "ALL",       // Grants all privileges, except OWNERSHIP, on a table.
];

pub(crate) const VIEW_PRIVILEGES: &[&str] = &[
    "SELECT", // Enables executing a SELECT statement on a view. Note that this privilege is sufficient to query a view. The SELECT privilege on the underlying objects for a view is not required.
    "REFERENCES", //Enables viewing the structure of a view (but not the data) via the DESCRIBE or SHOW command or by querying the Information Schema.
    "OWNERSHIP", // Grants full control over the view. Required to alter a view. Only a single role can hold this privilege on a specific object at a time. Note that in a managed access schema, only the schema owner (i.e. the role with the OWNERSHIP privilege on the schema) or a role with the MANAGE GRANTS privilege can grant or revoke privileges on objects in the schema, including future grants.
                 // All is materialized as every other privilege granted instead of being its own privilege.
                 // "ALL",       // Grants all privileges, except OWNERSHIP, on a view.
];
