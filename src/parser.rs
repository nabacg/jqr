use pest::iterators::Pair;
use std::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "jqr.pest"]
pub struct JQRParser;



#[derive(Debug, Eq, Clone)]
pub enum QueryCmd {
    ArrayIndexAccess(Vec<usize>),
    KeywordAccess(Vec<String>),
    MultiCmd(Vec<QueryCmd>),
    TransformIntoObject(Vec<(String, QueryCmd)>),
    FilterCmd(Box<QueryCmd>, String, String),
    ListKeys,
    ListValues,
    Count
}

impl PartialEq for QueryCmd {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (QueryCmd::ArrayIndexAccess(xs), QueryCmd::ArrayIndexAccess(ys)) => xs == ys,
            (QueryCmd::KeywordAccess(xs), QueryCmd::KeywordAccess(ys)) => xs == ys,
            (QueryCmd::MultiCmd(xs), QueryCmd::MultiCmd(ys)) => xs == ys,
            (QueryCmd::ListKeys, QueryCmd::ListKeys) => true,
            (QueryCmd::ListValues, QueryCmd::ListValues) => true,
            (QueryCmd::Count, QueryCmd::Count) => true,
            (QueryCmd::FilterCmd(c1, op1, v1), QueryCmd::FilterCmd(c2, op2, v2)) => c1 == c2 && op1 == op2 && v1 == v2,
            (QueryCmd::TransformIntoObject(x_ps), QueryCmd::TransformIntoObject(y_ps)) => {
                x_ps == y_ps
            }
            _ => false,
        }
    }
}

impl QueryCmd {
    fn keyword_access(kws: &[&str]) -> QueryCmd {
        QueryCmd::KeywordAccess(kws.iter().map(|k| k.to_string()).collect())
    }

    fn filter_cmd(f: QueryCmd, op: &str, v: &str) -> QueryCmd {
        QueryCmd::FilterCmd(Box::new(f), op.to_string(), v.to_string())
    }
}

// fn next_res<T>(op: Pair<T>) -> Result<T, Box<dyn Error>> {
//     let err: Box<dyn Error> = String::from("Empty top level parse result").into();
//     op.next().ok_or(err)
// }

fn parse_keyword(expr: Pair<Rule>) -> Result<QueryCmd, Box<dyn Error>> {

    let kws: Vec<_> = expr.into_inner().map(|kw| kw.as_str()).collect();
    //println!("kws: {:?}", kws);
    Ok(QueryCmd::keyword_access(&kws))
}

fn parse_err(msg: &str) -> Box<dyn Error> {
    String::from(msg).into()
}

fn parse_expr(expr: Pair<Rule>) -> Result<QueryCmd, Box<dyn Error>> {
      match expr.as_rule() {
        Rule::valsExpr  => Ok(QueryCmd::ListValues),
        Rule::keysExpr  => Ok(QueryCmd::ListKeys),
        Rule::countExpr => Ok(QueryCmd::Count),
        Rule::multiKeyword =>  {
            parse_keyword(expr)
        },
        Rule::indexAccess => {
            let idx: Vec<usize> = expr.into_inner().map(|kw| kw.as_str().parse::<usize>().unwrap()).collect();
            Ok(QueryCmd::ArrayIndexAccess(idx))
        },
        Rule::rangeIndexAccess => {
            let idx_range: Vec<usize> = expr.into_inner().map(|e| e.as_str().parse::<usize>().unwrap()).collect();
            if idx_range.len() != 2 {
                panic!("expected [from..to], got: {:?}", idx_range)
            }

            Ok(QueryCmd::ArrayIndexAccess((idx_range[0]..idx_range[1]).collect()))
        },
        Rule::filterExpr => {
            let mut expr = expr.into_inner();
            let query_expr = expr.next().ok_or(parse_err("filterExpr - invalid queryExpr"))?;
            let op_expr    = expr.next().ok_or(parse_err("filterExpr - invalid operatorExpr"))?;
            let val_expr   = expr.next().ok_or(parse_err("filterExpr - invalid valueExpr"))?;
 
            Ok(QueryCmd::filter_cmd(parse_keyword(query_expr)?,
                                   op_expr.as_str(),
                                   val_expr.as_str()))

        },
        Rule::multiExpr => {
            let cmds = expr.into_inner().map(|expr| parse_expr(expr).expect("parseMulti failed"));
            Ok(QueryCmd::MultiCmd(cmds.collect()))
        },
          Rule::newObjExpr => {
              let properties = expr.into_inner().map(|e| {
                  let mut args = e.into_inner();
                  let prop_name = args.next().unwrap().as_str().to_string(); //todo handler errors better
                  let prop_value = parse_expr(args.next().unwrap()).unwrap();
                  (prop_name, prop_value)

              });
              Ok(QueryCmd::TransformIntoObject(properties.collect()))
          }
        _ => unreachable!()
      }
}

pub fn parse(input: &str) -> Result<QueryCmd, Box<dyn Error>> {
    let mut parsed = JQRParser::parse(Rule::jqExpr, input)?;

    let err: Box<dyn Error> = String::from("Empty top level parse result").into();
    let expr = parsed.next().ok_or(err)?;
    println!("parsed: {:?}\n, expr: {:?}\n, rule: {:?}", parsed, expr, expr.as_rule());
    let cmd = parse_expr(expr);
    println!("QueryCmd: {:?}", cmd);
    cmd
}


#[cfg(test)]
mod parser_test {
    use super::*;

    fn run_parse(s: &str) -> QueryCmd {

        parse(s).expect("Parse failed")
    }

    #[test]
    fn parse_test() {
        assert_eq!(run_parse("a"), QueryCmd::KeywordAccess(vec!["a".to_string()]));
        assert_eq!(run_parse("a.b.c"), QueryCmd::keyword_access(&["a", "b", "c"]));


        //assert_eq!(parse("a  .b .c").err().is_some(), true);
        assert_eq!(run_parse("a .b   .c"),
                   QueryCmd::KeywordAccess(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        assert_eq!(run_parse("[0]"), QueryCmd::ArrayIndexAccess(vec![0]));
        assert_eq!(parse("[]").err().is_some(), true);
        assert_eq!(run_parse("[1,3, 5]"), QueryCmd::ArrayIndexAccess(vec![1,3,5]));
        assert_eq!(run_parse("[11..15]"), QueryCmd::ArrayIndexAccess(vec![11, 12, 13, 14]));
        assert_eq!(run_parse("[15..11]"), QueryCmd::ArrayIndexAccess(vec![]));
        assert_eq!(run_parse("[15..15]"), QueryCmd::ArrayIndexAccess(vec![]));


        assert_eq!(parse("[1,3, ea]").err().is_some(), true);


        assert_eq!(run_parse(".vals"), QueryCmd::ListValues);
        assert_eq!(run_parse(".keys"), QueryCmd::ListKeys);
        assert_eq!(run_parse(".count"), QueryCmd::Count);

        assert_eq!(run_parse("username = \"Adam\""), QueryCmd::filter_cmd(
            QueryCmd::keyword_access(&["username"]), "=",  "Adam"));

        assert_eq!(run_parse("address = \"P Sherman 42 Wallaby Way\""), QueryCmd::filter_cmd(
            QueryCmd::keyword_access(&["address"]), "=", "P Sherman 42 Wallaby Way"));
        

        assert_eq!(run_parse("[230] | a.b"),
                   QueryCmd::MultiCmd(vec![QueryCmd::ArrayIndexAccess(vec![230]),
                                           QueryCmd::keyword_access(&["a", "b"])]));

        assert_eq!(run_parse("[230] | a.b | .vals"),
                   QueryCmd::MultiCmd(vec![
                       QueryCmd::ArrayIndexAccess(vec![230]),
                       QueryCmd::keyword_access(&["a", "b"]),
                       QueryCmd::ListValues]));

        assert_eq!(run_parse("{ a := xyz; b := testExpr.Abc }"),
                   QueryCmd::TransformIntoObject(vec![
                       (
                           "a".to_string(),
                           QueryCmd::keyword_access(&["xyz"])
                       ),
                       (
                           "b".to_string(),
                           QueryCmd::keyword_access(&["testExpr", "Abc"])
                       )
                   ]));

        assert_eq!(run_parse("Records | [0] | Details | [0]  | LastDate = \"2020-04-24T00:00:00Z\""),
                   QueryCmd::MultiCmd(vec![
                       QueryCmd::keyword_access(&["Records"]),
                       QueryCmd::ArrayIndexAccess(vec![0]),
                       QueryCmd::keyword_access(&["Details"]),
                       QueryCmd::ArrayIndexAccess(vec![0]),
                       QueryCmd::filter_cmd(QueryCmd::keyword_access(&["LastDate"]),
                                           "=",
                                           "2020-04-24T00:00:00Z")
                   ]));

        assert_eq!(run_parse("LastDate = \"2020\""),
                   QueryCmd::filter_cmd(QueryCmd::keyword_access(&["LastDate"]), "=", "2020"));

        assert_eq!(run_parse("Clicks = 7"),
                   QueryCmd::filter_cmd(QueryCmd::keyword_access(&["Clicks"]), "=", "7"));

        assert_eq!(run_parse("Clicks > 0"),
                   QueryCmd::filter_cmd(QueryCmd::keyword_access(&["Clicks"]), ">", "0"));

        assert_eq!(run_parse("CTR < 0.1"),
                   QueryCmd::filter_cmd(QueryCmd::keyword_access(&["CTR"]), "<", "0.1"));
        


    }
}
