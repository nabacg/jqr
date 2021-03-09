#[macro_use]
extern crate nom;
use parser::QueryCmd;
use serde_json::json;
use serde_json::map::Map;
use serde_json::Value;
use serde_json::Deserializer;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, BufReader};
mod parser;

#[derive(Debug)]
pub struct CmdArgs {
    input_file: Option<String>,
    query: Option<String>,
    flag: Option<String>
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
        Ok(("", cmd)) => {
            //   println!("Cmd={:?}", cmd); // ToDo add a --Debug flag to print it out?
            Ok(cmd)
        }
        Ok((_, cmd)) => {
            //    println!("Cmd={:?} but found unconsumed input={}" , cmd, input_left);
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

fn read_json_from_stdin() -> Result<Value, Box<dyn Error>> {
    let stdin = io::stdin();
    let json:Value = serde_json::from_reader(stdin.lock())?;
    Ok(json)
}

fn read_multi_json_from_stdin() -> Result<Value, Box<dyn Error>> {

    let stdin = io::stdin();
    let json_array: Vec<Value>  = Deserializer::from_reader(stdin.lock())
        .into_iter::<Value>()
        .map(|v| v.unwrap())
        .collect();
    Ok(json!(json_array))

}


fn print_json(val: &Value) {
    if let Ok(s) = serde_json::to_string_pretty(val) {
        println!("{}", s)
    } else {
        println!("{}", val.to_string());
    }
}

fn eval_sub_cmds(val: &Value, arg_cmds: Vec<QueryCmd>) -> Vec<Value> {
    let mut res: Vec<Value> = Vec::new();

    for cmd in arg_cmds {
        let r = eval(val.clone(), cmd); // ToDo this needs fixing this cloning
        res.push(r);
    }
    res
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
                    let mut newProps: Map<String, Value> = Map::new();
                    for j in 0..names.len() {
                        newProps.insert(names[j].clone(), vals[j][i].clone());
                    }
                    res.push(Value::Object(newProps));
                }
                json!(res)
            } else {
                Value::Object(props)
            }
        }
        (json, QueryCmd::FunCallCmd(fun_name, arg_cmds)) => {
            function_registry_lookup(&fun_name, json, arg_cmds)
        }
        (json, QueryCmd::MultiCmd(cmds)) => {
            let mut val = json;
            for cmd in cmds {
                val = eval(val, cmd);
            }
            val
        }
    }
}

fn json_to_num(v: &Value) -> f64 {
    match v {
        n if n.is_i64() =>  n.as_i64().unwrap() as f64,
        n if n.is_u64() =>  n.as_u64().unwrap() as f64,
        n if n.is_f64() =>  n.as_f64().unwrap(),
        _ => panic!(
            "Don't know how to extract Numbers from non numeric Json Value! v={:?} is_f64={}",
            v,
            v.is_f64()
        ),
    }
}

fn flatten_json_to_num_array(json: Value, cmds: Vec<QueryCmd>) -> Vec<f64> {
    let vs: Vec<Value> = eval_sub_cmds(&json, cmds);
    let res: Vec<f64> =
     vs.iter().flat_map(|v| match v {
        Value::Array(vs) => vs.iter().map(|v| json_to_num(&v)).collect(),
        _ => vec![json_to_num(&v)]

    }).collect();
    res
}

fn sum_json_nums(json: Value, cmds: Vec<QueryCmd>) -> Value {
    let res = flatten_json_to_num_array(json, cmds).iter().fold(0.0, |acc, n| acc + n);
    json!(res)
}

fn max_json_nums(json: Value, cmds: Vec<QueryCmd>) -> Value {
    let mut nums = flatten_json_to_num_array(json, cmds);
    // note a, b flipped, so it's a DESC sort
    nums.sort_by(|a, b| b.partial_cmp(a).unwrap());
    json!(nums[0])
}

fn min_json_nums(json: Value, cmds: Vec<QueryCmd>) -> Value {
    let mut nums = flatten_json_to_num_array(json, cmds);
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
   json!(nums[0])
}

fn avg_json_nums(json: Value, cmds: Vec<QueryCmd>) -> Value {
    let nums = flatten_json_to_num_array(json, cmds);
    let sum = nums.iter().fold(0.0, |acc, v| acc + v);
    println!("sum = {}, len = {}, avg = {}", sum,  nums.len(), sum / (nums.len() as f64));
    json!((sum / (nums.len() as f64)))
}

fn sort_json_nums(json: Value, cmds: Vec<QueryCmd>) -> Value {
    let mut nums = flatten_json_to_num_array(json, cmds);
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    json!(nums)
}

fn group_by_cmd(json:Value, cmds:Vec<QueryCmd>) -> Value {
    if cmds.len() != 2 {
        panic!("groupBy can only be called with 2 arguments, a value to group and a expression to it groupBy")
    }

    let cmd_to_group = cmds[0].clone();
    let cmd_to_group_by = cmds[1].clone();

    let value_to_group = eval(json, cmd_to_group);

    match value_to_group {
        Value::Array(values) => {
            // extract a function here, similar code in eval branch   (json, QueryCmd::TransformIntoObject(prop_mapping))

          //  let mut group_dict: HashMap<String, >

            let mut props: Map<String, Value> = Map::new();
            // there has to be a better way to do this, not only does the code look horrible
            // but the performance seems to be really bad
            // below groupBy takes 60secs !!
            //  58.60s user 0.60s system 99% cpu 59.393 total
            //time cargo run 004ff2c5-7ed0-433b-8638-e6ceeceb1d09-7 "Records | [0] | groupBy(Details, Grouping.CampaignId) | .keys  "
//             [
//   "1337887",
//   "1382752",
//   "1438498",
//   "1441118",
//   "1455062",
//   "1490109",
//   "1491444",
//   "1495891",
//   "1496524",
//   "1496561",
//   "1498116",
//   "1502539",
//   "1502608",
//   "548278",
//   "608806",
//   "618697",
//   "817940"
// ]
// cargo run 004ff2c5-7ed0-433b-8638-e6ceeceb1d09-7
            // maybe use HashMap and mutable Vectors as vals ? https://doc.rust-lang.org/rust-by-example/std/hash.html
            for json_val in values {
                let group_key = eval(json_val.clone(), cmd_to_group_by.clone()); // Todo this cloning sucks! Can I do lifetimes to limit this?
                let key_string = serde_json::to_string(&group_key).unwrap();
                match props.get(&key_string) {
                    Some(Value::Array(vs)) =>  {

                        let mut new_vec = vs.clone();
                        new_vec.push(json_val);
                        props.insert(key_string, json!(new_vec));

                        }
                    None => { props.insert(key_string, json!(vec![json_val])); }
                    v => panic!("Group by return value adding to previous group should always find a Vec<Value>, but found something else: {:?}", v)
                }

            }
            json!(props)
        }
        json_val => {
            let group_key = eval(json_val.clone(), cmd_to_group_by.clone());
            let key_string = serde_json::to_string(&group_key).unwrap();
            json!({key_string: json_val})

        }

    }
}

// Apparently you need to impl Fn trait https://stackoverflow.com/a/38947708
//-> Vec<Value> -> Value {
// ((Vec<Value>) -> Value)
fn function_registry_lookup(fn_name: &str, json: Value, cmds: Vec<QueryCmd>) -> Value {
    match fn_name {
        "sum"  => sum_json_nums(json, cmds),
        "max"  => max_json_nums(json, cmds),
        "min"  => min_json_nums(json, cmds),
        "avg"  => avg_json_nums(json, cmds),
        "sort" => sort_json_nums(json, cmds),
        "groupBy" => group_by_cmd(json, cmds),
        _ => panic!(format!("Function {}! not supported yet!", fn_name)),
    }
}

fn eval_inner(json: Value, query: QueryCmd) {
     println!("command found = {:?}", query);
    let res_json = eval(json, query);
    print_json(&res_json)
}

pub fn eval_cmd(cmd: CmdArgs) -> Result<(), Box<dyn Error>> {
    println!("eval_cmd: {:?}", cmd);
    let json: Value = match (&cmd.input_file, &cmd.flag) {
        (Some(input_path), None) => read_json_file(&input_path)?,
        (Some(input_path), Some(flag)) => read_json_file(&input_path)?,
        (None, Some(flag)) => read_multi_json_from_stdin()?,
        (None, None) => read_json_from_stdin()?,
    };

    match cmd.query.map(|query| parse_cmd(&query)) {
        Some(Ok(cmd)) => eval_inner(json, cmd),
        Some(Err(msg)) => println!("Failed at cmd parsing with error= {}", msg), // Seems like too many levels of error handling
        None => print_json(&json),
    }
    Ok(())
}
