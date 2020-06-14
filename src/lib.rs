#[macro_use]
extern crate nom;
use std::fs;
use std::error::Error;
use serde_json::{ Value};
use parser::QueryCmd;
use std::io::{self, Read};
//use serde_json::json;
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
            [_] => Ok(CmdArgs {
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
        Ok((_, cmd)) => {
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

fn multi_key_access(json: &Value, keys: &[String]) {
    if keys.is_empty() {
        print_json(&json)
    } else {
        let k = &keys[0];
        multi_key_access(&json[k], &keys[1..])
    }
} 



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
            match json {
                Value::Array(vals) =>  for i in idxs {
                                             print_json(&vals[*i]);
                                       }
                _                  =>  panic!("Can only perform Array Index access on a Json Array!")
            }
            
        }
        QueryCmd::KeywordAccess(keys)  => {
            multi_key_access(&json, &keys);
        }
        QueryCmd::MultiCmd(cmds)  => {     
            if cmds.len() == 1 {
                // MultiCmd with single cmd can just be handled as single cmd
                eval_inner(json, &cmds[0]);
            } else {
                // create a vec to hold intermediate json results
                let mut res_vals = vec![json];
                for cmd in cmds {
                    //ToDo this is such spagetti code, needs to be extracted into functions
                    match cmd {
                        QueryCmd::MultiArrayIndex(idx) => { 
                            // allocate Vec for next intermediate Json values 
                            let mut new_res_vals:Vec<&Value> = Vec::new();
                            for val in res_vals {
                                match val  {
                                    Value::Array(arr) => {
                                        for i in idx {
                                            new_res_vals.push(&arr[*i]);
                                        }
                                    }
                                    _                 =>  panic!("Can only perform Array Index access on a Json Array!")
                                }
                            }
                            res_vals = new_res_vals;                                                
                        },
                        QueryCmd::KeywordAccess(keys)  => {
                            let mut new_res_vals:Vec<&Value> = Vec::new();
                            for rv in res_vals {
                                let mut val = rv;
                                for k in keys {
                                    val = & val[k];
                               }
                               new_res_vals.push(&val);
                            }
                            res_vals = new_res_vals;
                        },
                        _ =>  (),
                    }
                }
                for val in res_vals {
                    print_json(val);
                }       
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