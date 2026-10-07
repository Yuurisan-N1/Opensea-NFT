use opensea_mint::engine::orchestrator;
use opensea_mint::ui::logger::{close_row, lr, sanitize};
use opensea_mint::ui::prompt::restore_echo;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    tokio::select! {
        result = orchestrator::run() => {
            if let Err(error) = result {
                lr(&sanitize(&error.to_string()));
            }
        }
        _ = tokio::signal::ctrl_c() => {
            restore_echo();
            close_row();
            lr("Script stopped by user");
        }
    }
}
