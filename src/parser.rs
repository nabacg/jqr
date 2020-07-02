use nom::{
  IResult,
  bytes::complete::{tag},
  combinator::{map_res, map, all_consuming},
};
use nom::multi::{separated_list};
use nom::character::complete::{ digit1};
use nom::branch::alt;
use nom::error::{ErrorKind};
use nom::{ InputTakeAtPosition, AsChar};

#[derive(Debug, Eq, Clone)]
pub enum QueryCmd {
    MultiArrayIndex(Vec<usize>),
    KeywordAccess(Vec<String>),
    MultiCmd(Vec<QueryCmd>),
    ListKeys,
    ListValues,

}

impl PartialEq for QueryCmd {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (QueryCmd::MultiArrayIndex(xs), QueryCmd::MultiArrayIndex(ys)) => xs == ys,
            (QueryCmd::KeywordAccess(xs), QueryCmd::KeywordAccess(ys)) => xs == ys,
            (QueryCmd::MultiCmd(xs), QueryCmd::MultiCmd(ys)) => xs == ys,
            (QueryCmd::ListKeys, QueryCmd::ListKeys)            => true, 
            (QueryCmd::ListValues, QueryCmd::ListValues)        => true,
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

named!(list_keys_or_vals(&str) -> QueryCmd, alt!(
    tag!(".vals") => { |_| QueryCmd::ListValues} | 
    tag!(".keys") => { |_| QueryCmd::ListKeys }
)); 

//named!(keyword_access(&str) -> QueryCmd, map!(ws!(tuple!(tag!("{"), call!(string_list), tag!("}"))), |(_, ks, _)| QueryCmd::KeywordAccess(ks)));
named!(keyword_access(&str) -> QueryCmd, map!(ws!(call!(string_list)), |ks| QueryCmd::KeywordAccess(ks)));

named!(int_list(&str) ->  Vec<usize>,  ws!(separated_list!(tag(","), map_res(digit1, |s: &str| s.parse::<usize>()))));

named!(array_index_access(&str) -> QueryCmd, map!(ws!(tuple!(tag!("["), call!(int_list), tag!("]"))), |(_, ids, _)| QueryCmd::MultiArrayIndex(ids)));

named!(single_cmd(&str) -> QueryCmd, alt!(list_keys_or_vals | array_index_access | keyword_access));

fn single_top_level_cmd(s: &str) -> IResult<&str, QueryCmd> {
    // all_consuming - makes sure parser succeeds only if all input was consumed 
    // https://docs.rs/nom/5.0.0/nom/combinator/fn.all_consuming.html?search=
    all_consuming(single_cmd)(s)
}

named!(multi_cmd_list( &str) -> QueryCmd, 
    map!(ws!(separated_list!(tag("|"),  single_cmd)),
      |cmds| QueryCmd::MultiCmd(cmds)));
//https://docs.rs/nom/5.0.0/nom/macro.alt.html#behaviour-of-alt
named!(top_level_parser(&str) -> QueryCmd, alt!( single_top_level_cmd | multi_cmd_list));
//named!(top_level_parser(&str) -> QueryCmd, multi_cmd_list);

pub fn parse(input: &str) -> IResult<&str, QueryCmd> {
    top_level_parser(input)
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
        assert_eq!(keyword_access(&""),                Ok(("", QueryCmd::KeywordAccess(vec![]))));
        assert_eq!(keyword_access(&"abc"),             Ok(("", QueryCmd::KeywordAccess(vec!["abc".to_string()]))));
        assert_eq!(keyword_access(&"abc.def.ghi"),     Ok(("", QueryCmd::KeywordAccess(vec!["abc".to_string(), "def".to_string(), "ghi".to_string()]))));
        assert_eq!(keyword_access(&"TotalDuplicateImpressionBucketClicks"),     Ok(("", QueryCmd::KeywordAccess(vec!["TotalDuplicateImpressionBucketClicks".to_string()]))));
       

        assert_eq!(keyword_access(&"abc-e.edf_g"),     Ok(("", QueryCmd::KeywordAccess(vec!["abc-e".to_string(), "edf_g".to_string()]))));
       
    }



    #[test]
    fn multi_cmd_list_test() {
        assert_eq!(multi_cmd_list(&"aaa"), Ok(("", QueryCmd::MultiCmd(vec![
            QueryCmd::KeywordAccess(vec!["aaa".to_string()])
            ]))));
        assert_eq!(multi_cmd_list(&"[0]"), Ok(("", QueryCmd::MultiCmd(vec![
            QueryCmd::MultiArrayIndex(vec![0])]))));
        assert_eq!(multi_cmd_list(&"[0] | abc"), Ok(("", QueryCmd::MultiCmd(vec![
                                                                QueryCmd::MultiArrayIndex(vec![0]), 
                                                                QueryCmd::KeywordAccess(vec!["abc".to_string()])
                                                                ]))));
    }
    #[test]
    fn parse_test() {
        assert_eq!(parse(&"[0]|abc"), Ok(("", QueryCmd::MultiCmd(vec![
                                                                QueryCmd::MultiArrayIndex(vec![0]), 
                                                                QueryCmd::KeywordAccess(vec!["abc".to_string()])]))));
    }




    #[test]
    fn list_vals_or_keys_test() {
        assert_eq!(parse(&".vals"), Ok(("", QueryCmd::ListValues)));
        assert_eq!(parse(&".keys"), Ok(("", QueryCmd::ListKeys)));

        assert_eq!(parse(&"[0] | ArrayField | .keys"), Ok(("", QueryCmd::MultiCmd(vec![
            QueryCmd::MultiArrayIndex(vec![0]), 
            QueryCmd::KeywordAccess(vec!["ArrayField".to_string()]),
            QueryCmd::ListKeys
            ]))));

            assert_eq!(parse(&"[0] | ArrayField | .keys | SubTree | [4] | .vals"), Ok(("", QueryCmd::MultiCmd(vec![
                QueryCmd::MultiArrayIndex(vec![0]), 
                QueryCmd::KeywordAccess(vec!["ArrayField".to_string()]),
                QueryCmd::ListKeys,
                QueryCmd::KeywordAccess(vec!["SubTree".to_string()]),
                QueryCmd::MultiArrayIndex(vec![4]),
                QueryCmd::ListValues,
                ]))));
    }


}
