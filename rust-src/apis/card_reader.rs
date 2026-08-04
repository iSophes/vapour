pub async fn read_card(_price: f32) -> Result<bool, Box<dyn std::error::Error>> {
    // I do not possess a card reader nor APIs to deal with the data from said card reader
    // if an individual is reading this and knows what they're doing: throw a PR in
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    return Err("test err".into()); // test
                                   //return Ok(false);
}
