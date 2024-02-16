use self::token::Token;

pub mod token;

pub struct Lexer {
    input: Vec<char>,     // input string (program)
    position: usize,      // current position in input (points to current char)
    read_position: usize, // current reading position in input (after current char)
    ch: Option<char>,     // current char under examination
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let mut l = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: None,
        };
        l.read_char();
        l
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.ch {
            if !ch.is_whitespace() {
                break;
            }
            self.read_char();
        }
    }

    #[allow(dead_code)]
    pub fn read_identifier(&mut self) -> String {
        let position = self.position;
        while let Some(ch) = self.ch {
            // Minimum effort to get the lexer to work
            if !ch.is_alphanumeric() {
                break;
            }
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }

    pub fn read_string(&mut self) -> String {
        let position = self.position;
        while let Some(ch) = self.ch {
            if ch.is_whitespace() {
                break;
            }
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }

    fn match_rangle(&mut self) -> Token {
        match self.peek_char() {
            '|' => {
                self.read_char();
                Token::RangleF
            }
            '>' => {
                self.read_char();
                Token::DoubleRangle
            }
            _ => Token::Rangle,
        }
    }

    fn match_rangle2(&mut self) -> Token {
        match self.peek_char() {
            '>' => {
                self.read_char();
                match self.peek_char() {
                    '|' => {
                        self.read_char();
                        Token::Rangle2F
                    }
                    '>' => {
                        self.read_char();
                        Token::DoubleRangle2
                    }
                    _ => Token::Rangle2,
                }
            }
            _ => Token::String(self.read_string()),
        }
    }

    #[allow(clippy::match_single_binding)] //TODO: remove this
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let ch = self.ch?;
        let tok = match ch {
            '&' => Token::And,
            '<' => Token::Langle,
            '>' => self.match_rangle(),
            '2' => self.match_rangle2(),
            _ => Token::String(self.read_string()),
        };
        self.read_char();
        Some(tok)
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn test_eof(input: String) {
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn test_next_token_empty_inputs() {
        test_eof("".to_string());
        test_eof(" ".to_string());
        test_eof("\t".to_string());
        test_eof("\n".to_string());
        test_eof("\r".to_string());
        test_eof("\r\n".to_string());
        test_eof("  \n     \t  \r   ".to_string());
    }

    fn test_identifier(input: String, expected: String) {
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Some(Token::String(expected)));
    }

    #[test]
    fn test_next_token_one_identifier() {
        test_identifier("a".to_string(), "a".to_string());
        test_identifier("  ab".to_string(), "ab".to_string());
        test_identifier("abc ".to_string(), "abc".to_string());
        test_identifier("  a   ".to_string(), "a".to_string());
    }

    #[test]
    fn test_next_token_multiple_identifiers() {
        let mut lexer = Lexer::new("a b c".to_string());
        assert_eq!(lexer.next_token(), Some(Token::String("a".to_string())));
        assert_eq!(lexer.next_token(), Some(Token::String("b".to_string())));
        assert_eq!(lexer.next_token(), Some(Token::String("c".to_string())));
        assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn test_redirection_tokens() {
        let mut lexer = Lexer::new("< > 2> >| 2>| >> 2>>".to_string());
        assert_eq!(lexer.next_token(), Some(Token::Langle));
        assert_eq!(lexer.next_token(), Some(Token::Rangle));
        assert_eq!(lexer.next_token(), Some(Token::Rangle2));
        assert_eq!(lexer.next_token(), Some(Token::RangleF));
        assert_eq!(lexer.next_token(), Some(Token::Rangle2F));
        assert_eq!(lexer.next_token(), Some(Token::DoubleRangle));
        assert_eq!(lexer.next_token(), Some(Token::DoubleRangle2));
        assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn test_background_token() {
        let mut lexer = Lexer::new("a & ".to_string());
        assert_eq!(lexer.next_token(), Some(Token::String(String::from("a"))));
        assert_eq!(lexer.next_token(), Some(Token::And));
        assert_eq!(lexer.next_token(), None);
    }
}
