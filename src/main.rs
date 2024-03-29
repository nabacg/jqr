use jqr::CmdArgs;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = CmdArgs::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing command arguments: {}", err); //eprintln! writes to StdErr
        process::exit(1);
    });

    if let Err(e) = jqr::eval_cmd(cmd) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
