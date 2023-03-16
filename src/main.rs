#[tokio::main]
async fn main() {
    let response = reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await
        .unwrap();
    println!("Status: {}", response.status());
    let body = response.text().await.unwrap();
    println!("Body: {}", body);
}
