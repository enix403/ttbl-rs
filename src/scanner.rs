use std::collections::HashMap;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OperatorType { AND, NOT, OR, CNDL, BI_CNDL }

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType
{
    Operator(OperatorType),
    Variable,
    Literal(bool),

    LeftParen,
    RightParen,

    LeftBrace,
    RightBrace,

    Error,
    EOF
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub token_type: TokenType,
}

pub struct ScanState<'a>
{
    source: &'a str,
    input_length: usize,
    start: usize,
    next_unobserved: usize,
}

impl<'a> ScanState<'a> {
    fn init_from(stmt: & str) -> ScanState {
        return ScanState {
            source: stmt,
            input_length: stmt.len(),
            start: 0,
            next_unobserved: 0,
        };
    }

    fn is_at_end(&self) -> bool
    {
        return self.next_unobserved == self.input_length;
    }

    fn peek(&self) -> char
    {
        return self.source.as_bytes()[self.next_unobserved] as char;
    }

    fn move_forward(&mut self) -> char
    {
        let ch = self.peek();
        self.next_unobserved += 1;
        return ch;
    }

    fn move_if_match(&mut self, target: char) -> bool
    {
        if self.is_at_end() { return false; };
        if self.peek() != target { return false; };

        self.move_forward();
        return true;
    }

    fn make_token(&self, token_type: TokenType) -> Token<'a>
    {
        Token {
            lexeme: &self.source[self.start..self.next_unobserved],
            token_type
        }
    }
}


pub fn tokenize(stmt: &str) -> Vec<Token> {

    let keywords_map: HashMap<&str, OperatorType> = HashMap::from([
        ("and", OperatorType::AND),
        ("or", OperatorType::OR),
        ("not", OperatorType::NOT),
    ]);

    let mut state = ScanState::init_from(stmt);

    let mut tokens: Vec<Token> = vec![];

    while !state.is_at_end()
    {
        let ch = state.move_forward();

        match ch {

            ' ' | '\t' | '\n' => (),

            '&' => { tokens.push(state.make_token(TokenType::Operator(OperatorType::AND))); },
            '|' => { tokens.push(state.make_token(TokenType::Operator(OperatorType::OR))); },
            '!' | '~' => { tokens.push(state.make_token(TokenType::Operator(OperatorType::NOT))); },
            '=' => {
                if state.move_if_match('>')
                {
                    tokens.push(state.make_token(TokenType::Operator(OperatorType::CNDL)));
                }
                else
                {
                    tokens.push(state.make_token(TokenType::Error));
                }
            },

            '<' => {
                if state.move_if_match('=') && state.move_if_match('>')
                {
                    tokens.push(state.make_token(TokenType::Operator(OperatorType::BI_CNDL)));
                }
                else
                {
                    tokens.push(state.make_token(TokenType::Error));
                }
            },

            '(' => { tokens.push(state.make_token(TokenType::LeftParen)); },
            ')' => { tokens.push(state.make_token(TokenType::RightParen)); },
            '{' => { tokens.push(state.make_token(TokenType::LeftBrace)); },
            '}' => { tokens.push(state.make_token(TokenType::RightBrace)); },

            _ => {
                if ch.is_ascii_alphabetic()
                {
                    loop {
                        if state.is_at_end() { break; }

                        let l_next = state.peek();
                        if l_next.is_ascii_alphanumeric() { state.move_forward(); } else { break; }
                    }

                    let mut token = state.make_token(TokenType::Variable);
                    if let Some(&operator_type) = keywords_map.get(token.lexeme)
                    {
                        token.token_type = TokenType::Operator(operator_type);
                    }
                    else 
                    {
                        match token.lexeme.to_ascii_lowercase().as_str()
                        {
                            "true" | "t"  => { token.token_type = TokenType::Literal(true); },
                            "false" | "f" => { token.token_type = TokenType::Literal(false); },
                            _ => ()
                        }
                    }

                    tokens.push(token);
                }
                else
                {
                    tokens.push(state.make_token(TokenType::Error));
                }
            }
        }
        state.start = state.next_unobserved;
    }
    
    tokens.push(Token { lexeme: "<EOF>", token_type: TokenType::EOF });

    return tokens;
}