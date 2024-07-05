use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// when true, will auto use upstream or use the branch with same name of remote branch to judge if we can push something
    #[arg(short, long, default_value_t = false)]
    pub auto_upstream: bool,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}
