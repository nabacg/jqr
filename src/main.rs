use std::env;
use std::process;
use jqr::CmdArgs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = CmdArgs::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing command arguments: {}", err); //eprintln! writes to StdErr
        process::exit(1);
    });

    if let Err(e) = jqr::print_json(cmd) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

}
