use clap::Parser;
use my_weather::get_weather;

#[derive(Debug, Parser)]
struct Args {
  #[arg(short, default_value_t = false)]
  current: bool,
}

#[cfg(feature = "async")]
#[tokio::main]
pub async fn main() {
  let args = Args::parse();

  match get_weather().await {
    Ok(forecast) => {
      if args.current {
        for entry in forecast.current_forecast() {
          println!("{entry}");
        }
      } else {
        println!("{forecast}");
      }
    }
    Err(e) => eprintln!("{e}"),
  }
}

#[cfg(not(feature = "async"))]
pub fn main() {
  let args = Args::parse();

  match get_weather() {
    Ok(forecast) => {
      if args.current {
        for entry in forecast.current_forecast() {
          println!("{entry}");
        }
      } else {
        println!("{forecast}");
      }
    }
    Err(e) => eprintln!("{e}"),
  }
}
