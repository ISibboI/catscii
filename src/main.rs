use serde::Deserialize;

#[tokio::main]
async fn main() {
    let response = reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await
        .unwrap();
    println!("Status: {}", response.status());

    #[derive(Deserialize)]
    struct CatImage {
        url: String,
    }

    let images: Vec<CatImage> = response.json().await.unwrap();
    let image = images.first().expect("The cat API should return at least one image.");
    println!("The image is at {}", image.url);
}
