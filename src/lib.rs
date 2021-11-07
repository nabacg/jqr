#![allow(deprecated)]
extern crate pest;
#[macro_use]
extern crate pest_derive;

use serde_json::Value::Number;
use parser::QueryCmd;
use serde_json::{StreamDeserializer, json};
use serde_json::map::Map;
use serde_json::Value;
use serde_json::Deserializer;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::io::{Write};
mod parser;

#[derive(Debug)]
pub struct CmdArgs {
    input_file: Option<String>,
    query: Option<String>,
    flag: Option<String> // this flag should be an enum actually!
}

impl CmdArgs {
    pub fn new(args: &[String]) -> Result<CmdArgs, String> {
        match args {
            [_, opt, query] if opt.as_str() == "-m" => Ok(CmdArgs {
                input_file: None,
                query: Some(query.clone()),
                flag: Some(opt.clone())
            }),
            [_, input_file, query] => Ok(CmdArgs {
                input_file: Some(input_file.clone()),
                query: Some(query.clone()),
                flag: None
            }),
            [_, query] => Ok(CmdArgs {
                input_file: None,
                query: Some(query.clone()),
                flag: None
            }),
            [_] => Ok(CmdArgs {
                input_file: None,
                query: None,
                flag: None
            }),
            _ => {
                return Err(format!(
                    "Wrong number of arguments passed, jqr expects 0, 1 or 2 args. Passed= {}",
                    args.len()
                ))
            }
        }
    }
}

fn parse_cmd(cmd_str: &String) -> Result<QueryCmd, &'static str> {
    match parser::parse(&cmd_str) {
        Ok(cmd) => {
            //   println!("Cmd={:?}", cmd); // ToDo add a --Debug flag to print it out?
            Ok(cmd)
        }
        Err(e) => {
            eprintln!("ERROR parsing cmd={:?} error={:?}", cmd_str, e);
            Err("Failing now")
        }
    }
}

pub fn read_json_file(file: &String) -> Result<Value, Box<dyn Error>> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    Ok(json)
}

fn print_json(val: &Value) {
    if let Ok(s) = serde_json::to_string_pretty(val) {
        println!("{}", s)
    } else {
        println!("{}", val.to_string());
    }
}


fn write_json(val: &Value) -> io::Result<()> {
    let mut stdout = io::stdout();
    if let Ok(s) = serde_json::to_string_pretty(val) {
        writeln!(&mut stdout, "{}", s)?
    } else {
        writeln!(&mut stdout, "{}", val.to_string())?

    }
    Ok(())
}

// ideally query should be an immutable ref, i.e. &QueryCmd but then we can't pattern match on both (json, query) because of Rust reasons..
// but it would be great to solve it to avoid cloing QueryCmd on each recursive call
// ToDo - could I use lifetimes to avoid cloning QueryCmd here?
fn eval(json: Value, query: QueryCmd) -> Value {
    match (json, query) {
        (v @ Value::Null, _) => v,
        (v @ Value::Bool(_), _) => v,
        (v @ Value::Number(_), _) => v,
        (v @ Value::String(_), _) => v,
        (Value::Array(vs), QueryCmd::ArrayIndexAccess(idxs)) => {
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
        (v @ Value::Array(_), QueryCmd::ListValues) => v,
        (Value::Array(arr), QueryCmd::ListKeys) => {
            let indices: Vec<usize> = (0..arr.len()).collect();
            json!(indices)
        }
        (Value::Array(arr), QueryCmd::Count) => json!(arr.len()),
        (Value::Object(obj), QueryCmd::Count) => json!(obj.len()),
        (Value::Array(vs), cmd @ QueryCmd::KeywordAccess(_)) => {
            let mut res: Vec<Value> = Vec::new();
            for v in vs {
                let r = eval(v, cmd.clone()); // ToDo this needs fixing this cloning
                res.push(r);
            }
            json!(res)
        }
        (v @ Value::Object(_), QueryCmd::ArrayIndexAccess(_)) => panic!(format!(
            "Cannot perform Array index access on an object! Json Found= {}",
            serde_json::to_string_pretty(&v).unwrap()
        )),
        (v @ Value::Object(_), QueryCmd::KeywordAccess(keys)) => {
            let mut val = &v;
            for k in keys {
                val = &val[k];
            }
            json!(*val)
        }
        (json, QueryCmd::TransformIntoObject(prop_mapping)) => {
            let mut props: Map<String, Value> = Map::new();

            for (prop_name, prop_access_cmd) in prop_mapping {
                let prop_val = eval(json.clone(), prop_access_cmd); // Todo this cloning sucks! Can I do lifetimes to limit this?
                props.insert(prop_name, prop_val);
            }

            if props.iter().all(|(_, v)| v.is_array()) {
                let vals: Vec<&Value> = props.values().collect(); // props.iter().map(|(_, v)| v).collect();
                let names: Vec<&String> = props.keys().collect();

                let shortest: usize = vals
                    .iter()
                    .map(|v| v.as_array().unwrap().len())
                    .max()
                    .unwrap();

                let mut res: Vec<Value> = Vec::new();
                for i in 0..shortest {
                    let mut new_props: Map<String, Value> = Map::new();
                    for j in 0..names.len() {
                        new_props.insert(names[j].clone(), vals[j][i].clone());
                    }
                    res.push(Value::Object(new_props));
                }
                json!(res)
            } else {
                Value::Object(props)
            }
        }
        (json, QueryCmd::FilterCmd(query, value)) => {
            let query_dbg = query.clone();
            let res = eval(json.clone(), *query);
            println!("Query inner filter={:?}, value={:?}, res={:?}", query_dbg, json!(value), &res );
            //TODO we should really cover all json types here properly
            match res {
                Number(n) if  n == value.parse().unwrap() => json!(json),
                serde_json::Value::String(s) if s == value => json!(json),
                _ => json!("")

            }

        }
        (json, QueryCmd::MultiCmd(cmds)) => {
            let mut val = json;
            for cmd in cmds {
                // println!("{:?}", cmd);
                val = eval(val, cmd);
            }
            val
        }
    }
}

fn eval_outer(json: Value, query: QueryCmd) {
    let res_json = eval(json, query);
    let write_res = write_json(&res_json);
    if let Ok(_res) =  write_res {

    } else {
        println!("write_json error{:?}", write_res)
    }

}

fn streaming_eval<I: Read>(json_iter: StreamDeserializer<serde_json::de::IoRead<I>, Value>, query: QueryCmd) -> Result<(), Box<dyn Error>> {
     match &query {
        QueryCmd::ArrayIndexAccess(idx) =>
             json_iter
             .enumerate()
             .filter(|(i, _)| idx.contains(&i) )
             .take(idx.len()).map(|(_, jv)| print_json(&(jv.unwrap())))
             .collect(),
        QueryCmd::MultiCmd(cmds) => {
            match &cmds[0] {
                QueryCmd::ArrayIndexAccess(idx) =>
                    json_iter
                    .enumerate()
                    .filter(|(i, _)| idx.contains(&i) )
                    .take(idx.len())
                    .map(|(_, jv)|
                         eval_outer(jv.unwrap(), QueryCmd::MultiCmd(cmds[1..].to_vec())))
                    .collect(),
                _ =>  {
                    json_iter
                        .map(|jv|
                             eval_outer(jv.unwrap(), QueryCmd::MultiCmd(cmds.clone())))
                        .collect()
                }
            }
        },
         QueryCmd::FilterCmd(cmd, val) => {
             let json_val = json!(val);
             let empty_json = json!("");

             json_iter.map(|json|
                           {
                               println!("filter cmd={:?}", cmd);
                             //  println!("json={:?}", json);
                               let json2 = json.unwrap();
                               let res = eval(json2.clone(), *cmd.clone());
                               //println!("Filter res={:?}", res);
                               if json_val == res {
                                   json2
                               } else {
                                   empty_json.clone()
                               }
                           }
             ).filter(|jv| *jv != empty_json.clone())
              .map(|j| write_json(&j))
              .for_each(drop)

         }
        query => {
            json_iter.map(|jv| eval_outer(jv.unwrap(), query.clone())).collect()
        }
    }

    Ok(())

}

pub fn eval_cmd(cmd: CmdArgs) -> Result<(), Box<dyn Error>> {

    match (&cmd.input_file, cmd.query.map(|query| parse_cmd(&query))) {
        (_, Some(Err(msg))) => println!("Failed at cmd parsing with error= {}", msg),
        (None, Some(Ok(cmd))) => {

            let std_in = io::stdin();
            //let reader = Box::new(std_in.lock()) as Box<BufRead>;
            let rdr = std_in.lock();
            let json_iter  = Deserializer::from_reader(rdr).into_iter::<Value>();
            streaming_eval(json_iter, cmd)?;
            ()
        }
        (Some(input_file), Some(Ok(cmd))) => {
            let file = File::open(input_file)?;
            let json_iter  = Deserializer::from_reader(BufReader::new(file)).into_iter::<Value>();

            streaming_eval(json_iter, cmd)?;
            ()
        },
        (None, None) => {
            let stdin = io::stdin();
            Deserializer::from_reader(stdin.lock()).into_iter::<Value>().map(|jv| print_json(&(jv.unwrap()))).for_each(drop);
            ()
        }
        (Some(input_file), None) => {
            let file = File::open(input_file)?;
            Deserializer::from_reader(BufReader::new(file)).into_iter::<Value>().map(|jv| print_json(&(jv.unwrap()))).for_each(drop);
            ()
        }

    };
    Ok(())
}
