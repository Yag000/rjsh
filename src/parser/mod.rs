use std::fmt::Display;

use crate::{
    lexer::{token::Token, Lexer},
    parser::ast::Command,
};

use self::ast::Redirection;

pub mod ast;

#[derive(Debug)]
pub struct ParseError {
    message: String,
    token: Option<Token>,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token {
            Some(token) => write!(f, "rjsh: parse error near : {token}\n{}", self.message),
            None => write!(f, "rjsh: {}", self.message),
        }
    }
}

impl ParseError {
    fn new(message: String, token: Option<Token>) -> Self {
        ParseError { message, token }
    }
}

pub struct Parser {
    lexer: Lexer,

    pub current_token: Option<Token>,
    pub peek_token: Option<Token>,
}

impl Parser {
    pub fn new(input: String) -> Parser {
        let lexer = Lexer::new(input);
        let mut p = Parser {
            lexer,
            current_token: None,
            peek_token: None,
        };
        p.read_token();
        p.read_token();
        p
    }

    fn read_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match &self.current_token {
            Some(Token::String(s)) => Ok(s.clone()),
            Some(token) => {
                let message = format!("expected string, found {token}");
                Err(ParseError::new(message, None))
            }
            None => Err(ParseError::new(
                String::from("expected string, found nothing"),
                None,
            )),
        }
    }

    fn parse_redirection(&mut self) -> Result<Redirection, ParseError> {
        let token = self.current_token.clone().unwrap();
        self.read_token();
        let filename = self.expect_string()?;
        Redirection::try_from((token, filename)).map_err(|s| ParseError::new(s, None))
    }

    pub fn parse_command(&mut self) -> Result<Command, ParseError> {
        let name = match self.current_token.clone() {
            Some(Token::String(s)) => s,
            token => return Err(ParseError::new(String::from("Unknown token"), token)),
        };

        self.read_token();

        let mut args = vec![];
        let mut redirections = vec![];
        let mut background = false;

        while let Some(tok) = self.current_token.clone() {
            match tok {
                Token::String(s) => args.push(s),
                Token::Langle
                | Token::Rangle
                | Token::RangleF
                | Token::Rangle2
                | Token::Rangle2F
                | Token::DoubleRangle
                | Token::DoubleRangle2 => redirections.push(self.parse_redirection()?),
                Token::And => {
                    background = true;
                    break;
                }
            }
            self.read_token();
        }

        Ok(Command::new(name, args, redirections, background))
    }
}

#[cfg(test)]
mod tests {
    use self::ast::{Redirectee, RedirectionPermission, RedirectionType};

    use super::*;

    fn assert_empty(input: String) {
        let mut parser = Parser::new(input);
        assert!(parser.parse_command().is_err());
    }

    #[test]
    fn test_parse_command_empty_inputs() {
        assert_empty("".to_string());
        assert_empty(" ".to_string());
        assert_empty("\t".to_string());
        assert_empty("\n".to_string());
        assert_empty("\r".to_string());
        assert_empty("\r\n".to_string());
        assert_empty("  \n     \t  \r   ".to_string());
    }

    fn assert_simple_comamnd(input: String, expected_name: String, expected_args: Vec<String>) {
        let command = Parser::new(input).parse_command();
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
        )
    }

    #[test]
    fn test_parse_command_one_identifier() {
        assert_simple_comamnd("a".to_string(), "a".to_string(), Vec::new());
        assert_simple_comamnd("  ab".to_string(), "ab".to_string(), Vec::new());
        assert_simple_comamnd("abc   ".to_string(), "abc".to_string(), Vec::new());
        assert_simple_comamnd("   a  ".to_string(), "a".to_string(), Vec::new());
    }

    #[test]
    fn test_parse_command_multiple_identifiers() {
        assert_simple_comamnd(
            "a b c".to_string(),
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
    }

    fn assert_command(input: String, expected: Command) {
        let command = Parser::new(input).parse_command();
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
            format!("a {redirection_string} b"),
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
        assert_empty(">".to_string());
        assert_empty("> >".to_string());
        assert_empty(" a >".to_string());
        assert_empty("> b".to_string());
        assert_empty("a  > >".to_string());
        assert_empty("a  > b < ".to_string());
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

        let input = "a < b 2>| c >> d".to_string();
        assert_command(
            input,
            Command::new("a".into(), Vec::new(), redirections, false),
        );
    }

    fn assert_background_comamnd(input: String, expected_name: String, expected_args: Vec<String>) {
        let command = Parser::new(input).parse_command();
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
        )
    }

    #[test]
    fn test_backgroudn_command_one_identifier() {
        assert_background_comamnd("a&".to_string(), "a".to_string(), Vec::new());
        assert_background_comamnd("  ab  & ".to_string(), "ab".to_string(), Vec::new());
        assert_background_comamnd("abc &  ".to_string(), "abc".to_string(), Vec::new());
        assert_background_comamnd("   a  &".to_string(), "a".to_string(), Vec::new());
    }

    #[test]
    fn test_background_multiple_identifiers() {
        assert_background_comamnd(
            "a b c&".to_string(),
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
        assert_background_comamnd(
            "a b c    &".to_string(),
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
        assert_background_comamnd(
            "a b c&  ".to_string(),
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
        assert_background_comamnd(
            "a b   c   &  ".to_string(),
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
    }
}
