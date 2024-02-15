use crate::{
    lexer::{token::Token, Lexer},
    parser::ast::Command,
};

pub mod ast;

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

    #[allow(unreachable_patterns)]
    pub fn parse_command(&mut self) -> Option<Command> {
        let name = match self.current_token.clone() {
            Some(Token::String(s)) => s,
            _ => return None,
        };

        self.read_token();

        let mut args = vec![];

        while let Some(tok) = self.current_token.clone() {
            match tok {
                Token::String(s) => args.push(s),
                _ => break,
            }
            self.read_token();
        }

        Some(Command::new(name, args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_empty(input: String) {
        let mut parser = Parser::new(input);
        assert_eq!(parser.parse_command(), None);
    }

    #[test]
    fn test_next_token_empty_inputs() {
        test_empty("".to_string());
        test_empty(" ".to_string());
        test_empty("\t".to_string());
        test_empty("\n".to_string());
        test_empty("\r".to_string());
        test_empty("\r\n".to_string());
        test_empty("  \n     \t  \r   ".to_string());
    }

    fn test_comamnd(input: String, expected_name: String, expected_args: Vec<String>) {
        let command = Parser::new(input).parse_command();
        assert_ne!(command, None);
        let command = command.unwrap();
        assert_eq!(
            command,
            Command {
                name: expected_name,
                args: expected_args,
                redirections: Vec::new(),
            }
        )
    }

    #[test]
    fn test_next_token_one_identifier() {
        test_comamnd("a".to_string(), "a".to_string(), Vec::new());
        test_comamnd("  ab".to_string(), "ab".to_string(), Vec::new());
        test_comamnd("abc   ".to_string(), "abc".to_string(), Vec::new());
        test_comamnd("   a  ".to_string(), "a".to_string(), Vec::new());
    }

    #[test]
    fn test_next_token_multiple_identifiers() {
        test_comamnd(
            "a b c".to_string(),
            "a".to_string(),
            vec!["b".to_string(), "c".to_string()],
        );
    }
}
