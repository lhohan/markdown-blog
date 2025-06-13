#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_utc_timestamps()
        .with_colors(true)
        .init()
        // in Shuttle environment logger is preconfigured
        .unwrap_or_else(|e| println!("Error configuring logger: {e}"));

    let app = blog_engine::create_app_with_defaults();
    Ok(app.into())
}

#[cfg(not(feature = "shuttle"))]
fn main() {
    panic!("This binary requires the 'shuttle' feature to be enabled");
}
