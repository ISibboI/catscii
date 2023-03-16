use pretty_hex::PrettyHex;
use reqwest::Client;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let image_bytes = get_cat_image_bytes(&Default::default()).await.unwrap();
    println!("The image is {:?}", image_bytes[..200].hex_dump());
}

async fn get_cat_image_bytes(client: &Client) -> color_eyre::Result<Vec<u8>> {
    Ok(client.get(get_cat_image_url(client).await?).send().await?.error_for_status()?.bytes().await?.to_vec())
}

async fn get_cat_image_url(client: &Client) -> color_eyre::Result<String> {
    let api_url = "https://api.thecatapi.com/v1/images/search";
    let res = client.get(api_url).send().await?;
    if !res.status().is_success() {
        return Err(color_eyre::eyre::eyre!(
            "The Cat API returned HTTP {}",
            res.status()
        ));
    }

    #[derive(Deserialize)]
    struct CatImage {
        url: String,
    }
    let images: Vec<CatImage> = res.json().await?;
    // this syntax is new in Rust 1.65
    let Some(image) = images.into_iter().next() else {
        return Err(color_eyre::eyre::eyre!("The Cat API returned no images"));
    };

    Ok(image.url)
}