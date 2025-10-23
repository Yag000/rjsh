use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::parser::ast::Command;
use crate::parser::token::Token;

use self::ast::Redirection;

pub mod ast;
mod token;

#[derive(Parser)]
#[grammar = "./src/parser/shell.pest"]
pub struct ShellParser;

pub fn parse_command(input: &str) -> Result<Command, String> {
    let mut pairs = ShellParser::parse(Rule::command, input).map_err(|e| e.to_string())?;

    let command_pair = pairs.next().unwrap();
    build_command(command_pair)
}

fn build_command(pair: Pair<Rule>) -> Result<Command, String> {
    let mut name = String::new();
    let mut args = Vec::new();
    let mut redirections = Vec::new();
    let mut background = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::name => name = inner.as_str().to_string(),
            Rule::arg => args.push(inner.as_str().to_string()),
            Rule::redirection => {
                let mut inner_rules = inner.into_inner();
                let op = inner_rules.next().unwrap();
                let token = Token::from(op.as_str());
                let redirectee = inner_rules.next().unwrap();
                let redir = Redirection::try_from((token, redirectee.as_str().to_string()))?;
                redirections.push(redir);
            }
            Rule::background => background = true,
            _ => {}
        }
    }

    Ok(Command::new(name, args, redirections, background))
}

#[cfg(test)]
mod tests {
    use self::ast::{Redirectee, RedirectionPermission, RedirectionType};

    use super::*;

    fn assert_empty(input: &str) {
        assert!(parse_command(input).is_err());
    }

    #[test]
    fn test_parse_command_empty_inputs() {
        assert_empty("");
        assert_empty(" ");
        assert_empty("\t");
        assert_empty("\n");
        assert_empty("\r");
        assert_empty("\r\n");
        assert_empty("  \n     \t  \r   ");
    }

    fn assert_simple_comamnd(input: &str, expected_name: String, expected_args: Vec<String>) {
        let command = parse_command(input);
        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(
            command,
            Command {
                name: expected_name,
                args: expected_args,
                redirections: Vec::new(),
                background: false,
            }
        );
    }

    #[test]
    fn test_parse_command_one_identifier() {
        assert_simple_comamnd("a", "a".to_string(), Vec::new());
        assert_simple_comamnd("  ab", "ab".to_string(), Vec::new());
        assert_simple_comamnd("abc   ", "abc".to_string(), Vec::new());
        assert_simple_comamnd("   a  ", "a".to_string(), Vec::new());
    }

    #[test]
    fn test_parse_command_multiple_identifiers() {
        assert_simple_comamnd(
            "a b c",
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
    }

    fn assert_command(input: &str, expected: Command) {
        let command = parse_command(input);
        if let Err(s) = &command {
            println!("{s} for input \"{input}\"");
        }
        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(command, expected);
    }

    fn test_simple_redirection(
        redirection_string: String,
        type_: RedirectionType,
        permissions: RedirectionPermission,
    ) {
        assert_command(
            format!("a {redirection_string} b").as_str(),
            Command::new(
                "a".into(),
                Vec::new(),
                vec![Redirection::new(
                    Redirectee::FileName("b".into()),
                    type_,
                    permissions,
                )],
                false,
            ),
        );
    }

    #[test]
    fn test_parse_command_simple_redirections() {
        test_simple_redirection(
            "<".into(),
            RedirectionType::Stdin,
            RedirectionPermission::Standard,
        );

        test_simple_redirection(
            ">".into(),
            RedirectionType::Stdout,
            RedirectionPermission::Standard,
        );
        test_simple_redirection(
            "2>".into(),
            RedirectionType::Stderr,
            RedirectionPermission::Standard,
        );

        test_simple_redirection(
            ">|".into(),
            RedirectionType::Stdout,
            RedirectionPermission::Truncate,
        );
        test_simple_redirection(
            "2>|".into(),
            RedirectionType::Stderr,
            RedirectionPermission::Truncate,
        );

        test_simple_redirection(
            ">>".into(),
            RedirectionType::Stdout,
            RedirectionPermission::Append,
        );
        test_simple_redirection(
            "2>>".into(),
            RedirectionType::Stderr,
            RedirectionPermission::Append,
        );
    }

    #[test]
    fn test_invalid_redirections() {
        assert_empty(">");
        assert_empty("> >");
        assert_empty(" a >");
        assert_empty("> b");
        assert_empty("a  > >");
        assert_empty("a  > b < ");
    }

    #[test]
    fn test_parse_multiple_redirections() {
        let redirections = vec![
            Redirection::new(
                Redirectee::FileName("b".into()),
                RedirectionType::Stdin,
                RedirectionPermission::Standard,
            ),
            Redirection::new(
                Redirectee::FileName("c".into()),
                RedirectionType::Stderr,
                RedirectionPermission::Truncate,
            ),
            Redirection::new(
                Redirectee::FileName("d".into()),
                RedirectionType::Stdout,
                RedirectionPermission::Append,
            ),
        ];

        let input = "a < b 2>| c >> d";
        assert_command(
            input,
            Command::new("a".into(), Vec::new(), redirections, false),
        );
    }

    fn assert_background_comamnd(input: &str, expected_name: String, expected_args: Vec<String>) {
        let command = parse_command(input);
        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(
            command,
            Command {
                name: expected_name,
                args: expected_args,
                redirections: Vec::new(),
                background: true,
            }
        );
    }

    #[test]
    fn test_background_invalid() {
        assert_empty("a& &");
        assert_empty("a&&");
        assert_empty("a &&");
        assert_empty("   a &&");
        assert_empty("   a &   &");
        assert_empty("   a &   &    ");
        assert_empty("   a  a &   &    ");
        assert_empty("a > & a &");
        assert_empty("a a  & > a &");
        assert_empty("a a  & > a");
    }

    #[test]
    fn test_background_command_one_identifier() {
        assert_background_comamnd("a&", "a".to_string(), Vec::new());
        assert_background_comamnd("  ab  & ", "ab".to_string(), Vec::new());
        assert_background_comamnd("abc &  ", "abc".to_string(), Vec::new());
        assert_background_comamnd("   a  &", "a".to_string(), Vec::new());
    }

    #[test]
    fn test_background_multiple_identifiers() {
        assert_background_comamnd(
            "a b c&",
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
        assert_background_comamnd(
            "a b c    &",
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
        assert_background_comamnd(
            "a b c&  ",
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
        assert_background_comamnd(
            "a b   c   &  ",
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
    }

    #[test]
    fn test_background_with_redirections() {
        let redirections = vec![Redirection::new(
            Redirectee::FileName("b".into()),
            RedirectionType::Stdout,
            RedirectionPermission::Standard,
        )];
        assert_command(
            "a > b &",
            Command::new("a".into(), Vec::new(), redirections, true),
        );

        let redirections = vec![
            Redirection::new(
                Redirectee::FileName("in".into()),
                RedirectionType::Stdin,
                RedirectionPermission::Standard,
            ),
            Redirection::new(
                Redirectee::FileName("err".into()),
                RedirectionType::Stderr,
                RedirectionPermission::Truncate,
            ),
            Redirection::new(
                Redirectee::FileName("out".into()),
                RedirectionType::Stdout,
                RedirectionPermission::Append,
            ),
        ];

        assert_command(
            "a < in 2>| err >> out &",
            Command::new("a".into(), Vec::new(), redirections, true),
        );
    }
}
