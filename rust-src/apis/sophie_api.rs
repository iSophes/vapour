use appwrite::models::Row;
use appwrite::services::tables_db::TablesDB;
use appwrite::{self, AppwriteError};
use serde_json::json;

pub async fn does_user_exist(
    appwrite_client: &appwrite::client::Client,
    student_id: &str, // Changed to &str for flexibility
) -> Result<bool, Box<dyn std::error::Error>> {
    let tables_db = TablesDB::new(appwrite_client);

    match tables_db
        .get_row("users", "users", student_id, None, None)
        .await
    {
        Ok(_) => Ok(true),
        Err(e) if e.get_response().contains("\"code\":404") => Ok(false),
        Err(e) => Err(e.into()),
    }
}

pub async fn create_user(
    appwrite_client: &appwrite::client::Client,
    student_id: &str,
) -> Result<Row, Box<dyn std::error::Error>> {
    let data = json!({ "balance": "£0.00" });
    let tables_db = TablesDB::new(appwrite_client);

    // Create the row
    tables_db
        .create_row("users", "users", student_id, data, None, None)
        .await?;

    // Fetch the newly created row safely
    let row = tables_db
        .get_row("users", "users", student_id, None, None)
        .await?;

    Ok(row)
}

pub async fn get_user_balance(
    appwrite_client: &appwrite::client::Client,
    student_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tables_db = TablesDB::new(appwrite_client);

    // Replace .unwrap() with ? to return errors instead of freezing
    let row = tables_db
        .get_row("users", "users", student_id, None, None)
        .await?;

    let balance = row
        .data
        .get("balance")
        .and_then(|v| v.as_str())
        .ok_or("Balance field missing or invalid format")?;

    Ok(balance.to_owned())
}

pub async fn set_balance(
    appwrite_client: &appwrite::client::Client,
    student_id: &str,
    new_balance: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tables_db = TablesDB::new(appwrite_client);
    let new_data = json!({ "balance": new_balance });

    // Replace .expect() with ?
    tables_db
        .update_row("users", "users", student_id, Some(new_data), None, None)
        .await?;

    Ok(())
}
