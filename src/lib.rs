#[macro_use]
extern crate nom;
use std::fs;
use std::error::Error;
use serde_json::{ Value};
use parser::QueryCmd;
use std::io::{self, Read};
mod parser;

#[derive(Debug)]
pub struct CmdArgs {
    input_file: Option<String>,
    query: Option<String>
}

impl CmdArgs {
    pub fn new(args: &[String]) -> Result<CmdArgs, String> {
        match args {
            [_, input_file, query] =>  Ok(CmdArgs{
                input_file: Some(input_file.clone()),
                query: Some(query.clone())
            }),
            [_, query] => Ok(CmdArgs {
                input_file: None,
                query: Some(query.clone())
            }),
            [] => Ok(CmdArgs {
                input_file: None,
                query: None
            }),
            _ => return Err(format!("Wrong number of arguments passed, jqr expects 0, 1 or 2 args. Passed= {}", args.len()))
        }
    }
}


fn parse_cmd(cmd_str: &String) -> Result<QueryCmd, &'static str> {
    // https://github.com/Geal/nom/blob/master/doc/making_a_new_parser_from_scratch.md
    match parser::parse(&cmd_str) {
        Ok(("", cmd)) => {
         //   println!("Cmd={:?}", cmd); // ToDo add a --Debug flag to print it out?
            Ok(cmd)
        }
        Ok((input_left, cmd)) => {
        //    println!("Cmd={:?} but found unconsumed input={}" , cmd, input_left);
            Ok(cmd)
        }
        Err(e) => {
            eprintln!("ERROR parsing cmd={:?} error={:?}", cmd_str,  e);
            Err("Failing now")
        }
    }
    
}

pub fn read_json_file(file: &String) -> Result<Value,  Box<dyn Error>> {
    let file_contents = &fs::read_to_string(file)?; 
    let json: Value = serde_json::from_str(file_contents)?;

    Ok(json)
}

fn read_json_from_stdin() -> Result<Value, Box<dyn Error>> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;

    let json: Value = serde_json::from_str(&buffer)?;

    Ok(json)
}

fn multi_key_access(json: &Value, keys: &[String]) -> String {
    if keys.is_empty() {
        json.to_string()
    } else {
        let k = &keys[0];
        multi_key_access(&json[k], &keys[1..])
    }
} 

// fn multi_key_access_V(json: &Value, keys: &[String]) -> &'static Value {
//     if keys.is_empty() {
//         json
//     } else {
//         let k = &keys[0];
//         multi_key_access_V(&json[k], &keys[1..])
//     }
// } 

// fn find_value(json: &Value, cmd: QueryCmd) -> Vec<&Value> {
//     match cmd {
//         QueryCmd::MultiArrayIndex(idxs)  => {
//             let vals = json.as_array().unwrap();
//             idxs.iter().map(|i| &vals[*i]).collect::<Vec<_>>()
//         }
//         QueryCmd::KeywordAccess(keys)  => vec![multi_key_access_V(&json, &keys)],
//         QueryCmd::MultiCmd(cmds)  => {
//             if cmds.is_empty() {
//                 vec![json]
//             } else  {
//                 let v = find_value(json, cmds[0]);
//                 find_value(&v, QueryCmd::MultiCmd(cmds[1..].to_vec()))
//             }
//         }
//     }
// }

fn print_json(val:&Value) {
    if let Ok(s) = serde_json::to_string_pretty(val) {
        println!("{}", s)
    } else {
        println!("{}", val.to_string());
    }
}

fn eval_inner(json:&Value, query: &QueryCmd) {
    match query {
        QueryCmd::MultiArrayIndex(idxs)  => {
            let vals = json.as_array().unwrap();
            for i in idxs {
                print_json(&vals[*i]);
            }
        }
        QueryCmd::KeywordAccess(keys)  => {
            let string_val = multi_key_access(&json, &keys);
            println!("{}", string_val);
        }
        QueryCmd::MultiCmd(cmds)  => {
           
            if cmds.len() == 1 {
                eval_inner(json, &cmds[0]);
            } else {
                let mut val = json; 
                // This needs to be refactored, maybe using a function like find_value
                for cmd in cmds {
                    match cmd {
                        QueryCmd::MultiArrayIndex(idx) => { 
                            let arr  = val.as_array().unwrap();                            
                            val = & arr[idx[0]];
                        },
                        QueryCmd::KeywordAccess(keys)  => for k in keys {
                            val = & val[k];
                        },
                        _ => val = & val,
                    }
                }
                print_json(val);
            }
        }
    }
}

pub fn eval_cmd(cmd: CmdArgs) -> Result<(), Box<dyn Error>> {
    let json: Value = match &cmd.input_file {
        Some(input_path) => read_json_file(&input_path)?,
        None             => read_json_from_stdin()?
    };
    
    match cmd.query.map(|query| parse_cmd(&query)) {
        Some(Ok(cmd))   => eval_inner(&json, &cmd), 
        Some(Err(msg))  => println!("Failed at cmd parsing with error= {}", msg), // Seems like too many levels of error handling
        None            => print_json(&json)
    }
    Ok(())
}