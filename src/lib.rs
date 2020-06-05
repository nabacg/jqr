#[macro_use]
extern crate nom;
use std::fs;
use std::error::Error;
use serde_json::{ Value};
use parser::QueryCmd;
mod parser;

#[derive(Debug)]
pub struct CmdArgs {
    input_file: String,
    query: Option<String>
}

impl CmdArgs {
    pub fn new(args: &[String]) -> Result<CmdArgs, &'static str> {
        let arg_len = args.len();

        if arg_len < 2 {
            return Err("jqr requires at least 1 argument");
        }

        if arg_len == 3 {
            Ok(CmdArgs{
                input_file: args[1].clone(),
                query: Some(args[2].clone())
            })
        } else {
            Ok(CmdArgs {
                input_file: args[1].clone(),
                query: None
            })
        }
        
    }
}


fn parse_cmd(cmd_str: &String) -> Result<QueryCmd, &'static str> {
    // https://github.com/Geal/nom/blob/master/doc/making_a_new_parser_from_scratch.md
    match parser::parse(&cmd_str) {
        Ok(("", cmd)) => {
            println!("Cmd={:?}", cmd);
            Ok(cmd)
        }
        Ok((input_left, cmd)) => {
            println!("Cmd={:?} but found unconsumed input={}" , cmd, input_left);
            Ok(cmd)
        }
        Err(e) => {
            eprintln!("ERROR parsing cmd={:?} error={:?}", cmd_str,  e);
            Err("Failing now")
        }
    }
    
}

pub fn parse_json(file: &String) -> Result<Value,  Box<dyn Error>> {
    let file_contents = &fs::read_to_string(file)?; 
    let json: Value = serde_json::from_str(file_contents)?;

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

pub fn print_json(cmd: CmdArgs) -> Result<(), Box<dyn Error>> {
    let json: Value = parse_json(&cmd.input_file)?;
    if cmd.query.is_some() {
        match parse_cmd(&cmd.query.unwrap())? {
            QueryCmd::MultiArrayIndex(idxs)  => {
                let vals = json.as_array().unwrap();
                for i in idxs {
                    println!("{}", vals[i].to_string());
                }
            }
            QueryCmd::KeywordAccess(keys)  => {
                let string_val = multi_key_access(&json, &keys);
                println!("{}", string_val);
            }
            QueryCmd::MultiCmd(cmds)  => {
                let mut val = & json;
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
                println!("{}", val);
            }

            _ =>  println!("{}", json.to_string())
        }
    } else {
        println!("{}", json.to_string());
    } 
    Ok(())
}