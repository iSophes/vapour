// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use reqwest;
use std::env;
use std::error::Error;

mod qrscan;

slint::include_modules!();

/*
async fn create_authentication(id: String, password: String) {
    let student_id = env::var("STUDENT_ID").expect("Student ID not there!").to_string();
    let account_email = env::var("ACCOUNT_EMAIL").expect("Account Email not there!").to_string();
    let account_password = env::var("ACCOUNT_PASSWORD").expect("Account Password not there!").to_string();
    let project_id = env::var("PROJECT_ID").expect("Project ID not there!").to_string();

    let reqwest_client = reqwest::Client::new();

     // Step 1: Create session and capture cookie
    let session_response = reqwest_client
        .post("https://appwrite.danieldb.uk/v1/account/sessions/email")
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
        .post("https://appwrite.danieldb.uk/v1/account/jwt")
        .header("Content-Type", "application/json")
        .header("X-Appwrite-Project", project_id.clone())
        .header("Cookie", cookie)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let jwt_string = jwt_result["jwt"].as_str().unwrap().to_string();

    let balance = reqwest_client
    .get("https://runshaw-api.danieldb.uk/api/name/get/".to_owned() + &student_id)
    .header("Authorization", format!("Bearer {}", jwt_string))
    .send()
    .await?
    .json::<serde_json::Value>()
    .await?;

    Ok(());
}*/
async fn start_screen(ui: AppWindow) {
    ui.set_highContrast(false);
    ui.set_currentMenu("startup".into());

    let scanned = qrscan::scan_qr().unwrap();
    println!("{}", scanned);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let ui = AppWindow::new()?;
    let weakui = ui.as_weak();

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
    
    start_screen(ui.clone_strong()).await;

    ui.run()?;

    Ok(())
}
