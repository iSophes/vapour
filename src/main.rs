// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::env;
use std::error::Error;
mod apis;
mod qrscan;

slint::include_modules!(); // yes this code errors, i do not care

async fn start_screen(ui: &AppWindow, appwrite_client: &appwrite::client::Client) {
    ui.set_highContrast(false);
    ui.set_currentMenu("startup".into());

    tokio::time::sleep(std::time::Duration::from_secs(3)).await; // Temporary sleep for testing purposes

    let scanned = qrscan::scan_qr().unwrap();
    let used_string = apis::my_runshaw_api::get_hello_text(&scanned).await.unwrap();

    // handle database stuff

    let does_user_exist = apis::sophie_api::does_user_exist(appwrite_client, &scanned).await.unwrap();

    if does_user_exist == false {
        let _created_row = apis::sophie_api::create_user(appwrite_client, &scanned).await; // we have this just to shut the thing up
    }

    let balance = apis::sophie_api::get_user_balance(appwrite_client, &scanned).await;

    ui.set_balance(balance.into());
    ui.set_hello_text(used_string.into());
    ui.set_currentMenu("topup".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let ui = AppWindow::new()?;
    let weakui = ui.as_weak();

    let appwrite_project_id = env::var("PROJECT_ID")
        .expect("No appwrite project ID")
        .to_string();

    let appwrite_api_key = env::var("APPWRITE_API_KEY")
        .expect("No appwrite project API Key")
        .to_string();

    let appwrite_client = appwrite::Client::new()
        .set_endpoint("https://fra.cloud.appwrite.io/v1")
        .set_project(appwrite_project_id)
        .set_key(appwrite_api_key);

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

    let second_ui = ui.as_weak();

    slint::spawn_local(async move {
        start_screen(&second_ui.upgrade().unwrap(), &appwrite_client).await;
    })
    .unwrap();

    ui.run()?;

    Ok(())
}
