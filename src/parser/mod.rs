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
