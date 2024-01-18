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

    #[allow(clippy::match_single_binding)] //TODO: remove this
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let ch = self.ch?;
        let tok = match ch {
            _ => Token::String(self.read_identifier()),
        };
        self.read_char();
        Some(tok)
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
}
