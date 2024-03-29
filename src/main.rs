use anyhow::Context;
use axum::body::BoxBody;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use opentelemetry::sdk::export::trace::stdout;
use reqwest::Client;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{error, info, Level, span, warn};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install a new OpenTelemetry trace pipeline
    let tracer = stdout::new_pipeline().install_simple();

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);

    /*let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .with_context(|| format!("RUST_LOG should be a valid tracing filter"))?;
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();*/

    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
        let _enter = root.enter();

        error!("This event will be logged in the root span.");
    });

    let app = Router::new().route("/", get(root_get));

    let quit_signal = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };

    let address = "0.0.0.0:8080".parse()?;
    info!("Listening on {address}");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(quit_signal)
        .await?;

    Ok(())
}

async fn root_get() -> Response<BoxBody> {
    match get_cat_ascii_art().await {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(error) => {
            error!("Something went wrong: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
        }
    }
}

async fn get_cat_ascii_art() -> color_eyre::Result<String> {
    let image_bytes = get_cat_image_bytes(&Default::default()).await?;
    let image = image::load_from_memory(&image_bytes)?;
    let ascii_art = artem::convert(
        image,
        artem::options::OptionBuilder::new()
            .target(artem::options::TargetType::HtmlFile(true, true))
            .build(),
    );
    Ok(ascii_art)
}

async fn get_cat_image_bytes(client: &Client) -> color_eyre::Result<Vec<u8>> {
    Ok(client
        .get(get_cat_image_url(client).await?)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
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
