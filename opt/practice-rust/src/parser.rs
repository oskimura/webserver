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
use sqlparser::parser::{Parser, ParserError};
use sqlparser::ast::*;
use sqlparser::ast::{Statement,visit_expressions_mut};
use core::ops::ControlFlow;

pub fn replace_column(sql :&str) -> Result<Vec<Statement>, ParserError> {

    match Parser::parse_sql(&GenericDialect{}, sql) {
        Ok(mut statements) =>{
            visit_expressions_mut(&mut statements, |mut expr| {
                if let Expr::Identifier(col_name) = expr{
                    println!("{}", col_name.value);
                    let digest = md5::compute(col_name.value.clone());
                    col_name.value = format!("{:x}", digest).to_string();
                }
                ControlFlow::<()>::Continue(())
            });
            return Ok(statements)},
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a() {
        let sql = "SELECT x, y FROM t WHERE z = 1";
        let statements = replace_column(sql).unwrap();
        println!("{}",statements[0].to_string());
        let expectd = format!("SELECT {:x}, {:x} FROM t WHERE {:x} = 1",md5::compute("x"),md5::compute("y"),md5::compute("z"));
        assert_eq!(statements[0].to_string(), expectd);

     }
}

