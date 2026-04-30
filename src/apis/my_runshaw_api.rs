// This will be the API calls to 'My Runshaw' for the Runshaw College version of the app.

use reqwest;
use std::env;

fn remove_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    return chars.as_str();
}

async fn get_my_runshaw_jwt() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let reqwest_client = reqwest::Client::new();
    let project_id = env::var("RUNSHAW_ID")
        .expect("My Runshaw Project ID not there!")
        .to_string();
    let student_id = env::var("MY_RUNSHAW_ID")
        .expect("No MY RUNSHAW ID")
        .to_string();
    let password = env::var("MY_RUNSHAW_PASSWORD")
        .expect("No MY RUNSHAW password")
        .to_string();

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
        return Ok(None);
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

    return Ok(Some(jwt_string));
}

pub async fn does_account_exist(student_id: &String) -> Result<bool, reqwest::Error> {
    let does_exist =
        reqwest::get("https://runshaw-api.danieldb.uk/api/exists/".to_owned() + &student_id)
            .await?;
    let body = does_exist.text().await?;
    return Ok(body == "{\"exists\":true}");
}

async fn get_name_from_student_id(
    jwt: &str,
    student_id: &String,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let reqwest_client = reqwest::Client::new();

    let name_callback = reqwest_client
        .get("https://runshaw-api.danieldb.uk/api/name/get/".to_owned() + student_id)
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer ".to_owned() + jwt)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if name_callback["detail"] != serde_json::Value::Null {
        return Ok(Some("FAIL!!!!!! RETRY!!!".to_owned()));
    }

    let mut name = name_callback["name"].to_string();

    if name.chars().next().unwrap() == "\"".chars().next().unwrap() {
        name = remove_first_and_last(&name).to_string();
    }

    return Ok(Some(name));
}

pub async fn get_hello_text(student_id: &String) -> Result<String, Box<dyn std::error::Error>> {
    let jwt = get_my_runshaw_jwt().await?.unwrap();

    if !does_account_exist(student_id).await? {
        return Ok("Hello!".to_owned());
    }

    let mut name = get_name_from_student_id(&jwt, student_id).await?.unwrap();
    if name.starts_with("FAIL!") {
        // Retry and if it fails, fallback to hello. (Somethings probably going really bad that I don't want to deal with)
        name = get_name_from_student_id(&jwt, student_id).await?.unwrap();
        if name.starts_with("FAIL!") {
            return Ok("Hello!".to_owned());
        }
    }

    return Ok("Hello, ".to_owned() + &name + "!");
}
