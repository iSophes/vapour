use std::env;

use reqwest;

pub async fn does_account_exist(student_id: String) -> Result<bool, reqwest::Error> {
    let does_exist = reqwest::get("https://runshaw-api.danieldb.uk/api/exists/".to_owned() + &student_id).await?;
    let body = does_exist.text().await?;

    println!("{}", body);

    return Ok(body == "{\"exists\":true}");
}

pub async fn check_password_and_authenticate(student_id: String, password: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let reqwest_client = reqwest::Client::new();
    let project_id = env::var("PROJECT_ID").expect("Project ID not there!").to_string();

    let session_response = reqwest_client
        .post("https://appwrite.danieldb.uk/v1/account/sessions/email")
        .header("Content-Type", "application/json")
        .header("X-Appwrite-Project", project_id.clone())
        .json(&serde_json::json!({
            "email": student_id + "@student.runshaw.ac.uk",
            "password": password
        }))
        .send()
        .await?;

    if !session_response.status().is_success() {
        return Ok(None)     
    }

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

    return Ok(Some(jwt_string))
}

pub async fn get_balance(jwt: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let reqwest_client = reqwest::Client::new();

    println!("getting balance");

    let balance_callback = reqwest_client
        .post("https://appwrite.danieldb.uk/v1/api/payments/balance")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer ".to_owned() + &jwt)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{}", balance_callback);

    let balance = balance_callback.as_str().unwrap().to_string();

    return Ok(Some(balance));
}