use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::Browser;
use headless_chrome::LaunchOptions;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::Deserialize;
use std::ffi::OsStr;
use std::sync::Arc;
use warp::Filter;

lazy_static! {
    static ref BROWSER: Arc<Mutex<Option<Arc<Browser>>>> =
        Arc::new(Mutex::new(Some(create_browser())));
}

fn create_browser() -> Arc<Browser> {
    Arc::new(
        Browser::new(
            LaunchOptions::default_builder()
                .args(vec![
                    OsStr::new("--force-device-scale-factor=2"),
                    OsStr::new("--window-size=2560,1440"),
                    OsStr::new("--keep-alive"),
                ])
                .idle_browser_timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
        )
        .expect("Failed to create browser"),
    )
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
    let browser = {
        let mut browser_guard = BROWSER.lock();
        if browser_guard.is_none() {
            *browser_guard = Some(create_browser());
        }
        Arc::clone(browser_guard.as_ref().unwrap())
    };

    let tab = match browser.new_tab() {
        Ok(tab) => tab,
        Err(_) => {
            // Browser might have timed out, try to recreate it
            let mut browser_guard = BROWSER.lock();
            *browser_guard = Some(create_browser());
            browser_guard.as_ref().unwrap().new_tab().map_err(|e| {
                eprintln!("Failed to create new tab after browser restart: {:?}", e);
                warp::reject::custom(ServerError)
            })?
        }
    };

    let html_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("templates")
        .join(format!("{}.html", params.contract_type.to_lowercase()))
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
