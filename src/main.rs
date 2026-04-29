// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::cell::Cell;
use std::env;
use std::error::Error;
use std::rc::Rc;

mod qrscan;
mod auth;
mod database;

slint::include_modules!();

fn remove_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    return chars.as_str();
}

async fn start_screen(ui: &AppWindow, failed_scan_index: Rc<Cell<u32>>) {
    ui.set_highContrast(false);
    ui.set_currentMenu("startup".into());

    tokio::time::sleep(std::time::Duration::from_secs(3)).await; // Temporary sleep for testing purposes

    let scanned = qrscan::scan_qr().unwrap(); 
    let does_account_exist = auth::does_account_exist(scanned.clone()).await.unwrap();

    // TODO: Check if they have an account in our database
    // if not, create one. else get it
    // grab name from my runshaw
    // if no name then just set text to hello!
    let mut used_string = String::from("Hello!");

    if does_account_exist {
        let mut name = auth::get_name(scanned.clone()).await.unwrap().unwrap();

        if name.chars().next().unwrap() == "\"".to_owned().chars().next().unwrap() {
            name = remove_first_and_last(&name).to_string();
        }

        used_string = format!("Hello!, {}!", name);
    }

    ui.set_name(used_string.into());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let ui = AppWindow::new()?;
    let weakui = ui.as_weak();
    
    let failed_scan_index = Rc::new(Cell::new(0u32));

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
        start_screen(&second_ui.upgrade().unwrap(), failed_scan_index).await;
    }).unwrap();
    
    ui.run()?;

    Ok(())
}
