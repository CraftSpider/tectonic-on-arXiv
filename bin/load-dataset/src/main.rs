use clap::{ArgAction, Parser};
use std::error::Error;

mod bundle;
mod download;

#[derive(Parser)]
struct Args {
    dataset: u32,
    #[arg(long, action = ArgAction::SetTrue)]
    skip_download: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    skip_bundle: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if !args.skip_download {
        download::download_dataset(args.dataset).await?;
    }
    if !args.skip_bundle {
        bundle::bundle_dataset(args.dataset).await?;
    }

    Ok(())
}

fn pretty_print(mut e: &dyn Error) {
    println!("    0: {e}");
    let mut idx = 1;
    while let Some(cause) = e.source() {
        println!("    {idx}: {cause}");
        idx += 1;
        e = cause;
    }
}
