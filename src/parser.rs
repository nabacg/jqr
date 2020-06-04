use nom::{
  IResult,
  sequence::{delimited, tuple},
  bytes::complete::{tag, is_not, take_while1, take_while_m_n},
  combinator::{map_res, map},
  // see the "streaming/complete" paragraph lower for an explanation of these submodules
  character::complete::char
};
use nom::multi::separated_list;
use nom::character::complete::{alpha1, digit1};
use nom::branch::alt;

#[derive(Debug, Eq)]
pub enum QueryCmd {
    Unknown,
    MultiArrayIndex(Vec<usize>),
    KeywordAccess(Vec<String>),

}

impl PartialEq for QueryCmd {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (QueryCmd::Unknown, QueryCmd::Unknown) => true,
            //(QueryCmd::SingleArrayIndex(i), QueryCmd::SingleArrayIndex(j)) => i == j,
            (QueryCmd::MultiArrayIndex(xs), QueryCmd::MultiArrayIndex(ys)) => xs == ys,
            (QueryCmd::KeywordAccess(xs), QueryCmd::KeywordAccess(ys)) => xs == ys,
            _ => false
        }
    }
}
// from map_res
/// let parse = map_res(digit1, |s: &str| s.parse::<u8>());
///
/// // the parser will convert the result of digit1 to a number
/// assert_eq!(parse("123"), Ok(("", 123)));
///
// let is_num = take_while1(is_digit)
//https://stackoverflow.com/questions/54735672/how-do-i-use-nom-to-parse-a-string-with-sign-into-an-i32
fn int_list(s: &str) -> IResult<&str, Vec<usize>> {
    separated_list(tag(","), map_res(digit1, |s: &str| s.parse::<usize>()))(s)
}

fn string_list(s: &str) -> IResult<&str, Vec<String>> {
    separated_list(tag("."), map(alpha1, |s: &str| s.to_string()))(s)
}


fn single_int(s: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(s)
}

fn array_index_access(s: &str) -> IResult<&str, QueryCmd> {
    let (input, (_, xs, _)) = tuple((char('['), int_list, char(']')))(s)?;
    Ok((input, QueryCmd::MultiArrayIndex(xs)))
}

fn keyword_access(s: &str) -> IResult<&str, QueryCmd> {
    let (input, (_, xs, _)) = tuple((char('{'), string_list, char('}')))(s)?;
    Ok((input, QueryCmd::KeywordAccess(xs)))
}

// combinator list
// https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md

//ToDo
// handle spaces in int_list like [1, 2, 3 ]
// https://docs.rs/nom/5.1.1/nom/
pub fn parse(input: &str) -> IResult<&str, QueryCmd> {

    alt((array_index_access, keyword_access))(input)

}


#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn int_list_test() {
        assert_eq!(int_list(&""),      Ok(("", vec![])));
        assert_eq!(int_list(&"1,2,3"), Ok(("", vec![1,2,3])));
        assert_eq!(int_list(&"1"),     Ok(("", vec![1])));
    }


    #[test]
    fn square_bracket_array_test() {
        assert_eq!(array_index_access(&"[]"),      Ok(("", QueryCmd::MultiArrayIndex(vec![]))));
        assert_eq!(array_index_access(&"[1,2,3]"), Ok(("", QueryCmd::MultiArrayIndex(vec![1,2,3]))));
        assert_eq!(array_index_access(&"[1]"),     Ok(("", QueryCmd::MultiArrayIndex(vec![1]))));
    }


    #[test]
    fn squiggly_paren_keywords_test() {
        assert_eq!(keyword_access(&"{}"),                Ok(("", QueryCmd::KeywordAccess(vec![]))));
        assert_eq!(keyword_access(&"{abc}"),             Ok(("", QueryCmd::KeywordAccess(vec!["abc".to_string()]))));
        assert_eq!(keyword_access(&"{abc.def.ghi}"),     Ok(("", QueryCmd::KeywordAccess(vec!["abc".to_string(), "def".to_string(), "ghi".to_string()]))));
        assert_eq!(keyword_access(&"{TotalDuplicateImpressionBucketClicks}"),     Ok(("", QueryCmd::KeywordAccess(vec!["TotalDuplicateImpressionBucketClicks".to_string()]))));
       
    }
}