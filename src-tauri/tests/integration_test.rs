//! Integration tests for crabase backend against a real Docker Postgres.
//! Requires `just test-setup` to have been run first.
//! Connection: postgresql://test:test@localhost:5433/crabase_test

use crabase::db::{
    ChangeSet, ConnectionInfo, DbState, Filter, RowDelete, RowInsert, RowUpdate, SortCol,
};

const TEST_CONN: &str = "postgresql://test:test@localhost:5433/crabase_test";

/// Helper: create a connected DbState for tests.
async fn connected_state() -> DbState {
    let state = DbState::new();
    let info = ConnectionInfo {
        host: "localhost".to_string(),
        port: 5433,
        user: "test".to_string(),
        dbname: "crabase_test".to_string(),
        schema: "public".to_string(),
        sslmode: "disable".to_string(),
        password: "test".to_string(),
    };
    state.connect(info).await.expect("Failed to connect to test DB");
    state
}

/// Helper: create a DbState connected to a specific schema.
async fn connected_state_with_schema(schema: &str) -> DbState {
    let state = DbState::new();
    let info = ConnectionInfo {
        host: "localhost".to_string(),
        port: 5433,
        user: "test".to_string(),
        dbname: "crabase_test".to_string(),
        schema: schema.to_string(),
        sslmode: "disable".to_string(),
        password: "test".to_string(),
    };
    state.connect(info).await.expect("Failed to connect to test DB");
    state
}

// === Phase 30 Tests ===

#[tokio::test]
async fn test_connect_db_valid() {
    let state = DbState::new();
    let info = ConnectionInfo {
        host: "localhost".to_string(),
        port: 5433,
        user: "test".to_string(),
        dbname: "crabase_test".to_string(),
        schema: "public".to_string(),
        sslmode: "disable".to_string(),
        password: "test".to_string(),
    };
    let result = state.connect(info).await;
    assert!(result.is_ok(), "connect_db should succeed with valid credentials");
}

#[tokio::test]
async fn test_connect_db_invalid() {
    let state = DbState::new();
    let info = ConnectionInfo {
        host: "localhost".to_string(),
        port: 5433,
        user: "wrong_user".to_string(),
        dbname: "crabase_test".to_string(),
        schema: "public".to_string(),
        sslmode: "disable".to_string(),
        password: "wrong_pass".to_string(),
    };
    let result = state.connect(info).await;
    assert!(result.is_err(), "connect_db should fail with invalid credentials");
}

#[tokio::test]
async fn test_disconnect_db() {
    let state = connected_state().await;
    let result = state.disconnect();
    assert!(result.is_ok(), "disconnect_db should succeed");
    // After disconnect, list_tables should fail
    let tables = state.list_tables().await;
    assert!(tables.is_err(), "list_tables should fail after disconnect");
}

#[tokio::test]
async fn test_get_connection_info() {
    let state = connected_state().await;
    let info = state.get_connection_info().unwrap();
    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 5433);
    assert_eq!(info.user, "test");
    assert_eq!(info.dbname, "crabase_test");
}

#[tokio::test]
async fn test_list_schemas() {
    let schemas = crabase::db::list_schemas(TEST_CONN).await.unwrap();
    assert!(schemas.contains(&"public".to_string()), "Should contain 'public' schema");
    assert!(schemas.contains(&"test_schema".to_string()), "Should contain 'test_schema'");
}

#[tokio::test]
async fn test_list_tables() {
    let state = connected_state().await;
    let tables = state.list_tables().await.unwrap();
    assert!(tables.contains(&"users".to_string()), "Should contain 'users' table");
    assert!(tables.contains(&"products".to_string()), "Should contain 'products' table");
    assert!(tables.contains(&"events".to_string()), "Should contain 'events' table");
}

#[tokio::test]
async fn test_get_column_info_users() {
    let state = connected_state().await;
    let columns = state.get_column_info("users").await.unwrap();

    // Check id column
    let id_col = columns.iter().find(|c| c.name == "id").expect("Should have 'id' column");
    assert!(id_col.is_primary_key);
    assert!(id_col.is_auto_increment);
    assert!(!id_col.is_nullable);

    // Check role column (enum)
    let role_col = columns.iter().find(|c| c.name == "role").expect("Should have 'role' column");
    assert!(role_col.is_enum);
    assert!(role_col.enum_values.contains(&"admin".to_string()));
    assert!(role_col.enum_values.contains(&"editor".to_string()));
    assert!(role_col.enum_values.contains(&"viewer".to_string()));
    assert!(role_col.enum_values.contains(&"guest".to_string()));

    // Check tags column (array)
    let tags_col = columns.iter().find(|c| c.name == "tags").expect("Should have 'tags' column");
    assert!(tags_col.is_array);

    // Check nullable column
    let age_col = columns.iter().find(|c| c.name == "age").expect("Should have 'age' column");
    assert!(age_col.is_nullable);

    // Check metadata column (jsonb)
    let meta_col = columns.iter().find(|c| c.name == "metadata").expect("Should have 'metadata' column");
    assert_eq!(meta_col.data_type, "jsonb");
}

#[tokio::test]
async fn test_get_table_data_basic() {
    let state = connected_state().await;
    let data = state.get_table_data("users", 1, 10).await.unwrap();
    assert_eq!(data.total_count, 12);
    assert_eq!(data.rows.len(), 10); // page_size = 10, so first page has 10 rows
    assert!(!data.columns.is_empty());
}

#[tokio::test]
async fn test_get_table_data_pagination() {
    let state = connected_state().await;
    let page1 = state.get_table_data("users", 1, 5).await.unwrap();
    let page2 = state.get_table_data("users", 2, 5).await.unwrap();

    assert_eq!(page1.rows.len(), 5);
    assert_eq!(page2.rows.len(), 5);
    assert_eq!(page1.total_count, 12);
    assert_eq!(page2.total_count, 12);
    // Pages should have different data
    assert_ne!(page1.rows[0], page2.rows[0]);
}

#[tokio::test]
async fn test_get_table_data_tagged_values() {
    let state = connected_state().await;
    let data = state.get_table_data("users", 1, 1).await.unwrap();
    let row = &data.rows[0];

    // Check that values are tagged with their type
    // Find the username column index
    let username_idx = data.columns.iter().position(|c| c.name == "username").unwrap();
    let username_val = &row[username_idx];
    // Tagged values have { "type": "...", "value": ... }
    assert!(username_val.get("type").is_some(), "Values should be tagged with type");
    assert!(username_val.get("value").is_some(), "Values should have a value field");
}

#[tokio::test]
async fn test_get_table_data_null_handling() {
    let state = connected_state().await;
    // Eve (row 5) has NULL age, score, rating, balance, tags, metadata, preferences, bio
    let data = state.get_table_data("users", 1, 12).await.unwrap();

    // Find Eve's row (username = "eve")
    let username_idx = data.columns.iter().position(|c| c.name == "username").unwrap();
    let age_idx = data.columns.iter().position(|c| c.name == "age").unwrap();

    let eve_row = data.rows.iter().find(|r| {
        r[username_idx].get("value").and_then(|v| v.as_str()) == Some("eve")
    }).expect("Should find Eve's row");

    // NULL values should be serde_json::Value::Null (untagged)
    assert!(eve_row[age_idx].is_null(), "NULL values should be null");
}

#[tokio::test]
async fn test_get_table_data_timestamp_format() {
    let state = connected_state().await;
    let data = state.get_table_data("users", 1, 1).await.unwrap();
    let row = &data.rows[0];

    let created_at_idx = data.columns.iter().position(|c| c.name == "created_at").unwrap();
    let created_at_val = &row[created_at_idx];

    // Should be a proper ISO-like string, not "Timestamp" literal
    let val_str = created_at_val.get("value").and_then(|v| v.as_str()).unwrap_or("");
    assert!(val_str.contains("2024"), "Timestamp should contain year, got: {}", val_str);
    assert!(!val_str.contains("Timestamp"), "Should not be literal 'Timestamp'");
}

#[tokio::test]
async fn test_get_table_data_filtered_equals() {
    let state = connected_state().await;
    let filters = vec![Filter {
        column: "role".to_string(),
        operator: "=".to_string(),
        value: "admin".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // alice, diana, heidi are admins
    assert_eq!(data.total_count, 3);
}

#[tokio::test]
async fn test_get_table_data_filtered_not_equals() {
    let state = connected_state().await;
    let filters = vec![Filter {
        column: "is_active".to_string(),
        operator: "=".to_string(),
        value: "false".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // charlie and ivan are inactive
    assert_eq!(data.total_count, 2);
}

#[tokio::test]
async fn test_get_table_data_filtered_is_null() {
    let state = connected_state().await;
    let filters = vec![Filter {
        column: "age".to_string(),
        operator: "IS NULL".to_string(),
        value: "".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // Eve has NULL age
    assert_eq!(data.total_count, 1);
}

#[tokio::test]
async fn test_get_table_data_filtered_is_not_null() {
    let state = connected_state().await;
    let filters = vec![Filter {
        column: "age".to_string(),
        operator: "IS NOT NULL".to_string(),
        value: "".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // 11 users have non-null age
    assert_eq!(data.total_count, 11);
}

#[tokio::test]
async fn test_get_table_data_filtered_like() {
    let state = connected_state().await;
    let filters = vec![Filter {
        column: "email".to_string(),
        operator: "LIKE".to_string(),
        value: "%a%".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // All emails contain 'a' (alice, charlie, diana, frank, grace, heidi, ivan, judy, karl, lara) = many
    assert!(data.total_count > 0);
}

#[tokio::test]
async fn test_get_table_data_filtered_and_combinator() {
    let state = connected_state().await;
    let filters = vec![
        Filter {
            column: "role".to_string(),
            operator: "=".to_string(),
            value: "admin".to_string(),
            combinator: "AND".to_string(),
        },
        Filter {
            column: "is_active".to_string(),
            operator: "=".to_string(),
            value: "true".to_string(),
            combinator: "AND".to_string(),
        },
    ];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // alice, diana, heidi are active admins
    assert_eq!(data.total_count, 3);
}

#[tokio::test]
async fn test_get_table_data_filtered_or_combinator() {
    let state = connected_state().await;
    let filters = vec![
        Filter {
            column: "role".to_string(),
            operator: "=".to_string(),
            value: "admin".to_string(),
            combinator: "AND".to_string(),
        },
        Filter {
            column: "role".to_string(),
            operator: "=".to_string(),
            value: "guest".to_string(),
            combinator: "OR".to_string(),
        },
    ];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    // admins (3) + guests (2) = 5
    assert_eq!(data.total_count, 5);
}

#[tokio::test]
async fn test_get_table_data_filtered_sort_asc() {
    let state = connected_state().await;
    let sort = vec![SortCol {
        column: "username".to_string(),
        direction: "asc".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 12, vec![], sort)
        .await
        .unwrap();
    let username_idx = data.columns.iter().position(|c| c.name == "username").unwrap();
    let first = data.rows[0][username_idx].get("value").and_then(|v| v.as_str()).unwrap();
    let last = data.rows[11][username_idx].get("value").and_then(|v| v.as_str()).unwrap();
    assert_eq!(first, "alice");
    assert_eq!(last, "lara");
}

#[tokio::test]
async fn test_get_table_data_filtered_sort_desc() {
    let state = connected_state().await;
    let sort = vec![SortCol {
        column: "username".to_string(),
        direction: "desc".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 12, vec![], sort)
        .await
        .unwrap();
    let username_idx = data.columns.iter().position(|c| c.name == "username").unwrap();
    let first = data.rows[0][username_idx].get("value").and_then(|v| v.as_str()).unwrap();
    assert_eq!(first, "lara");
}

#[tokio::test]
async fn test_save_changes_update() {
    let state = connected_state().await;

    // Update alice's bio
    let changes = ChangeSet {
        updates: vec![RowUpdate {
            pk_values: [("id".to_string(), serde_json::json!(1))].into(),
            changes: [("bio".to_string(), serde_json::json!("Updated bio"))].into(),
        }],
        inserts: vec![],
        deletes: vec![],
    };
    state.save_changes("users", changes).await.unwrap();

    // Verify
    let data = state.get_table_data("users", 1, 12).await.unwrap();
    let username_idx = data.columns.iter().position(|c| c.name == "username").unwrap();
    let bio_idx = data.columns.iter().position(|c| c.name == "bio").unwrap();
    let alice_row = data.rows.iter().find(|r| {
        r[username_idx].get("value").and_then(|v| v.as_str()) == Some("alice")
    }).unwrap();
    let bio = alice_row[bio_idx].get("value").and_then(|v| v.as_str()).unwrap();
    assert_eq!(bio, "Updated bio");

    // Restore original
    let restore = ChangeSet {
        updates: vec![RowUpdate {
            pk_values: [("id".to_string(), serde_json::json!(1))].into(),
            changes: [("bio".to_string(), serde_json::json!("Software engineer"))].into(),
        }],
        inserts: vec![],
        deletes: vec![],
    };
    state.save_changes("users", restore).await.unwrap();
}

#[tokio::test]
async fn test_save_changes_insert() {
    let state = connected_state().await;

    let changes = ChangeSet {
        updates: vec![],
        inserts: vec![RowInsert {
            values: [
                ("username".to_string(), serde_json::json!("test_insert_user")),
                ("email".to_string(), serde_json::json!("test@insert.com")),
                ("role".to_string(), serde_json::json!("viewer")),
                ("is_active".to_string(), serde_json::json!(true)),
            ]
            .into(),
        }],
        deletes: vec![],
    };
    state.save_changes("users", changes).await.unwrap();

    // Verify it exists
    let filters = vec![Filter {
        column: "username".to_string(),
        operator: "=".to_string(),
        value: "test_insert_user".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    assert_eq!(data.total_count, 1);

    // Clean up: delete the inserted row
    let id_idx = data.columns.iter().position(|c| c.name == "id").unwrap();
    let id_val = data.rows[0][id_idx].get("value").unwrap().clone();
    let cleanup = ChangeSet {
        updates: vec![],
        inserts: vec![],
        deletes: vec![RowDelete {
            pk_values: [("id".to_string(), id_val)].into(),
        }],
    };
    state.save_changes("users", cleanup).await.unwrap();
}

#[tokio::test]
async fn test_save_changes_delete() {
    let state = connected_state().await;

    // First insert a row to delete
    let changes = ChangeSet {
        updates: vec![],
        inserts: vec![RowInsert {
            values: [
                ("username".to_string(), serde_json::json!("to_delete")),
                ("email".to_string(), serde_json::json!("delete@test.com")),
                ("role".to_string(), serde_json::json!("guest")),
                ("is_active".to_string(), serde_json::json!(false)),
            ]
            .into(),
        }],
        deletes: vec![],
    };
    state.save_changes("users", changes).await.unwrap();

    // Find its ID
    let filters = vec![Filter {
        column: "username".to_string(),
        operator: "=".to_string(),
        value: "to_delete".to_string(),
        combinator: "AND".to_string(),
    }];
    let data = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    let id_idx = data.columns.iter().position(|c| c.name == "id").unwrap();
    let id_val = data.rows[0][id_idx].get("value").unwrap().clone();

    // Delete it
    let delete_changes = ChangeSet {
        updates: vec![],
        inserts: vec![],
        deletes: vec![RowDelete {
            pk_values: [("id".to_string(), id_val)].into(),
        }],
    };
    state.save_changes("users", delete_changes).await.unwrap();

    // Verify it's gone
    let filters2 = vec![Filter {
        column: "username".to_string(),
        operator: "=".to_string(),
        value: "to_delete".to_string(),
        combinator: "AND".to_string(),
    }];
    let data2 = state
        .get_table_data_filtered("users", 1, 50, filters2, vec![])
        .await
        .unwrap();
    assert_eq!(data2.total_count, 0);
}

#[tokio::test]
async fn test_save_changes_mixed() {
    let state = connected_state().await;

    // Insert a row, update another, in one changeset
    let changes = ChangeSet {
        updates: vec![RowUpdate {
            pk_values: [("id".to_string(), serde_json::json!(2))].into(),
            changes: [("bio".to_string(), serde_json::json!("Mixed test update"))].into(),
        }],
        inserts: vec![RowInsert {
            values: [
                ("username".to_string(), serde_json::json!("mixed_test")),
                ("email".to_string(), serde_json::json!("mixed@test.com")),
                ("role".to_string(), serde_json::json!("viewer")),
                ("is_active".to_string(), serde_json::json!(true)),
            ]
            .into(),
        }],
        deletes: vec![],
    };
    state.save_changes("users", changes).await.unwrap();

    // Verify update
    let data = state.get_table_data("users", 1, 12).await.unwrap();
    let username_idx = data.columns.iter().position(|c| c.name == "username").unwrap();
    let bio_idx = data.columns.iter().position(|c| c.name == "bio").unwrap();
    let bob_row = data.rows.iter().find(|r| {
        r[username_idx].get("value").and_then(|v| v.as_str()) == Some("bob")
    }).unwrap();
    assert_eq!(bob_row[bio_idx].get("value").and_then(|v| v.as_str()).unwrap(), "Mixed test update");

    // Clean up: restore bob's bio and delete the inserted row
    let filters = vec![Filter {
        column: "username".to_string(),
        operator: "=".to_string(),
        value: "mixed_test".to_string(),
        combinator: "AND".to_string(),
    }];
    let inserted = state
        .get_table_data_filtered("users", 1, 50, filters, vec![])
        .await
        .unwrap();
    let id_idx = inserted.columns.iter().position(|c| c.name == "id").unwrap();
    let id_val = inserted.rows[0][id_idx].get("value").unwrap().clone();

    let cleanup = ChangeSet {
        updates: vec![RowUpdate {
            pk_values: [("id".to_string(), serde_json::json!(2))].into(),
            changes: [("bio".to_string(), serde_json::json!("Content writer"))].into(),
        }],
        inserts: vec![],
        deletes: vec![RowDelete {
            pk_values: [("id".to_string(), id_val)].into(),
        }],
    };
    state.save_changes("users", cleanup).await.unwrap();
}

#[tokio::test]
async fn test_execute_query_select() {
    let state = connected_state().await;
    let result = state
        .execute_query("SELECT username, email FROM users WHERE role = 'admin' ORDER BY username")
        .await
        .unwrap();
    assert_eq!(result.columns, vec!["username", "email"]);
    assert_eq!(result.rows.len(), 3); // alice, diana, heidi
}

#[tokio::test]
async fn test_execute_query_multi_success() {
    let state = connected_state().await;
    let results = state
        .execute_query_multi("SELECT 1 as num; SELECT 'hello' as greeting;")
        .await
        .unwrap();
    assert_eq!(results.len(), 2);

    // First result
    match &results[0] {
        crabase::db::StatementResult::Rows { columns, rows, .. } => {
            assert_eq!(columns, &vec!["num".to_string()]);
            assert_eq!(rows.len(), 1);
        }
        _ => panic!("Expected Rows result"),
    }

    // Second result
    match &results[1] {
        crabase::db::StatementResult::Rows { columns, rows, .. } => {
            assert_eq!(columns, &vec!["greeting".to_string()]);
            assert_eq!(rows.len(), 1);
        }
        _ => panic!("Expected Rows result"),
    }
}

#[tokio::test]
async fn test_execute_query_multi_with_error() {
    let state = connected_state().await;
    let results = state
        .execute_query_multi("SELECT 1; SELECT * FROM nonexistent_table_xyz;")
        .await
        .unwrap();
    assert_eq!(results.len(), 2);

    match &results[0] {
        crabase::db::StatementResult::Rows { .. } => {}
        _ => panic!("First statement should succeed"),
    }

    match &results[1] {
        crabase::db::StatementResult::Error { message, .. } => {
            assert!(message.contains("nonexistent_table_xyz") || message.contains("does not exist"));
        }
        _ => panic!("Second statement should be an error"),
    }
}

#[tokio::test]
async fn test_drop_table() {
    let state = connected_state().await;

    // Create a temp table to drop
    state
        .execute_query("CREATE TABLE drop_test (id SERIAL PRIMARY KEY, name TEXT)")
        .await
        .unwrap();

    // Verify it exists
    let tables = state.list_tables().await.unwrap();
    assert!(tables.contains(&"drop_test".to_string()));

    // Drop it
    state.drop_table("drop_test").await.unwrap();

    // Verify it's gone
    let tables2 = state.list_tables().await.unwrap();
    assert!(!tables2.contains(&"drop_test".to_string()));
}

#[tokio::test]
async fn test_truncate_table() {
    let state = connected_state().await;

    // Create a temp table with data
    state
        .execute_query("CREATE TABLE IF NOT EXISTS truncate_test (id SERIAL PRIMARY KEY, name TEXT)")
        .await
        .unwrap();
    state
        .execute_query("INSERT INTO truncate_test (name) VALUES ('a'), ('b'), ('c')")
        .await
        .unwrap();

    // Truncate it
    state.truncate_table("truncate_test").await.unwrap();

    // Verify it's empty
    let data = state.get_table_data("truncate_test", 1, 50).await.unwrap();
    assert_eq!(data.total_count, 0);

    // Clean up
    state.drop_table("truncate_test").await.unwrap();
}

#[tokio::test]
async fn test_export_table_json() {
    let state = connected_state().await;
    let json_str = state.export_table_json("users").await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 12);
}

#[tokio::test]
async fn test_export_table_sql() {
    let state = connected_state().await;
    let sql = state.export_table_sql("users").await.unwrap();
    assert!(sql.contains("INSERT INTO"), "Should contain INSERT statements");
    assert!(sql.contains("alice"), "Should contain data from the table");
}

#[tokio::test]
async fn test_get_columns_for_autocomplete() {
    let state = connected_state().await;
    let tables = vec!["users".to_string(), "products".to_string()];
    let result = state.get_columns_for_autocomplete(&tables).await.unwrap();
    assert!(result.contains_key("users"));
    assert!(result.contains_key("products"));
    let user_cols = result.get("users").unwrap();
    assert!(user_cols.contains(&"username".to_string()));
    assert!(user_cols.contains(&"email".to_string()));
}

#[tokio::test]
async fn test_get_full_schema_text() {
    let state = connected_state().await;
    let schema_text = state.get_full_schema_text().await.unwrap();
    assert!(!schema_text.is_empty());
    assert!(schema_text.contains("users"), "Should contain table names");
    assert!(schema_text.contains("username"), "Should contain column names");
}

#[tokio::test]
async fn test_enum_on_non_public_schema() {
    let state = connected_state_with_schema("test_schema").await;
    let columns = state.get_column_info("tasks").await.unwrap();

    let status_col = columns.iter().find(|c| c.name == "status").expect("Should have 'status' column");
    assert!(status_col.is_enum, "status should be an enum");
    assert!(status_col.enum_values.contains(&"pending".to_string()));
    assert!(status_col.enum_values.contains(&"active".to_string()));
    assert!(status_col.enum_values.contains(&"completed".to_string()));
    assert!(status_col.enum_values.contains(&"cancelled".to_string()));
}
