// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use reqwest;
use std::env;
use std::error::Error;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let ui = AppWindow::new()?;
    let weakui = ui.as_weak();
    ui.set_currentMenu("startup".into());

    std::thread::spawn(move || loop {
        let now = chrono::Local::now()
            .format("%I:%M %p")
            .to_string()
            .trim_start_matches('0')
            .to_string();
        let cloned_ui = weakui.clone();

        slint::invoke_from_event_loop(move || {
            cloned_ui.unwrap().set_time(now.into());
        })
        .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
    });

    // let key = env::var("API_KEY").expect("API Key not there!").to_string();
    let account_email = env::var("ACCOUNT_EMAIL").expect("API Key not there!").to_string();
    let account_password = env::var("ACCOUNT_PASSWORD").expect("API Key not there!").to_string();
    let project_id = env::var("PROJECT_ID").expect("Project ID not there!").to_string();

    let reqwest_client = reqwest::Client::new();

    // Step 1: Create session and capture cookie
    let session_response = reqwest_client
        .post("https://fra.cloud.appwrite.io/v1/account/sessions/email")
        .header("Content-Type", "application/json")
        .header("X-Appwrite-Project", project_id.clone())
        .json(&serde_json::json!({
            "email": account_email,
            "password": account_password
        }))
        .send()
        .await?;

    let cookie = session_response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()?
        .to_string();

    let jwt_result = reqwest_client
        .post("https://fra.cloud.appwrite.io/v1/account/jwt")
        .header("Content-Type", "application/json")
        .header("X-Appwrite-Project", project_id.clone())
        .header("Cookie", cookie)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let jwt_string = jwt_result["jwt"].as_str().unwrap().to_string();

    let balance = reqwest_client
    .get("https://runshaw-api.danieldb.uk/api/name/get/")
    .header("Authorization", format!("{}", jwt_string))
    .send()
    .await?
    .json::<serde_json::Value>()
    .await?;

    println!("{:#?}", balance);
    
    ui.run()?;

    Ok(())
}
