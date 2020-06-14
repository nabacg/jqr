use nom::{
  IResult,
  bytes::complete::{tag},
  combinator::{map_res, map},
};
use nom::multi::{separated_list};
use nom::character::complete::{ digit1};
use nom::branch::alt;
use nom::error::{ErrorKind};
use nom::{ InputTakeAtPosition, AsChar};


#[derive(Debug, Eq)]
pub enum QueryCmd {
    MultiArrayIndex(Vec<usize>),
    KeywordAccess(Vec<String>),
    MultiCmd(Vec<QueryCmd>)

}

impl PartialEq for QueryCmd {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (QueryCmd::MultiArrayIndex(xs), QueryCmd::MultiArrayIndex(ys)) => xs == ys,
            (QueryCmd::KeywordAccess(xs), QueryCmd::KeywordAccess(ys)) => xs == ys,
            (QueryCmd::MultiCmd(xs), QueryCmd::MultiCmd(ys)) => xs == ys,
            _ => false
        }
    }
}

fn alpha_or_spec_char(input: &str) -> IResult<&str, &str>
{
    //ToDo this predicate needs to be rewritten as not(char('.')) because I think we allow pretty much every charater in Json keyword, don't we? 
    // but maybe even '.' should be allowed if quoted?
    input.split_at_position1_complete(|item| !(item.is_alphanum() || item == '-' || item == '_' || item == '?'), ErrorKind::Alpha)
}

fn string_list(s: &str) -> IResult<&str, Vec<String>> {
    separated_list(tag("."), map(alpha_or_spec_char, |s: &str| s.to_string()))(s)
}


//named!(keyword_access(&str) -> QueryCmd, map!(ws!(tuple!(tag!("{"), call!(string_list), tag!("}"))), |(_, ks, _)| QueryCmd::KeywordAccess(ks)));
named!(keyword_access(&str) -> QueryCmd, map!(ws!(call!(string_list)), |ks| QueryCmd::KeywordAccess(ks)));

named!(int_list(&str) ->  Vec<usize>,  ws!(separated_list!(tag(","), map_res(digit1, |s: &str| s.parse::<usize>()))));

named!(array_index_access(&str) -> QueryCmd, map!(ws!(tuple!(tag!("["), call!(int_list), tag!("]"))), |(_, ids, _)| QueryCmd::MultiArrayIndex(ids)));

named!(multi_cmd_list( &str) -> QueryCmd, 
    map!(ws!(separated_list!(tag("|"),  
     alt((array_index_access,keyword_access)))),
      |cmds| QueryCmd::MultiCmd(cmds)));


pub fn parse(input: &str) -> IResult<&str, QueryCmd> {
     multi_cmd_list(input)
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
       

        assert_eq!(keyword_access(&"{abc-e.edf_g}"),     Ok(("", QueryCmd::KeywordAccess(vec!["abc-e".to_string(), "edf_g".to_string()]))));
       
    }



    #[test]
    fn multi_cmd_list_test() {
        assert_eq!(multi_cmd_list(&"{aaa}"), Ok(("", QueryCmd::MultiCmd(vec![
            QueryCmd::KeywordAccess(vec!["aaa".to_string()])
            ]))));
        assert_eq!(multi_cmd_list(&"[0]"), Ok(("", QueryCmd::MultiCmd(vec![
            QueryCmd::MultiArrayIndex(vec![0])]))));
        assert_eq!(multi_cmd_list(&"[0] | {abc}"), Ok(("", QueryCmd::MultiCmd(vec![
                                                                QueryCmd::MultiArrayIndex(vec![0]), 
                                                                QueryCmd::KeywordAccess(vec!["abc".to_string()])
                                                                ]))));
    }
    #[test]
    fn parse_test() {
        assert_eq!(parse(&"[0]|{abc}"), Ok(("", QueryCmd::MultiCmd(vec![
                                                                QueryCmd::MultiArrayIndex(vec![0]), 
                                                                QueryCmd::KeywordAccess(vec!["abc".to_string()])]))));
    }


}
