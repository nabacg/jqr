#![allow(deprecated)]
extern crate pest;
#[macro_use]
extern crate pest_derive;

use serde_json::Value::Number;
use parser::QueryCmd;
use serde_json::{json};
use serde_json::map::Map;
use serde_json::Value;
use serde_json::Deserializer;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader};
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
        (Value::Array(vs), cmd @ QueryCmd::FilterCmd(_, _, _ )) => {
            let mut res: Vec<Value> = Vec::new();
            for v in vs {
                let r = apply_filter(v,cmd.clone()); // ToDo this needs fixing this cloning
                if r != json!("") {
                    res.push(r);
                }

            }
            json!(res)
        }
        (json, f @ QueryCmd::FilterCmd(_, _, _)) => {
            apply_filter(json, f)
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

fn eval_and_print(json: Value, query: QueryCmd) {
    let res_json = eval(json, query);
    print_json(&res_json)

}

fn apply_filter(candidate:Value, filter_cmd: QueryCmd) -> Value {
    if let QueryCmd::FilterCmd(cmd, op, value) = filter_cmd {
        match eval(candidate.clone(), *cmd.clone()) {
            Number(n) if op == "=" &&  n == value.parse().unwrap() => json!(candidate),
            // TODO seems like a classic case of multiple dispatch, extract into separate function, maybe in a trait?
            Number(n) if op == ">" &&  n.is_i64() && n.as_i64().unwrap() > value.parse().unwrap() => json!(candidate),
            Number(n) if op == ">" &&  n.is_f64() && n.as_f64().unwrap() > value.parse().unwrap() => json!(candidate),
            Number(n) if op == "<" &&  n.is_i64() && n.as_i64().unwrap() < value.parse().unwrap() => json!(candidate),
            Number(n) if op == "<" &&  n.is_f64() && n.as_f64().unwrap() < value.parse().unwrap() => json!(candidate),
            serde_json::Value::String(s) if s == value => json!(candidate),
            _ => json!("")
        }
    } else {
        json!("")
    }

}

fn apply_consecutive_filters(candidate:Value, cmds: Vec<QueryCmd> ) -> (Value, Vec<QueryCmd>) {

    // let filter_pred = |c:QueryCmd| matches!(c, QueryCmd::FilterCmd(_, _, _));
    let mut v = candidate;
    let rest : Vec<QueryCmd> = cmds.iter().skip_while(|c| matches!(c, QueryCmd::FilterCmd(_, _, _))).map(|c| c.clone()).collect();
    for cmd in cmds.iter().take_while(|c| matches!(c, QueryCmd::FilterCmd(_, _, _))) {
        v = apply_filter(v, cmd.clone())
    }
    (v, rest)

}

//https://stackoverflow.com/a/47606476
fn streaming_eval(json_iter: impl Iterator<Item = Value>, query: QueryCmd) -> Result<(), Box<dyn Error>> {
    let empty_json = json!("");
     match &query {
        QueryCmd::ArrayIndexAccess(idx) =>
             json_iter
             .enumerate()
             .filter(|(i, _)| idx.contains(&i) )
             .take(idx.len()).map(|(_, jv)| print_json(&(jv)))
             .collect(),
        f @ QueryCmd::FilterCmd(_, _, _) => {
                json_iter.map(|json| apply_filter(json, f.clone()))
                 .filter(|jv| *jv != empty_json)
                 .map(|j| print_json(&j))
                 .for_each(drop)
        },
        QueryCmd::MultiCmd(cmds) => {
            match &cmds[0] {
                QueryCmd::ArrayIndexAccess(idx) => {
                    json_iter
                    .enumerate()
                    .filter(|(i, _)| idx.contains(&i))
                    .take(idx.len())
                    .map(|(_, json)| json)
                    .map(|json| apply_consecutive_filters(json, cmds.to_vec()))
                    .filter(|(jv, _)| *jv != empty_json)
                    .map(|(jv, cmds)|
                        if cmds.len() > 0 {
                            eval_and_print(jv, QueryCmd::MultiCmd(cmds[1..].to_vec()))
                        } else {
                            print_json(&jv)
                        })
                    .collect()
                },
                QueryCmd::FilterCmd(_, _, _)=>  {
                        json_iter
                        .map(|json| apply_consecutive_filters(json, cmds.to_vec()))
                        .filter(|(jv, _)| *jv != empty_json)
                        .map(|(jv, cmds)|
                            if cmds.len() > 0 {
                                eval_and_print(jv, QueryCmd::MultiCmd(cmds))
                            } else {
                                print_json(&jv)
                            })
                        .collect()
                },
                //  f @ QueryCmd::FilterCmd(_, _, _)=>  {
                //     let empty_json = json!("");

                //     json_iter
                //     .map(|json| apply_filter(json, f.clone()))  // TODO it's really here that's you'd want to apply all applicable cmds, transducer style
                //     .filter(|jv| *jv != empty_json)             // i.e. if there are several filters (or ArrayIndexAccess), we could apply them one after other
                //     .map(| jv|
                //         eval_and_print(jv, QueryCmd::MultiCmd(cmds[1..].to_vec())))
                //     .collect()
                // },
                _cmd =>  {
                    // json_iter
                    //     .map(|jv|
                    //          eval_and_print(jv, QueryCmd::MultiCmd(cmds.clone())))
                    //     .collect()

                    // let mut sliced_json = json_iter.map(|jv|
                    //     eval(jv, cmd.clone()));

                    // for cmd in cmds[1..].to_vec() {
                    //     sliced_json =  sliced_json.map(|jv|
                    //         eval(jv, cmd.clone()));
                    // }


                    // TODO Finish this, last time I tried it, it put rustc into infinite loop!
                    // let sliced_json = json_iter.map(|jv|  eval(jv, _cmd.clone()));
                    // let rest_cmds = cmds[1..].to_vec();
                    // if rest_cmds.len() > 0 {
                    //      streaming_eval(sliced_json, QueryCmd::MultiCmd(rest_cmds))?;
                    // }
                    // ()



                    let mut sliced_json = json_iter.collect::<Vec<Value>>();
                    for cmd in cmds {
                        sliced_json = sliced_json
                        .iter()
                        .map(|jv|  eval(jv.clone(), cmd.clone()))
                        .collect::<Vec<Value>>();
                    }
                }
            }
        },
        query => {
            json_iter.map(|jv| eval_and_print(jv, query.clone())).collect()
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
            let json_iter  = Deserializer::from_reader(rdr).into_iter::<Value>().map(|v|v.unwrap());
            streaming_eval(json_iter, cmd)?;
            ()
        }
        (Some(input_file), Some(Ok(cmd))) => {
            let file = File::open(input_file)?;
            let json_iter  = Deserializer::from_reader(BufReader::new(file)).into_iter::<Value>().map(|v|v.unwrap());

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
            Deserializer::from_reader(BufReader::new(file))
                .into_iter::<Value>()
                .map(|jv| print_json(&(jv.unwrap())))
                .for_each(drop);
            ()
        }

    };
    Ok(())
}
