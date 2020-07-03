#[macro_use]
extern crate nom;
use std::fs;
use std::error::Error;
use serde_json::{ Value};
use serde_json::map::{ Map};
use parser::QueryCmd;
use std::io::{self, Read};
use serde_json::json;
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

fn print_json(val:&Value) {
    if let Ok(s) = serde_json::to_string_pretty(val) {
        println!("{}", s)
    } else {
        println!("{}", val.to_string());
    }
}

// ideally query should be an immutable ref, i.e. &QueryCmd but then we can't pattern match on both (json, query) because of Rust reasons.. 
// but it would be great to solve it to avoid cloing QueryCmd on each recursive call
fn eval(json:Value, query: QueryCmd) -> Value { 
    match (json, query) {
        (v@Value::Null, _)       =>  v, 
        (v@Value::Bool(_), _)    =>  v, 
        (v@Value::Number(_), _)  =>  v, 
        (v@Value::String(_), _)  =>  v, 
        (Value::Array(vs),   QueryCmd::ArrayIndexAccess(idxs))  => {
            if idxs.len() == 1 {
                json!(vs[idxs[0]])
            } else {
                let mut arr = vec![];
            
                for i in idxs {
                    arr.push(&vs[i]);
                }
                json!(arr)
            }

        }
        (Value::Object(o), QueryCmd::ListKeys) => {
            let keys: Vec<&String> = o.keys().collect();
            json!(keys)
        }
        (Value::Object(o), QueryCmd::ListValues) => {
            let keys: Vec<&Value> = o.values().collect();
            json!(keys)
        }
        (v@Value::Array(_), QueryCmd::ListValues) => {
            v
        }
        (Value::Array(arr), QueryCmd::ListKeys) => {
            let indices: Vec<usize> = (0..arr.len()).collect();
            json!(indices)
        }
        (Value::Array(vs),   cmd@QueryCmd::KeywordAccess(_))   => {
            let mut res:Vec<Value> = Vec::new();
            for v in vs {
                let r = eval(v, cmd.clone()); // ToDo this needs fixing this cloning
                res.push(r); 
            }
            json!(res)
        } 
        (v@Value::Object(_),  QueryCmd::ArrayIndexAccess(_))   => panic!(format!("Cannot perform Array index access on an object! Json Found= {}", serde_json::to_string_pretty(&v).unwrap())),
        (v@Value::Object(_),  QueryCmd::KeywordAccess(keys))  => {
            let mut val = &v;
            for k in keys {
                val = &val[k];
            }
            json!(*val)
        }
        (json, QueryCmd::TransformIntoObject(prop_mapping)) => {
            let mut props:Map<String,Value> = Map::new();

            for (prop_name, prop_access_cmd) in prop_mapping {
                let prop_val = eval(json.clone(), prop_access_cmd); // Todo this cloning sucks!
                props.insert(prop_name, prop_val);
            }

            Value::Object(props)
        }
        (json,  QueryCmd::MultiCmd(cmds))   => {
            let mut val = json; 
            for cmd in cmds {
                val = eval(val, cmd);
            }
            val
        }
    }
}

fn eval_inner(json:Value, query: QueryCmd) {
   let res_json = eval(json, query);
   print_json(&res_json)
}

pub fn eval_cmd(cmd: CmdArgs) -> Result<(), Box<dyn Error>> {
    let json: Value = match &cmd.input_file {
        Some(input_path) => read_json_file(&input_path)?,
        None             => read_json_from_stdin()?
    };
    
    match cmd.query.map(|query| parse_cmd(&query)) {
        Some(Ok(cmd))   => eval_inner(json, cmd), 
        Some(Err(msg))  => println!("Failed at cmd parsing with error= {}", msg), // Seems like too many levels of error handling
        None            => print_json(&json)
    }
    Ok(())
}