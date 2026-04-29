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
    let project_id = env::var("RUNSHAW_ID").expect("My Runshaw Project ID not there!").to_string();

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

pub async fn get_name(student_id: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let my_id = env::var("MY_RUNSHAW_ID").expect("").to_string();
    let my_password = env::var("MY_RUNSHAW_ID").expect("").to_string()
    let jwt = check_password_and_authenticate(my_id, my_password).await?.unwrap();
    
    let reqwest_client = reqwest::Client::new();

    let name_callback = reqwest_client
        .get("https://runshaw-api.danieldb.uk/api/name/get/".to_owned() + &student_id)
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer ".to_owned() + &jwt)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let name = name_callback["name"].to_string();

    if name_callback["detail"] != serde_json::Value::Null {
        return Ok(Some(name_callback["detail"].to_string()))
    } 
    
    return Ok(Some(name));
}