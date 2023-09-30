extern crate nom;

use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0, multispace1}, combinator::opt, multi::separated_list0, sequence::preceded, IResult, InputTakeAtPosition, AsChar};
use std::fmt;
use nom::combinator::{map, value};
use nom::error::{ErrorKind, ParseError};
use serde::{Deserialize, Serialize};
use md5;
use nom::branch::permutation;
use nom::bytes::complete::{escaped_transform, tag_no_case, take_while_m_n};
use nom::character::complete::{none_of, space0};
use nom::sequence::delimited;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;


#[derive(Debug,Clone,PartialEq,Serialize,Deserialize,
strum_macros::EnumString,
strum_macros::EnumIter)]
pub enum SelectColumn {
    ColumnName(String),
}

impl fmt::Display for SelectColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectColumn::ColumnName(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize,
Default
)]
pub struct SelectStatement {
    columns: Vec<SelectColumn>,
    table: String,
    where_clause: Option<SearchCondition>,
}

#[derive(Debug, PartialEq, Clone, Serialize,Deserialize,
strum_macros::EnumString,
strum_macros::Display,
strum_macros::IntoStaticStr,
strum_macros::EnumIter
)]
pub enum SelectColumnHashed {
    Column(String, String),
}

#[derive(Debug, PartialEq,Clone,Serialize,Deserialize, Default)]
pub struct SelectStatementHashed {
    columns: Vec<SelectColumnHashed>,
    table: String,
    where_clause: Option<SearchCondition>,
}

pub fn symbol<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
    where
        T: InputTakeAtPosition,
        <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position1_complete(|item| {
        let c = item.as_char();
        !(c.is_alphanumeric() || c == '_')
    }, ErrorKind::AlphaNumeric)
}

pub fn parse_column(input: &str) -> IResult<&str, SelectColumn> {
    alt((
        map_column_name,
    ))(input)
}

fn map_column_name(input: &str) -> IResult<&str, SelectColumn> {
    let (input, column_name) = symbol(input)?;
    Ok((input, SelectColumn::ColumnName(column_name.to_string())))
}

pub fn parse_select(input: &str) -> IResult<&str, SelectStatement> {
    let (input, _) = tag_no_case("SELECT")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, columns) = separated_list0(
        char(','),
        preceded(multispace0, parse_column),
    )(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag_no_case("FROM")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, table) = symbol(input)?;

    let (input, where_clause) = opt(preceded(
          permutation((multispace0,tag_no_case("WHERE"),multispace1)) ,
        parse_search_condition, // Changed to match "WHERE" without spaces
    ))(input)?;
    let (input, where_condition) = match where_clause {
        Some(clause) => (input,clause),
        None => (input, SearchCondition::Empty),
    };

    Ok((
        input,
        SelectStatement {
            columns,
            table: table.to_string(),
            where_clause: Some(where_condition),
        },
    ))
}

fn parse_comparison_operator(input: &str) -> IResult<&str, ComparisonOperator> {
    alt((
        map(permutation((space0,tag("="),space0)) , |_| ComparisonOperator::Equal),
        map(permutation((space0,tag("!="),space0)), |_| ComparisonOperator::NotEqual),
    ))(input)
}

fn parse_and_condition(input: &str) -> IResult<&str, SearchCondition> {

    let (input, _) = permutation((space0,tag_no_case("AND"),space0))(input)?;
    let (input, left) = parse_val(input)?;
    let (input, right) = parse_val(input)?;

    Ok((input, SearchCondition::And(left, right)))
}

fn parse_or_condition(input: &str) -> IResult<&str, SearchCondition> {
    let (input, _) = permutation((space0,tag_no_case("OR"),space0))(input)?;
    let (input, right) = parse_val(input)?;
    let (input, left) = parse_val(input)?;
    Ok((input, SearchCondition::Or(left, right)))
}

fn parse_not_condition(input: &str) -> IResult<&str, SearchCondition> {
    let (input, _) = permutation((space0,tag_no_case("NOT"),space0)) (input)?;
    let (input, condition) = parse_val(input)?;

    Ok((input, SearchCondition::Not(condition)))
}

fn parse_comparison_condition(input: &str) -> IResult<&str, SearchCondition> {

    let (input, left) = parse_val(input)?;
    let (input, operator) = parse_comparison_operator(input)?;
    let (input, right) = parse_val(input)?;

    Ok((input, SearchCondition::ColumnComparison(left, operator,right)))
}


fn parse_search_condition(input: &str) -> IResult<&str, SearchCondition> {
    alt((
        //terminated(parse_search_condition_helper, multispace0),
        parse_comparison_condition,
        parse_and_condition,
        parse_or_condition,
        parse_not_condition,
        map(tag(""), |_| SearchCondition::Empty),  // 空文字列
    ))(input)
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize,
    strum_macros::IntoStaticStr
)]
pub enum Val {
    StringVal(String),
    Column(SelectColumn),
}
impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::StringVal(s) => write!(f, "{}", s),
            Val::Column(SelectColumn::ColumnName(c)) => write!(f, "{}", c),
        }
    }
}

fn parse_column_val(input: &str) -> IResult<&str, Val> {
    let (input, c) = parse_column(input)?;
    Ok((input,Val::Column(c)))
}
fn parse_string(s: &str) -> IResult<&str, Val> {
    let (input, string) = string_literal(s)?;

    Ok((input, Val::StringVal(string)))
}

fn parse_val(input: &str) -> IResult<&str, Val> {
        alt((parse_string, parse_column_val))(input)
}
#[derive(Debug, PartialEq, Clone, Serialize,Deserialize)]
pub enum SearchCondition {
    ColumnComparison(Val, ComparisonOperator, Val),
    And(Val, Val),
    Or(Val, Val),
    Not(Val),
    Empty,  // ε (空文字列)
}

#[derive(Debug, PartialEq, Clone, Serialize,Deserialize,
strum_macros::EnumString,
strum_macros::Display,)]
pub enum ComparisonOperator {
    #[strum(serialize = "=")]
    Equal,
    #[strum(serialize = "!=")]
    NotEqual,
}



// 構造体 SelectStatement から SelectStatementHashed への変換関数
pub fn convert_select_statement(statement: &SelectStatement) -> SelectStatementHashed {
    let columns_hashed: Vec<SelectColumnHashed> = statement
        .columns
        .iter()
        .map(|column| convert_select_column(column))
        .collect();

    SelectStatementHashed {
        columns: columns_hashed,
        table: statement.table.clone(),
        where_clause: statement.where_clause.clone(),
    }
}


// 列挙型 SelectColumn から SelectColumnHashed への変換関数
pub fn convert_select_column(column: &SelectColumn) -> SelectColumnHashed {
    match column {
        SelectColumn::ColumnName(column_name) => {
            let digest = md5::compute(column_name.clone());
            SelectColumnHashed::Column(column_name.clone(), format!("{:x}", digest))
        }
    }
}



fn string_literal(s: &str) -> IResult<&str, String> {
    delimited(
        char('\"'),
        escaped_transform(none_of("\"\\"), '\\', alt((
            value('\\', char('\\')),
            value('\"', char('\"')),
            value('\'', char('\'')),
            value('\r', char('r')),
            value('\n', char('n')),
            value('\t', char('t')),
            map(
                permutation((char('u'), take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()))),
                |(_, code): (char, &str)| -> char {
                    decode_utf16(vec![u16::from_str_radix(code, 16).unwrap()])
                        .nth(0).unwrap().unwrap_or(REPLACEMENT_CHARACTER)
                },
            )
        ))),
        char('\"'),
    )(s)
}
////////////////////////////////////////////////////////
// TRAVERSE
///////////////////////////////////////////////////////
pub fn traverse_select_statement(s: SelectStatementHashed) -> String {
    let columns_str: String = s
        .columns
        .iter()
        .map(|SelectColumnHashed::Column(_, b)| b.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let where_clause_str = traverse_where_clause(s.where_clause);

    format!("SELECT {} FROM {} {}", columns_str, s.table, where_clause_str)
}

pub fn traverse_where_clause(s: Option<SearchCondition>) -> String {
    match s {
        Some(SearchCondition::ColumnComparison(a, op, c)) => " WHERE ".to_owned() + &*a.to_string() + &*op.to_string() + &*c.to_string(),
        Some(SearchCondition::And(a,b)) => " WHERE ".to_owned() + &*a.to_string() + " AND " + &*b.to_string(),
        Some(SearchCondition::Or(a,b)) =>  " WHERE ".to_owned() + &*a.to_string() + " OR " + &*b.to_string(),
        Some(SearchCondition::Not(a)) =>  " WHERE ".to_owned() + &*"NOT ".to_owned() + &*a.to_string(),
        Some(SearchCondition::Empty) => "".to_string(),
        None => "".to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bb() {
        assert_eq!(string_literal("\"abcd\""), Ok(("", String::from("abcd"))));
    }
    #[test]
    fn test_parse_column_name() {
        assert_eq!(
            parse_column("column_name"),
            Ok(("", SelectColumn::ColumnName("column_name".to_string())))
        );
    }

    #[test]
    fn test_string_literal() {
        // Test with a simple string
        let input = "\"Hello, World!\"";
        let expected_output = Val::StringVal("Hello, World!".to_string());
        assert_eq!(parse_string(input), Ok(("", expected_output)));

        // Test with an escaped character
        let input = "\"Newline: \\n\"";
        let expected_output = Val::StringVal("Newline: \n".to_string());
        assert_eq!(parse_string(input), Ok(("", expected_output)));

        // Add more test cases as needed
    }



    #[test]
    fn test_parse_select() {
        let sql = "SELECT column1, column2 FROM my_table WHERE column4 = \"value\"";
        let expected_ast = SelectStatement {
            columns: vec![
                SelectColumn::ColumnName("column1".to_string()),
                SelectColumn::ColumnName("column2".to_string()),
            ],
            table: "my_table".to_string(),
            where_clause: Some(SearchCondition::ColumnComparison(
                Val::Column(SelectColumn::ColumnName("column4".to_string())),
                ComparisonOperator::Equal,
                Val::StringVal("value".to_string()),
            )),
        };

        assert_eq!(parse_select(sql), Ok(("", expected_ast)));
    }

    #[test]
    fn test_convert_select_column_column_name() {
        let column = SelectColumn::ColumnName("column1".to_string());
        let digest = md5::compute("column1");
        let expected = SelectColumnHashed::Column("column1".to_string(), format!("{:x}", digest));
        assert_eq!(convert_select_column(&column), expected);
    }


    #[test]
    fn test_parse_comparison_condition() {
        let input = "column1 = column2";
        let expected = SearchCondition::ColumnComparison(Val::Column(SelectColumn::ColumnName("column1".to_string())), ComparisonOperator::Equal, Val::Column(SelectColumn::ColumnName("column2".to_string())));
        assert_eq!(parse_search_condition(input), Ok(("", expected)));
    }

    #[test]
    fn test_convert_select_statement() {
        let original_select = SelectStatement {
            columns: vec![
                SelectColumn::ColumnName("column1".to_string()),
            ],
            table: "my_table".to_string(),
            where_clause: Some(SearchCondition::ColumnComparison(
                Val::Column(SelectColumn::ColumnName("column2".to_string())),
                ComparisonOperator::Equal,
                Val::StringVal("'value'".to_string()),
            )),
        };

        let digest = md5::compute("column1");

        let hashed_select = SelectStatementHashed {
            columns: vec![
                SelectColumnHashed::Column("column1".to_string(), format!("{:x}", digest)),
            ],
            table: "my_table".to_string(),
            where_clause: Some(SearchCondition::ColumnComparison(
                Val::Column(SelectColumn::ColumnName("column2".to_string())),
                ComparisonOperator::Equal,
                Val::StringVal("'value'".to_string()),
            )),
        };


        assert_eq!(convert_select_statement(&original_select), hashed_select);
    }

    #[test]
    fn test_traverse_select_statement() {

        let select_stmt = SelectStatementHashed {
            columns: vec![
                SelectColumnHashed::Column("column1".to_string(),"col1".to_string()),
                SelectColumnHashed::Column("column2".to_string(),"col2".to_string())
            ],
            table: "my_table".to_string(),
            where_clause: Option::Some(SearchCondition::ColumnComparison(Val::Column(SelectColumn::ColumnName( "column1".to_string())),ComparisonOperator::Equal,   Val::StringVal("value".to_string()))),
        };

        let result = traverse_select_statement(select_stmt);
        let expected = "SELECT col1,col2 FROM my_table WHERE column1 = 'value'".to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_traverse_where_clause() {
        let cond = SearchCondition::ColumnComparison(
            Val::Column(SelectColumn::ColumnName("column1".to_string())),
            ComparisonOperator::Equal,
            Val::StringVal("value".to_string()),
        );

        let result = traverse_where_clause(Some(cond));
        let expected = "column1 = \"value\"".to_string();

        assert_eq!(result, expected);
    }

}

#[test]
fn a() {
    let sql = "SELECT a, b, 123, myfunc(b) \
           FROM table_1 \
           WHERE a > b AND b < 100 \
           ORDER BY a DESC, b";

    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    println!("AST: {:?}", ast);
}

