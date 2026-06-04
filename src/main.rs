mod config;
mod banner;
mod handler;

use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{routing::any, Router, extract::DefaultBodyLimit};
use std::net::SocketAddr;
use crate::config::Config;
use crate::handler::{handler, AppState};
use crate::banner::print_banner;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the directory to serve
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Address/port tuple string to listen to
    #[arg(short, long)]
    pub listen: Option<String>,

    /// Explicit port mapping
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Single Page Application fallback
    #[arg(short, long)]
    pub single: bool,

    /// Path to a custom configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Enable verbose log outputs
    #[arg(short, long)]
    pub debug: bool,

    /// Prevent copying the local address to the clipboard
    #[arg(long)]
    pub no_clipboard: bool,

    /// Initialize a configuration file (json, yaml, toml)
    #[arg(long, num_args(0..=1), default_missing_value = "json")]
    pub init: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    if let Some(format) = args.init {
        Config::generate_default_config(&format)?;
        return Ok(());
    }

    // Initialize tracing
    let filter = if args.debug {
        "serve=debug,tower_http=debug"
    } else {
        "serve=info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    let mut config = Config::find_and_load(args.config)?;
    
    // Handle SPA fallback via config field if -s is passed
    if args.single {
        config.spa = Some(true);
    }

    let shared_state = Arc::new(AppState {
        config,
        base_path: args.path.canonicalize().unwrap_or(args.path),
    });

    let app = Router::new()
        .fallback(any(handler))
        .with_state(shared_state)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)); // 1GB limit for body

    // Parsing address and port
    let addr = if let Some(listen_str) = args.listen {
        if listen_str.contains(':') {
            listen_str.parse::<SocketAddr>()?
        } else {
            let port = listen_str.parse::<u16>().unwrap_or(3000);
            format!("0.0.0.0:{}", port).parse::<SocketAddr>()?
        }
    } else if let Some(port) = args.port {
        format!("0.0.0.0:{}", port).parse::<SocketAddr>()?
    } else {
        "0.0.0.0:3000".parse::<SocketAddr>()?
    };

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();

    print_banner(actual_port, args.no_clipboard);

    axum::serve(listener, app).await?;
    
    Ok(())
}
