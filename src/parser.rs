use nom::{
  IResult,
//   sequence::{delimited, tuple},
  bytes::complete::{tag, is_a},
  combinator::{map_res, map},
};
use nom::bytes::complete::take_while;
use nom::multi::{separated_list, many1};
use nom::character::is_alphanumeric;
use nom::character::complete::{alpha0, alpha1, digit1, char, anychar};
use nom::branch::alt;
use nom::error::{ErrorKind, ParseError};
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
            //(QueryCmd::SingleArrayIndex(i), QueryCmd::SingleArrayIndex(j)) => i == j,
            (QueryCmd::MultiArrayIndex(xs), QueryCmd::MultiArrayIndex(ys)) => xs == ys,
            (QueryCmd::KeywordAccess(xs), QueryCmd::KeywordAccess(ys)) => xs == ys,
            (QueryCmd::MultiCmd(xs), QueryCmd::MultiCmd(ys)) => xs == ys,
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
// fn int_list(s: &str) -> IResult<&str, Vec<usize>> {
//     separated_list(tag(","), map_res(digit1, |s: &str| s.parse::<usize>()))(s)
// }


// fn single_int(s: &str) -> IResult<&str, usize> {
//     map_res(digit1, |s: &str| s.parse::<usize>())(s)
// }

// fn array_index_access(s: &str) -> IResult<&str, QueryCmd> {
//     let (input, (_, xs, _)) = tuple((char('['), int_list, char(']')))(s)?;
//     Ok((input, QueryCmd::MultiArrayIndex(xs)))
// }

// fn keyword_access(s: &str) -> IResult<&str, QueryCmd> {
//     let (input, (_, xs, _)) = tuple((char('{'), string_list, char('}')))(s)?;
//     Ok((input, QueryCmd::KeywordAccess(xs)))
// }

// fn multi_cmd_list(s: &str) -> IResult<&str, QueryCmd> {
//     let (input, cmds) = separated_list(tag("|"), alt((array_index_access, keyword_access)))(s)?;
//     Ok((input, QueryCmd::MultiCmd(cmds)))
// }

// Nom macros https://github.com/Geal/nom/blob/master/doc/how_nom_macros_work.md

// named!(multi_cmd_listMac( &str) -> Vec<QueryCmd>, 
//   ws!(separated_list!(tag!("|"), 
//     alt!((
//         map!(separated_list!(tag!(","), map_res!(digit1, |s: &str| s.parse::<usize>())), |ids| QueryCmd::MultiArrayIndex(ids)) , 
//         map!(separated_list!(tag!("."), map!(alpha1, |s: &str| s.to_string()))         , |kws| QueryCmd::KeywordAccess(kws))
//     )))
//  )
// );
fn alpha_or_spec_char(input: &str) -> IResult<&str, &str>
{
    input.split_at_position1_complete(|item| !(item.is_alpha() || item == '-' || item == '_'), ErrorKind::Alpha)
}

fn string_list(s: &str) -> IResult<&str, Vec<String>> {
    separated_list(tag("."), map(alpha_or_spec_char, |s: &str| s.to_string()))(s)
}
//     separated_list(tag("."), map(
//         //(take_while(|c| is_alphanumeric(c) || is_a("_")(c) || is_a("-")(c) )
//         many1(alt(
//             (alpha0, 
//                 alt(
//                     (tag("_"), 
//                     tag("-")))
//                 )))
// , |s: &str| s.to_string()))(s)


named!(keyword_access(&str) -> QueryCmd, map!(ws!(tuple!(tag!("{"), call!(string_list), tag!("}"))), |(_, ks, _)| QueryCmd::KeywordAccess(ks)));

named!(int_list(&str) ->  Vec<usize>,  ws!(separated_list!(tag(","), map_res(digit1, |s: &str| s.parse::<usize>()))));

named!(array_index_access(&str) -> QueryCmd, map!(ws!(tuple!(tag!("["), call!(int_list), tag!("]"))), |(_, ids, _)| QueryCmd::MultiArrayIndex(ids)));

named!(multi_cmd_list( &str) -> QueryCmd, 
    map!(ws!(separated_list!(tag("|"),  
     alt((array_index_access,keyword_access)))),
      |cmds| QueryCmd::MultiCmd(cmds)));


// named!(taggy_tags(&str) -> &str,  ws!(tag!(",")));

// named!(int_mac(&str) ->  usize,  ws!( map_res!(digit1, |s: &str| s.parse::<usize>())));


// combinator list
// https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md

//ToDo
// handle spaces in int_list like [1, 2, 3 ]
// https://docs.rs/nom/5.1.1/nom/
pub fn parse(input: &str) -> IResult<&str, QueryCmd> {

    //alt((array_index_access, keyword_access))(input)
    //multi_cmd_list(input)
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
