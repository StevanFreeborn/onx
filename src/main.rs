use std::process;

use onx::{parse_cli_from, render_cli_error, run};

#[tokio::main]
async fn main() {
  let cli = match parse_cli_from(std::env::args_os()) {
    Ok(cli) => cli,
    Err(err) => {
      if err.code == 0 {
        print!("{}", err.message);
        process::exit(0);
      }
      eprintln!("{}", render_cli_error(&err, false));
      process::exit(err.code)
    }
  };

  if let Err(err) = run(cli).await {
    eprintln!("{}", render_cli_error(&err, false));
    process::exit(err.code)
  }
}
