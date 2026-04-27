use reqwest;

pub async fn does_account_exist(student_id: String) -> Result<bool, reqwest::Error> {
    let does_exist = reqwest::get("https://runshaw-api.danieldb.uk/api/exists/".to_owned() + &student_id).await?;
    let body = does_exist.text().await?;
    return Ok(body == "true");
}