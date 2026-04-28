// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use std::cell::Cell;
use std::env;
use std::error::Error;
use std::rc::Rc;

mod qrscan;
mod auth;

slint::include_modules!();

async fn start_screen(ui: &AppWindow, failed_scan_index: Rc<Cell<u32>>) {
    ui.set_highContrast(false);
    ui.set_currentMenu("topup".into());

    tokio::time::sleep(std::time::Duration::from_secs(300)).await;

    let scanned = qrscan::scan_qr().unwrap();
    let does_account_exist = auth::does_account_exist(scanned.clone()).await.unwrap();

    if !does_account_exist {
        ui.set_currentMenu("noaccount".into());

        let weak_ui = ui.as_weak();

        failed_scan_index.set(failed_scan_index.get() + 1);
        let current_get = failed_scan_index.get();
        let cloned_value = failed_scan_index.clone();

        slint::spawn_local(async move {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            if current_get != cloned_value.get() {
                return; // Multiple users have failed. 
            }

            if let Some(ui) = weak_ui.upgrade() {
                failed_scan_index.set(0);
                if ui.get_currentMenu() != "noaccount" {
                    return;
                }

                ui.set_currentMenu("startup".into());
            }
        }).unwrap();
    } else {
        ui.set_currentMenu("passwordinput".into());
        ui.on_check_password(move |password| {
            let id = scanned.clone();
            let cloned_password = password.clone();

            let _ = slint::spawn_local(async move {
                check_password(id, cloned_password.to_string()).await;
            });
        });
    }
}

async fn check_password(id: String, password: String) {
    match auth::check_password(id, password).await {
        Ok(Some(jwt)) => {println!("success")},
        Ok(None) => {println!("failed")},
        Err(e)        => {},
    }
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
