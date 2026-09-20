use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{i32, multispace0, multispace1, not_line_ending};
use nom::combinator::{all_consuming, eof, peek, value, verify};
use nom::multi::many0;
use nom::sequence::{preceded, terminated};
use nom::IResult;

fn skip_line(input: &str) -> IResult<&str, ()> {
    value(
        (),
        preceded(
            terminated(alt((tag("c"), tag("p"))), peek(alt((multispace1, eof)))),
            terminated(not_line_ending, multispace0),
        ),
    )(input)
}

fn parse_clause(input: &str) -> IResult<&str, Vec<i32>> {
    let (input, literals) = many0(terminated(verify(i32, |l| *l != 0), multispace0))(input)?;
    let (input, _) = terminated(tag("0"), multispace0)(input)?;
    Ok((input, literals))
}

pub fn parse_dimacs_cnf(input: &str) -> IResult<&str, Vec<Vec<i32>>> {
    let (input, _) = multispace0(input)?;
    let (input, _) = many0(skip_line)(input)?;
    let (input, clauses) = many0(preceded(many0(skip_line), parse_clause))(input)?;
    let (input, _) = all_consuming(many0(skip_line))(input)?;
    Ok((input, clauses))
}

pub fn parse(input: &str) -> Result<Vec<Vec<i32>>, String> {
    match parse_dimacs_cnf(input) {
        Ok((_, clauses)) => Ok(clauses),
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(unexpected(e.input)),
        Err(nom::Err::Incomplete(_)) => Err("parsing error".to_string()),
    }
}

fn unexpected(rest: &str) -> String {
    let line: String = rest
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(60)
        .collect();
    format!("unexpected input at '{line}'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_clause() {
        let input = "0\n";
        let result = parse_clause(input).expect("parse error").1;
        assert_eq!(result, vec![] as Vec<i32>);
    }

    #[test]
    fn test_parse_clause() {
        let input = "1 -2 8 -5 0\n";
        let result = parse_clause(input).expect("parse error").1;
        assert_eq!(result, vec![1, -2, 8, -5]);
    }

    #[test]
    fn test_empty_formula() {
        let input = "p cnf 0 0\n";
        let result: Vec<Vec<i32>> = parse_dimacs_cnf(input).expect("parse error").1;
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_clause_in_formula_mcs3() {
        let input = "p cnf 3 7\n-1 0\n1 0\n2 0\n-2 0\n3 0\n0\n-3 0\n";
        let result = parse_dimacs_cnf(input).expect("parse error").1;
        assert_eq!(
            result,
            vec![
                vec![-1],
                vec![1],
                vec![2],
                vec![-2],
                vec![3],
                vec![],
                vec![-3]
            ]
        );
    }

    #[test]
    fn test_parse_dimacs_cnf() {
        let input = "p cnf 2 2\n1 0\n-1 2 0\n";
        let result = parse_dimacs_cnf(input).expect("parse error").1;
        assert_eq!(result, vec![vec![1], vec![-1, 2]]);
    }

    #[test]
    fn test_parse_dimacs_cnf_without_first_line() {
        let input = "1 0\n-1 2 0\n";
        let result = parse_dimacs_cnf(input).expect("parse error").1;
        assert_eq!(result, vec![vec![1], vec![-1, 2]]);
    }
}
