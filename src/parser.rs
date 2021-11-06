use pest::iterators::Pair;
use pest::iterators::Pairs;
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
    FunCallCmd(String, Vec<QueryCmd>),
    FilterCmd(Box<QueryCmd>, String),
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
            (QueryCmd::FilterCmd(c1, s1), QueryCmd::FilterCmd(c2, s2)) => c1 == c2 && s1 == s2,
            (QueryCmd::FunCallCmd(fn1, args1), QueryCmd::FunCallCmd(fn2, args2)) => fn1 == fn2 && args1 == args2,
            (QueryCmd::TransformIntoObject(x_ps), QueryCmd::TransformIntoObject(y_ps)) => {
                x_ps == y_ps
            }
            _ => false,
        }
    }
}

// fn next_res<T>(op: Pair<T>) -> Result<T, Box<dyn Error>> {
//     let err: Box<dyn Error> = String::from("Empty top level parse result").into();
//     op.next().ok_or(err)
// }

fn parse_keyword(expr: Pair<Rule>) -> Result<QueryCmd, Box<dyn Error>>{
    let kws: Vec<String> = expr.into_inner().map(|kw| kw.as_str().to_string()).collect();
    println!("kws: {:?}", kws);
    Ok(QueryCmd::KeywordAccess(kws))
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
        Rule::filterExpr => {
            let mut expr = expr.into_inner();
            let query_expr = expr.next().ok_or(parse_err("filterExpr - invalid queryExpr"))?;
            let val_expr   = expr.next().ok_or(parse_err("filterExpr - invalid valueExpr"))?;
            let unquoted_val = val_expr.into_inner().next().ok_or(parse_err("valueExpr - expected quoted value"))?;


            Ok(QueryCmd::FilterCmd(Box::new(parse_keyword(query_expr)?), unquoted_val.as_str().to_string()))

        },
        Rule::multiExpr => {
            let cmds = expr.into_inner().map(|expr| parse_expr(expr).expect("parseMulti failed"));
            Ok(QueryCmd::MultiCmd(cmds.collect()))
        },
        _ => unreachable!()
      }
}

pub fn parse(input: &str) -> Result<QueryCmd, Box<dyn Error>> {
    let mut parsed = JQRParser::parse(Rule::jqExpr, input)?;
    let parse_res = parsed.next().unwrap();


    let err: Box<dyn Error> = String::from("Empty top level parse result").into();
    let expr = parse_res.into_inner().next().ok_or(err)?;
    println!("parsed: {:?}, expr: {:?}, rule: {:?}", parsed, expr, expr.as_rule());
    parse_expr(expr)
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
        assert_eq!(run_parse("a.b.c"),
                   QueryCmd::KeywordAccess(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        assert_eq!(run_parse("a.b.c"),
                   QueryCmd::KeywordAccess(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        //assert_eq!(parse("a  .b .c").err().is_some(), true);
        assert_eq!(run_parse("a .b   .c"),
                   QueryCmd::KeywordAccess(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        assert_eq!(run_parse("[0]"), QueryCmd::ArrayIndexAccess(vec![0]));
        assert_eq!(parse("[]").err().is_some(), true);
        assert_eq!(run_parse("[1,3, 5]"), QueryCmd::ArrayIndexAccess(vec![1,3,5]));

        assert_eq!(parse("[1,3, ea]").err().is_some(), true);


        assert_eq!(run_parse(".vals"), QueryCmd::ListValues);
        assert_eq!(run_parse(".keys"), QueryCmd::ListKeys);
        assert_eq!(run_parse(".count"), QueryCmd::Count);

        assert_eq!(run_parse("username = \"Adam\""), QueryCmd::FilterCmd(Box::new(
            QueryCmd::KeywordAccess(vec!("username".to_string()))),  "Adam".to_string()));

        assert_eq!(run_parse("address = \"P Sherman 42 Wallaby Way\""), QueryCmd::FilterCmd(Box::new(
            QueryCmd::KeywordAccess(vec!("address".to_string()))),  "P Sherman 42 Wallaby Way".to_string()));

        assert_eq!(run_parse("[230] | a.b"), QueryCmd::MultiCmd(vec![QueryCmd::ArrayIndexAccess(vec![230]),
                                                                     QueryCmd::KeywordAccess(vec!["a".to_string(), "b".to_string()])]));

        assert_eq!(run_parse("[230] | a.b | .vals"),
                   QueryCmd::MultiCmd(vec![
                       QueryCmd::ArrayIndexAccess(vec![230]),
                       QueryCmd::KeywordAccess(vec!["a".to_string(), "b".to_string()]),
                   QueryCmd::ListValues]));
    }
}
