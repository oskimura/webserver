extern crate nom;

use std::error::Error;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0, multispace1}, combinator::opt, multi::separated_list0, sequence::{preceded, separated_pair}, IResult, InputTakeAtPosition, AsChar, HexDisplay};
use std::fmt;
use std::hash::{Hash, Hasher};
use nom::combinator::{map, recognize};
use nom::error::{ErrorKind, ParseError};
use serde::{Serialize};
use strum::{IntoEnumIterator};
use md5;
use nom::branch::permutation;
use nom::sequence::{pair, terminated};

#[derive(Debug,PartialEq)]
pub enum SelectColumn {
    ColumnName(String),
}



#[derive(Debug,PartialEq)]
pub struct SelectStatement {
    columns: Vec<SelectColumn>,
    table: String,
    where_clause: Option<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum SelectColumnHashed {
    Column(String, String),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SelectStatementHashed {
    columns: Vec<SelectColumnHashed>,
    table: String,
    where_clause: Option<String>,
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
    let (input, _) = tag("SELECT")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, columns) = separated_list0(
        char(','),
        preceded(multispace0, parse_column),
    )(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("FROM")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, table) = symbol(input)?;
    // WHERE 句の前の空白文字を解析しないように修正
    let (input, where_clause) = opt(preceded(
        permutation((multispace0,tag("WHERE"))),
        permutation((parse_column,tag("="),parse_column)),
    ))(input)?;

    Ok((
        input,
        SelectStatement {
            columns,
            table: table.to_string(),
            where_clause: where_clause.map(|w| w.to_string()),
        },
    ))
}


















#[derive(Debug, PartialEq)]
enum SearchCondition {
    ColumnComparison(String, ComparisonOperator, String),
    And(Box<SearchCondition>, Box<SearchCondition>),
    Or(Box<SearchCondition>, Box<SearchCondition>),
    Not(Box<SearchCondition>),
    Empty,  // ε (空文字列)
}

#[derive(Debug, PartialEq)]
enum ComparisonOperator {
    Equal,
    NotEqual,
}

fn parse_comparison_operator(input: &str) -> IResult<&str, ComparisonOperator> {
    alt((
        map(tag("=="), |_| ComparisonOperator::Equal),
        map(tag("!="), |_| ComparisonOperator::NotEqual),
    ))(input)
}


fn parse_column_name(input: &str) -> IResult<&str, String> {
    map(symbol, |s: &str| s.to_string())(input)
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
            //let mut hasher = DefaultHasher::new();
            let digest = md5::compute(column_name.clone());
            //column_name.hash(&mut hasher);
            SelectColumnHashed::Column(column_name.clone(), format!("{:x}", digest))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_column_name() {
        assert_eq!(
            parse_column("column_name"),
            Ok(("", SelectColumn::ColumnName("column_name".to_string())))
        );
    }



    #[test]
    fn test_parse_select() {
        let sql = "SELECT column1, column2 FROM my_table WHERE column4 = 'value'";
        let expected_ast = SelectStatement {
            columns: vec![
                SelectColumn::ColumnName("column1".to_string()),
                SelectColumn::ColumnName("column2".to_string()),
            ],
            table: "my_table".to_string(),
            where_clause: Some("column4 = 'value'".to_string()),
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

/*
    #[test]
    fn test_parse_comparison_condition() {
        let input = "column1 = column2";
        let expected = SearchCondition::ColumnComparison("column1".to_string(), ComparisonOperator::Equal, "column2".to_string());
        assert_eq!(parse_search_condition(input), Ok(("", expected)));
    }

    #[test]
    fn test_parse_and_condition() {
        let input = "column1 = column2 AND column3 != column4";
        let expected = SearchCondition::And(
            Box::new(SearchCondition::ColumnComparison("column1".to_string(), ComparisonOperator::Equal, "column2".to_string())),
            Box::new(SearchCondition::ColumnComparison("column3".to_string(), ComparisonOperator::NotEqual, "column4".to_string())),
        );
        assert_eq!(parse_and_condition(input), Ok(("", expected)));
    }

    #[test]
    fn test_parse_or_condition() {
        let input = "column1 = column2 OR column3 != column4";
        let expected = SearchCondition::Or(
            Box::new(SearchCondition::ColumnComparison("column1".to_string(), ComparisonOperator::Equal, "column2".to_string())),
            Box::new(SearchCondition::ColumnComparison("column3".to_string(), ComparisonOperator::NotEqual, "column4".to_string())),
        );
        assert_eq!(parse_or_condition(input), Ok(("", expected)));
    }

    #[test]
    fn test_parse_not_condition() {
        let input = "NOT column1 = column2";
        let expected = SearchCondition::Not(Box::new(SearchCondition::ColumnComparison("column1".to_string(), ComparisonOperator::Equal, "column2".to_string())));
        assert_eq!(parse_not_condition(input), Ok(("", expected)));
    }
*/

    #[test]
    fn test_convert_select_statement() {
        let original_select = SelectStatement {
            columns: vec![
                SelectColumn::ColumnName("column1".to_string()),
            ],
            table: "my_table".to_string(),
            where_clause: Some("column2 = 'value'".to_string()),
        };

        let digest = md5::compute("column1");

        let hashed_select = SelectStatementHashed {
            columns: vec![
                SelectColumnHashed::Column("column1".to_string(), format!("{:x}", digest)),
            ],
            table: "my_table".to_string(),
            where_clause: Some("column2 = 'value'".to_string()),
        };


        assert_eq!(convert_select_statement(&original_select), hashed_select);
    }
}


/*
fn main() {
    let sql = "SELECT column1, (column2 + column3) FROM my_table WHERE column4 = 'value'";
    match parse_select(sql) {
        Ok((_, ast)) => println!("{:?}", ast),
        Err(e) => println!("Error: {:?}", e),
    }
}
*/
