use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match hh_sim::run(&args) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(hh_sim::Fail::Usage(e)) => {
            eprintln!("error: {e}\n\n{}", hh_sim::USAGE);
            ExitCode::from(2)
        }
        Err(hh_sim::Fail::Check(e)) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}
