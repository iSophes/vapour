#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

mod apis;
mod qrscan;

slint::include_modules!();

fn convert_string_to_double(string: String) -> f32 {
    let sanitized = string.replace('£', "").trim().to_string();
    sanitized.parse::<f32>().unwrap_or(0.0)
}

fn remove_pound_sign_from_money(money: String) -> String {
    money.replace('£', "")
}

struct AppState {
    current_user_id: Mutex<Option<String>>,
}

async fn start_screen(
    register: bool,
    ui: &AppWindow,
    appwrite_client: &appwrite::client::Client,
    state: &Arc<AppState>,
) {
    ui.set_highContrast(false);
    ui.set_currentMenu("startup".into());

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Always scan for new QR code
    let scanned = match qrscan::scan_qr().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            return;
        }
    };

    // Store the current user ID
    *state.current_user_id.lock().unwrap() = Some(scanned.clone());

    let used_string = apis::my_runshaw_api::get_hello_text(&scanned)
        .await
        .unwrap_or_else(|_| "Welcome!".to_string());

    match apis::sophie_api::does_user_exist(appwrite_client, &scanned).await {
        Ok(exists) => {
            if !exists {
                let _ = apis::sophie_api::create_user(appwrite_client, &scanned).await;
            }
        }
        Err(e) => {
            eprintln!("⚠️ Database Error: {}", e);
            return;
        }
    }

    let balance = apis::sophie_api::get_user_balance(appwrite_client, &scanned)
        .await
        .unwrap_or_else(|_| "£0.00".to_string());

    ui.set_balance(balance.into());
    ui.set_hello_text(used_string.into());
    ui.set_currentMenu("topup".into());
}

async fn begin_card_read(
    appwrite_client: &appwrite::client::Client,
    user_id: String,
    ui: &AppWindow,
    price: f32,
    app_state: Arc<AppState>,
) {
    ui.set_currentMenu("card_input".into());

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();
    let weak_ui = ui.as_weak();
    let client = appwrite_client.clone();

    // Store the cancel handler in a way that can be replaced
    let cancel_handle: Arc<Mutex<Option<Box<dyn FnOnce() + Send>>>> = Arc::new(Mutex::new(None));
    let cancel_handle_clone = cancel_handle.clone();

    ui.on_cancel_payment(move || {
        cancel_token.cancel();
    });

    tokio::select! {
        result = apis::card_reader::read_card(price) => {
            match result {
                Ok(true) => {
                    let current_balance = apis::sophie_api::get_user_balance(&client, &user_id)
                        .await
                        .unwrap_or_else(|_| "£0.00".to_string());

                    let fixed_money = convert_string_to_double(remove_pound_sign_from_money(current_balance));
                    let added_money = price + fixed_money;
                    let converted_money = format!("£{:.2}", added_money);

                    let _ = apis::sophie_api::set_balance(&client, &user_id, &converted_money).await;

                    if let Some(ui) = weak_ui.upgrade() {
                        ui.set_balance(converted_money.into());
                        ui.set_currentMenu("accept".into());
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                    // After acceptance, go back to start screen with new scan
                    if let Some(ui) = weak_ui.upgrade() {
                        let client_clone = client.clone();
                        let state_clone = app_state.clone();
                        slint::spawn_local(async move {
                            start_screen(false, &ui, &client_clone, &state_clone).await;
                        }).unwrap();
                    }
                }
                _ => {
                    if let Some(ui) = weak_ui.upgrade() {
                        ui.set_currentMenu("decline".into());

                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                        // Go back to start screen after decline
                        let client_clone = client.clone();
                        let state_clone = app_state.clone();
                        slint::spawn_local(async move {
                            start_screen(false, &ui, &client_clone, &state_clone).await;
                        }).unwrap();
                    }
                }
            }
        }
        _ = cancel_clone.cancelled() => {
            if let Some(ui) = weak_ui.upgrade() {
                ui.set_currentMenu("cancelled".into());

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                // Go back to start screen after cancellation
                let client_clone = client.clone();
                let state_clone = app_state.clone();
                slint::spawn_local(async move {
                    start_screen(false, &ui, &client_clone, &state_clone).await;
                }).unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let ui = AppWindow::new()?;

    // Time update thread (Safe)
    let weak_ui = ui.as_weak();
    std::thread::spawn(move || loop {
        let now = chrono::Local::now().format("%I:%M %p").to_string();
        let cloned_weak = weak_ui.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = cloned_weak.upgrade() {
                ui.set_time(now.trim_start_matches('0').into());
            }
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
    });

    let appwrite_client = appwrite::Client::new()
        .set_endpoint("https://fra.cloud.appwrite.io/v1")
        .set_project(env::var("PROJECT_ID").unwrap_or_default())
        .set_key(env::var("APPWRITE_API_KEY").unwrap_or_default());

    // Create shared state
    let app_state = Arc::new(AppState {
        current_user_id: Mutex::new(None),
    });

    // Set up keypad handlers once
    let weak_ui_keypad = ui.as_weak();
    ui.on_add_letter(move |letter| {
        if let Some(ui) = weak_ui_keypad.upgrade() {
            let mut amount = ui.get_custom_amount().as_str().to_owned();
            if letter == "." && amount.contains('.') {
                return;
            }
            amount.push_str(&letter);
            ui.set_custom_amount(amount.into());
        }
    });

    let weak_ui_remove = ui.as_weak();
    ui.on_remove_letter(move || {
        if let Some(ui) = weak_ui_remove.upgrade() {
            let mut amount = ui.get_custom_amount().as_str().to_owned();
            amount.pop();
            ui.set_custom_amount(amount.into());
        }
    });

    // Set up submit handler once - uses current user ID from state
    let weak_ui_submit = ui.as_weak();
    let client_for_submit = appwrite_client.clone();
    let state_for_submit = app_state.clone();

    ui.on_submit(move || {
        let strong_ui = match weak_ui_submit.upgrade() {
            Some(u) => u,
            None => return,
        };

        let amount = strong_ui.get_custom_amount();
        if amount.is_empty() {
            return;
        }

        // Get current user ID from state
        let user_id = state_for_submit.current_user_id.lock().unwrap().clone();
        if let Some(user_id) = user_id {
            let price_as_float = convert_string_to_double(amount.as_str().to_owned());
            let moved_appwrite = client_for_submit.clone();
            let moved_ui = strong_ui;
            let moved_state = state_for_submit.clone();

            slint::spawn_local(async move {
                begin_card_read(
                    &moved_appwrite,
                    user_id,
                    &moved_ui,
                    price_as_float,
                    moved_state,
                )
                .await;
            })
            .unwrap();
        }
    });

    // Set up go_to_start handler once
    let ui_for_start = ui.as_weak();
    let client_for_start = appwrite_client.clone();
    let state_for_start = app_state.clone();

    ui.on_go_to_start(move || {
        let moved_ui = ui_for_start.clone();
        let moved_appwrite = client_for_start.clone();
        let moved_state = state_for_start.clone();

        slint::spawn_local(async move {
            if let Some(strong_ui) = moved_ui.upgrade() {
                // Always scan for new QR code when going back to start
                start_screen(false, &strong_ui, &moved_appwrite, &moved_state).await;
            }
        })
        .unwrap();
    });

    // Initial Start
    let ui_handle = ui.as_weak();
    let client_clone = appwrite_client.clone();
    let state_clone = app_state.clone();

    slint::spawn_local(async move {
        if let Some(ui) = ui_handle.upgrade() {
            start_screen(true, &ui, &client_clone, &state_clone).await;
        }
    })
    .unwrap();

    ui.run()?;
    Ok(())
}
