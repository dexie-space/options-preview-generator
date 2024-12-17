use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::Browser;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Arc;
use warp::Filter;

lazy_static! {
    static ref BROWSER: Arc<Browser> =
        Arc::new(Browser::default().expect("Failed to create browser"));
}

// Query parameters structure
#[derive(Deserialize, Debug)]
struct QueryParams {
    underlying_asset: String,
    settlement_asset: String,
    underlying_mojos: String,
    settlement_mojos: String,
    expiration: String,
    contract_type: String,
    contract_id: String,
}

async fn generate_screenshot(params: QueryParams) -> Result<impl warp::Reply, warp::Rejection> {
    let browser = Arc::clone(&BROWSER);
    let tab = browser
        .new_tab()
        .map_err(|_| warp::reject::custom(ServerError))?;

    let html_path = std::env::current_dir()
        .map_err(|_| warp::reject::custom(ServerError))?
        .join(format!(
            "templates/{}.html",
            params.contract_type.to_lowercase()
        ))
        .display()
        .to_string();

    let url = format!(
        "file://{}?underlying_asset={}&settlement_asset={}&underlying_mojos={}&settlement_mojos={}&expiration={}&contract_id={}",
        html_path,
        params.underlying_asset,
        params.settlement_asset,
        params.underlying_mojos,
        params.settlement_mojos,
        params.expiration,
        params.contract_id
    );

    println!("Generating for {:?}", params);

    tab.navigate_to(&url)
        .map_err(|_| warp::reject::custom(ServerError))?;

    tab.wait_until_navigated()
        .map_err(|_| warp::reject::custom(ServerError))?;

    let png_data = tab
        .wait_for_element("#content")
        .map_err(|_| warp::reject::custom(ServerError))?
        .capture_screenshot(CaptureScreenshotFormatOption::Png)
        .map_err(|_| warp::reject::custom(ServerError))?;

    Ok(warp::reply::with_header(
        png_data,
        "Content-Type",
        "image/png",
    ))
}

// Custom error type for error handling
#[derive(Debug)]
struct ServerError;
impl warp::reject::Reject for ServerError {}

#[tokio::main]
async fn main() {
    let screenshot = warp::get()
        .and(warp::query::<QueryParams>())
        .and_then(generate_screenshot);

    println!("Server starting on http://localhost:3030");

    warp::serve(screenshot).run(([127, 0, 0, 1], 3030)).await;
}
