use clap::Parser;

/// zentop - AMD Zen CPU Monitor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Refresh rate in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    pub refresh_rate: u64,
}

impl Config {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { refresh_rate: 1000 }
    }
}
