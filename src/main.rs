use gitflow::{args::Args, flow};

fn main() {
    let args = Args::new();

    flow::run(args).unwrap();
}
