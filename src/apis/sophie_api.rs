// My own appwrite database APIs. Replace with your own.

use appwrite::models::Row;
use appwrite::services::tables_db::TablesDB;
use appwrite::{self, AppwriteError};
use serde_json::json;

pub async fn does_user_exist(
    appwrite_client: &appwrite::client::Client,
    student_id: &String,
) -> Result<bool, reqwest::Error> {
    // check if user exists.

    let tables_db = TablesDB::new(appwrite_client);
    let data = tables_db
        .get_row("users", "users", student_id, None, None)
        .await;

    if data.is_err() {
        let error_message = data.err().unwrap();
        let error_code = error_message.get_response();

        if error_code.contains("\"code\":404") {
            return Ok(false); // not existing
        }

        // show an error on screen asking the user to retry.

        return Ok(false);
    }

    return Ok(true);
}

pub async fn create_user(
    appwrite_client: &appwrite::client::Client,
    student_id: &String,
) -> Result<Result<Row, AppwriteError>, Box<dyn std::error::Error>> {
    let data = json!({
        "balance": "£0.00"
    });

    let tables_db = TablesDB::new(appwrite_client);
    tables_db
        .create_row("users", "users", student_id, data, None, None)
        .await?;

    let row = tables_db
        .get_row("users", "users", student_id, None, None)
        .await;

    return Ok(row);
}

pub async fn get_user_balance(
    appwrite_client: &appwrite::client::Client,
    student_id: &String,
) -> String {
    // we know they exist with this
    let tables_db = TablesDB::new(appwrite_client);
    let row = tables_db
        .get_row("users", "users", student_id, None, None)
        .await;

    let unwrapped = row.unwrap();
    let balance = unwrapped.data.get("balance").unwrap().as_str().unwrap();

    return balance.to_owned();
}

pub async fn set_balance(
    appwrite_client: &appwrite::client::Client,
    student_id: &String,
    new_balance: &String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let tables_db = TablesDB::new(appwrite_client);
    let new_data = json!({
        "balance": new_balance
    });

    tables_db
        .update_row("users", "users", student_id, Some(new_data), None, None)
        .await
        .expect("FAILED TO TOPUP.");
    return Ok(true);
}
