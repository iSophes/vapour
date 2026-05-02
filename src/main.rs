// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio_util::sync::CancellationToken;
use std::rc::Rc;
use std::cell::RefCell;
use slint::{Timer, TimerMode};

mod apis;
mod qrscan;
mod gif_decoder;

slint::include_modules!(); // yes this code errors, i do not care

fn gif(ui: AppWindow, path: &str) {
    let frames = Rc::new(gif_decoder::decode_gif_frames(path));
    let index = Rc::new(RefCell::new(0usize));
    let ui_handle = ui.as_weak();

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(100), move || {
        let ui = ui_handle.unwrap();
        let i = *index.borrow();
        ui.set_current_frame(frames[i].clone());
        *index.borrow_mut() = (i + 1) % frames.len();
    });
}

async fn start_screen(ui: &AppWindow, appwrite_client: &appwrite::client::Client) {
    ui.set_highContrast(false);
    ui.set_currentMenu("startup".into());

    tokio::time::sleep(std::time::Duration::from_secs(3)).await; // Temporary sleep for testing purposes

    let scanned = qrscan::scan_qr().unwrap();
    let used_string = apis::my_runshaw_api::get_hello_text(&scanned)
        .await
        .unwrap();

    // handle database stuff

    let does_user_exist = apis::sophie_api::does_user_exist(appwrite_client, &scanned)
        .await
        .unwrap();

    if does_user_exist == false {
        let _created_row = apis::sophie_api::create_user(appwrite_client, &scanned).await;
        // we have this just to shut the thing up
    }

    let balance = apis::sophie_api::get_user_balance(appwrite_client, &scanned).await;

    keypad_ui(appwrite_client, &scanned, ui).await;
    ui.set_balance(balance.into());
    ui.set_hello_text(used_string.into());
    ui.set_currentMenu("topup".into());
}

fn convert_string_to_double(string: String) -> f32 {
    if string.contains(".") {
        let collected_split: Vec<&str> = string.split(".").collect();
        let mut pence: String = String::new();
        // trim for first two letters
        if collected_split[1].len() > 2 {
            let first_value = collected_split[1].chars().next().unwrap().to_string();
            let second_value = collected_split[1].chars().nth(1).unwrap().to_string();
            pence = format!("{}{}", first_value, second_value);
        }

        let price = collected_split[1].to_owned() + &pence;
        let converted_price: f32 = price.parse().unwrap();

        return converted_price; 
    } 

    return string.parse().unwrap();
}

fn convert_money_to_float(money: String) {
    let mut fixed_money = "";

}

async fn keypad_ui(appwrite_client: &appwrite::client::Client, student_id: &String, ui: &AppWindow) {
    let cloned_ui_for_add_letter = ui.as_weak();
    let cloned_ui_for_remove_letter = ui.as_weak();
    let cloned_ui_for_submit = ui.as_weak();

    ui.on_add_letter(move |letter| {
        let amount = cloned_ui_for_add_letter.upgrade().unwrap().get_custom_amount();
        
        if letter.as_str().to_owned().chars().next().unwrap() == ".".chars().next().unwrap() {
            if amount.contains(".") {
                return; // only one decimal allowed
            }          
        }

        let new_string = amount.as_str().to_owned() + &letter;
        cloned_ui_for_add_letter.upgrade().unwrap().set_custom_amount(new_string.into());
    });

    ui.on_remove_letter(move || {
        let amount = cloned_ui_for_remove_letter.upgrade().unwrap().get_custom_amount();
        let mut new_string = amount.as_str().to_owned();
        new_string.pop();
        cloned_ui_for_remove_letter.upgrade().unwrap().set_custom_amount(new_string.into());
    });

    let cloned_appwrite = appwrite_client.clone();
    let cloned_id = student_id.clone();

    ui.on_submit(move || {
        let amount = cloned_ui_for_submit.upgrade().unwrap().get_custom_amount();
        let actual_string = amount.as_str().to_owned();

        let price_as_float = convert_string_to_double(actual_string);

        let _ = begin_card_read(&cloned_appwrite, cloned_id.to_owned(), cloned_ui_for_submit.clone().upgrade().unwrap(), price_as_float);
    });
}

async fn begin_card_read(appwrite_client: &appwrite::client::Client, user_id: String, ui: AppWindow, price: f32) {
    ui.set_currentMenu("card_input".into());

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    ui.on_cancel_payment(move || {
        cancel_token.cancel();
    });

    tokio::select! {
        result = apis::card_reader::read_card(price) => {
            match result {
                Ok(true) => {
                    let current_balance = apis::sophie_api::get_user_balance(appwrite_client, &user_id).await;

                    apis::sophie_api::set_balance(appwrite_client, user_id, );
                    ui.set_currentMenu("accept".into());
                }
                Ok(false) => {}
                Err(_) => {}
            }
        }
        _ = cancel_clone.cancelled() => {
            ui.set_currentMenu("cancelled".into());
        }
    }


    
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
