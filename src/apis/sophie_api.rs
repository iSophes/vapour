// My own appwrite database APIs. Replace with your own. 

use appwrite;
use reqwest;

pub async fn does_user_exist(appwrite_client: &appwrite::client::Client, student_id: &String) -> Result<bool, reqwest::Error>  {
    // check if user exists.
    return Ok(false);
}

pub async fn create_user(appwrite_client: &appwrite::client::Client, student_id: &String) {
    
}

pub async fn get_user_balance(appwrite_client: &appwrite::client::Client, student_id: &String) {

}

pub async fn set_balance(appwrite_client: &appwrite::client::Client, student_id: &String) {

} 

