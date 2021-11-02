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


pub fn parse(input: &str) -> Result<QueryCmd, Box<dyn Error>> {
    let mut parsed = JQRParser::parse(Rule::keywordExpr, input)?;
    let parseRes = parsed.next().unwrap();
    println!("parsed: {:?}", parseRes.as_str());



    Ok(QueryCmd::KeywordAccess(vec![parseRes.as_str().to_string()]))
}
