mod bot_handler;
mod google_apis;

#[tokio::main]
async fn main() {
    bot_handler::bot_main::run_bot().await;
}