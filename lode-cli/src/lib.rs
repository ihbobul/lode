pub use clap::Parser;
use num_cpus;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Target URL to load test
    #[arg(short, long)]
    pub url: String,

    /// Number of requests to send
    #[arg(short, long, default_value = "100")]
    pub requests: u32,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = num_cpus::get())]
    pub concurrency: usize,

    /// HTTP method to use (GET, POST, etc.)
    #[arg(short, long, default_value = "GET")]
    pub method: String,

    /// Request timeout in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,

    /// JSON body for POST/PUT requests
    #[arg(short, long)]
    pub body: Option<String>,

    /// Custom headers (format: "key:value")
    #[arg(short = 'H', long, num_args = 0.., value_delimiter = ',')]
    pub headers: Option<Vec<String>>,

    /// Output format (text or json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Show debug logs
    #[arg(long, default_value_t = false)]
    pub no_capture: bool,
}
