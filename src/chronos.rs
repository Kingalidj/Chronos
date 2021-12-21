use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::mem;
use std::rc::Rc;

use crate::errors::*;

const DIGITS: &str = "0123456789";
const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

type ChInt = i32;
type ChFloat = f32;

fn match_enum_type<T>(t1: &T, t2: &T) -> bool {
    mem::discriminant(t1) == mem::discriminant(t2)
}

#[derive(Debug, Clone)]
//TODO: remove file_name and text from position!!!
pub struct Position {
    pub file_name: Rc<String>,
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub text: Rc<String>,
}

impl Position {
    fn new(file_name: String, index: usize, line: usize, column: usize, text: String) -> Self {
        Position {
            file_name: Rc::new(file_name),
            index,
            line,
            column,
            text: Rc::new(text),
        }
    }

    fn from_name(file_name: String) -> Self {
        Position {
            file_name: Rc::new(file_name),
            index: 0,
            line: 0,
            column: 0,
            text: Rc::new("".to_string()),
        }
    }

    pub fn empty() -> Self {
        Position {
            file_name: Rc::new(String::from("")),
            index: 0,
            line: 0,
            column: 0,
            text: Rc::new(String::from("")),
        }
    }

    fn advance(&mut self, current_char: &Option<char>) {
        match *current_char {
            Some('\n') => {
                self.line += 1;
                self.index += 1;
                self.column = 0;
            }

            _ => {
                self.index += 1;
                self.column += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Keyword {
    AND,
    OR,
    NOT,
    IF,
    ELIF,
    ELSE,
    WHILE,
    FOR,
    FUNC,
}

fn get_keyword(s: &String) -> Result<Keyword, ()> {
    match s.as_ref() {
        "&&" => Ok(Keyword::AND),
        "||" => Ok(Keyword::OR),
        "!" => Ok(Keyword::NOT),
        "if" => Ok(Keyword::IF),
        "elif" => Ok(Keyword::ELIF),
        "else" => Ok(Keyword::ELSE),
        "while" => Ok(Keyword::WHILE),
        "for" => Ok(Keyword::FOR),
        "fn" => Ok(Keyword::FUNC),
        _ => Err(()),
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    INT(ChInt),
    FLOAT(ChFloat),
    STRING(String),
    ADD,
    INCRMNT,
    SUB,
    DECRMNT,
    MUL,
    DIV,
    POW,
    LROUND,
    RROUND,
    LCURLY,
    RCURLY,
    SEMICLN,
    COMMA,
    EOF,

    ID(String),
    KEYWRD(Keyword),
    ASSIGN,

    EQUAL,
    NEQUAL,
    LESS,
    LESSEQ,
    GREATER,
    GREATEREQ,
}

#[derive(Clone)]
pub struct Token {
    token_type: TokenType,
    start_pos: Position,
    end_pos: Position,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.token_type)
    }
}

impl Token {
    pub fn new(token_type: TokenType, start_pos: Position, end_pos: Option<Position>) -> Self {
        if let Some(end) = end_pos {
            return Token {
                token_type,
                start_pos: start_pos.clone(),
                end_pos: end.clone(),
            };
        }

        let mut end_pos = start_pos.clone();
        end_pos.advance(&None);

        Token {
            token_type,
            start_pos: start_pos.clone(),
            end_pos,
        }
    }
}

struct Lexer {
    text: Box<[u8]>,
    position: Position,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(file_name: String, text: String) -> Self {
        let mut l = Lexer {
            text: (text.as_bytes().into()),
            position: Position {
                file_name: Rc::new(file_name),
                index: 0,
                line: 0,
                column: 0,
                text: Rc::new(text),
            },
            current_char: None,
        };
        l.current_char = Some(l.text[l.position.index] as char);
        l
    }

    fn advance(&mut self) {
        self.position.advance(&self.current_char);

        self.current_char = if self.position.index < self.text.len() {
            Some(self.text[self.position.index] as char)
        } else {
            None
        };
    }

    fn parse_tokens(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.current_char != None {
            let c = self.current_char.unwrap();

            if " \t\n".contains(c) {
                self.advance();
            } else if match c {
                '+' => {
                    tokens.push(self.make_add()?);
                    true
                }
                '-' => {
                    tokens.push(self.make_sub()?);
                    true
                }
                '/' => {
                    tokens.push(Token::new(TokenType::DIV, self.position.clone(), None));
                    self.advance();
                    true
                }
                '*' => {
                    tokens.push(Token::new(TokenType::MUL, self.position.clone(), None));
                    self.advance();
                    true
                }
                '"' | '\'' => {
                    tokens.push(self.make_string()?);
                    true
                }
                '^' => {
                    tokens.push(Token::new(TokenType::POW, self.position.clone(), None));
                    self.advance();
                    true
                }
                '(' => {
                    tokens.push(Token::new(TokenType::LROUND, self.position.clone(), None));
                    self.advance();
                    true
                }
                ')' => {
                    tokens.push(Token::new(TokenType::RROUND, self.position.clone(), None));
                    self.advance();
                    true
                }
                '{' => {
                    tokens.push(Token::new(TokenType::LCURLY, self.position.clone(), None));
                    self.advance();
                    true
                }
                '}' => {
                    tokens.push(Token::new(TokenType::RCURLY, self.position.clone(), None));
                    self.advance();
                    true
                }
                ',' => {
                    tokens.push(Token::new(TokenType::COMMA, self.position.clone(), None));
                    self.advance();
                    true
                }
                ';' => {
                    tokens.push(Token::new(TokenType::SEMICLN, self.position.clone(), None));
                    self.advance();
                    true
                }
                '=' => {
                    tokens.push(self.make_equal());
                    true
                }
                '!' => {
                    tokens.push(self.make_not()?);
                    true
                }
                '<' => {
                    tokens.push(self.make_less());
                    true
                }
                '>' => {
                    tokens.push(self.make_greater());
                    true
                }

                '&' | '|' => {
                    tokens.push(self.make_keyword()?);
                    true
                }
                _ => false,
            } {
            } else if LETTERS.contains(c) {
                tokens.push(self.make_identifier());
            } else if DIGITS.contains(c) {
                tokens.push(self.make_number());
            } else {
                let start_pos = self.position.clone();
                self.advance();
                return Err(Error::new(
                    ErrType::IllegalCharError,
                    &start_pos,
                    &self.position,
                    format!("Lexer: found '{}'", c),
                    None,
                ));
            }
        }

        tokens.push(Token::new(TokenType::EOF, self.position.clone(), None));
        Ok(tokens)
    }

    fn make_string(&mut self) -> Result<Token, Error> {
        let start = self.position.clone();
        let mut s = String::from("");
        self.advance();

        let escape_char = "\"\'";

        while self.current_char != None && !escape_char.contains(self.current_char.unwrap()) {
            s += &self.current_char.unwrap().to_string();
            self.advance();
        }
        self.advance();
        let end = self.position.clone();

        Ok(Token {
            token_type: TokenType::STRING(s),
            start_pos: start,
            end_pos: end,
        })
    }

    fn make_add(&mut self) -> Result<Token, Error> {
        let start = self.position.clone();
        self.advance();

        match self.current_char.unwrap() {
            '=' => {
                self.advance();
                Ok(Token::new(
                    TokenType::INCRMNT,
                    start,
                    Some(self.position.clone()),
                ))
            }
            _ => Ok(Token::new(TokenType::ADD, start, None)),
        }
    }

    fn make_sub(&mut self) -> Result<Token, Error> {
        let start = self.position.clone();
        self.advance();

        match self.current_char.unwrap() {
            '=' => {
                self.advance();
                Ok(Token::new(
                    TokenType::DECRMNT,
                    start,
                    Some(self.position.clone()),
                ))
            }
            _ => Ok(Token::new(TokenType::SUB, start, None)),
        }
    }

    fn make_keyword(&mut self) -> Result<Token, Error> {
        let mut keyword = self.current_char.unwrap().to_string();
        let start = self.position.clone();

        self.advance();
        if self.current_char != None {
            keyword.push(self.current_char.unwrap())
        }

        self.advance();
        match get_keyword(&keyword) {
            Ok(k) => Ok(Token::new(
                TokenType::KEYWRD(k),
                start,
                Some(self.position.clone()),
            )),
            Err(_) => Err(Error::new(
                ErrType::IllegalCharError,
                &start,
                &self.position,
                format!(
                    "Lexer: Unknown Keyword, expected '&&', '||' or '!' found '{}'",
                    keyword
                ),
                None,
            )),
        }
    }

    fn make_not(&mut self) -> Result<Token, Error> {
        let start = self.position.clone();
        self.advance();

        if self.current_char != None && self.current_char.unwrap() == '=' {
            self.advance();
            Ok(Token::new(
                TokenType::NEQUAL,
                start,
                Some(self.position.clone()),
            ))
        } else {
            Ok(Token::new(
                TokenType::KEYWRD(Keyword::NOT),
                start,
                Some(self.position.clone()),
            ))
        }
    }

    fn make_equal(&mut self) -> Token {
        let start = self.position.clone();
        let mut token_type = TokenType::ASSIGN;
        self.advance();

        if self.current_char != None && self.current_char.unwrap() == '=' {
            self.advance();
            token_type = TokenType::EQUAL;
        }

        Token::new(token_type, start, Some(self.position.clone()))
    }

    fn make_less(&mut self) -> Token {
        let start = self.position.clone();
        let mut token_type = TokenType::LESS;
        self.advance();

        if self.current_char != None && self.current_char.unwrap() == '=' {
            self.advance();
            token_type = TokenType::LESSEQ;
        }

        Token::new(token_type, start, Some(self.position.clone()))
    }

    fn make_greater(&mut self) -> Token {
        let start = self.position.clone();
        let mut token_type = TokenType::GREATER;
        self.advance();

        if self.current_char != None && self.current_char.unwrap() == '=' {
            self.advance();
            token_type = TokenType::GREATEREQ;
        }

        Token::new(token_type, start, Some(self.position.clone()))
    }

    fn make_identifier(&mut self) -> Token {
        let mut id = String::from("");
        let pos_start = self.position.clone();

        let allowed = LETTERS.to_owned() + "_";

        while self.current_char != None && allowed.contains(self.current_char.unwrap()) {
            id.push(self.current_char.unwrap());
            self.advance();
        }

        let token_type = match get_keyword(&id) {
            Ok(k) => TokenType::KEYWRD(k),
            Err(()) => TokenType::ID(id),
        };
        Token::new(token_type, pos_start, Some(self.position.clone()))
    }

    //TODO: don't use strings
    fn make_number(&mut self) -> Token {
        let mut num: String = String::new();
        let mut dot_count: u8 = 0;

        let start = self.position.clone();

        let s = DIGITS.to_owned() + ".";

        while self.current_char != None && (s).contains(self.current_char.unwrap()) {
            let c = self.current_char.unwrap();
            if c == '.' {
                if dot_count >= 1 {
                    break;
                }
                dot_count += 1;
                num += ".";
            } else {
                num.push(c);
            }

            self.advance();
        }

        if dot_count == 0 {
            return Token::new(
                TokenType::INT(num.parse::<ChInt>().unwrap()),
                start,
                Some(self.position.clone()),
            );
        }

        Token::new(
            TokenType::FLOAT(num.parse::<ChFloat>().unwrap()),
            start,
            Some(self.position.clone()),
        )
    }
}

struct Parser {
    tokens: Vec<Token>,
    token_index: usize,
    current_token: Token,
}

#[derive(Debug, Clone)]
pub enum Node {
    NUM(Token),
    STRING(Token),
    BINOP(Box<Node>, Token, Box<Node>),
    UNRYOP(Token, Box<Node>),
    ASSIGN(Token, Box<Node>),
    ACCESS(Token),
    IF(Vec<(Node, Node)>, Option<Box<Node>>),
    WHILE(Box<Node>, Box<Node>, Position, Position),
    FOR(
        Option<Box<Node>>,
        Box<Node>,
        Option<Box<Node>>,
        Box<Node>,
        Position,
        Position,
    ),
    FUNCDEF(Option<Token>, Vec<Token>, Box<Node>, Position, Position),
    CALL(Box<Node>, Vec<Node>),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let t = tokens[0].clone();
        Parser {
            tokens,
            token_index: 0,
            current_token: t,
        }
    }

    fn parse(&mut self) -> Result<Node, Error> {
        let nodes = self.expression()?;

        match self.current_token.token_type {
            TokenType::EOF => Ok(nodes),

            _ => Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!(
                    "Parser: expected EOF found {:?}",
                    self.current_token.token_type
                ),
                None,
            )),
        }
    }

    fn advance(&mut self) {
        self.token_index += 1;

        if self.token_index < self.tokens.len() {
            self.current_token = self.tokens[self.token_index].clone();
        }
    }

    fn retreat(&mut self) {
        self.token_index -= 1;
        self.current_token = self.tokens[self.token_index].clone();
    }

    fn atom(&mut self) -> Result<Node, Error> {
        let t = self.current_token.clone();

        return match t.token_type {
            TokenType::INT(_) | TokenType::FLOAT(_) => {
                self.advance();
                Ok(Node::NUM(t))
            }
            TokenType::STRING(_) => {
                self.advance();
                Ok(Node::STRING(t))
            }
            TokenType::ID(_) => {
                self.advance();
                Ok(Node::ACCESS(t))
            }
            TokenType::LROUND => {
                self.advance();
                let expr = self.expression()?;
                match self.current_token.token_type {
                    TokenType::RROUND => {
                        self.advance();
                        Ok(expr)
                    }
                    _ => Err(Error::new(
                        ErrType::InvalidSyntaxError,
                        &t.start_pos,
                        &self.current_token.end_pos,
                        format!(
                            "Parser: expected ')' found {:?}",
                            self.current_token.token_type
                        ),
                        None,
                    )),
                }
            }
            TokenType::KEYWRD(Keyword::IF) => self.if_expression(),
            TokenType::KEYWRD(Keyword::WHILE) => self.while_expression(),
            TokenType::KEYWRD(Keyword::FOR) => self.for_expression(),
            TokenType::KEYWRD(Keyword::FUNC) => self.func_expression(),
            _ => Err(Error::new(
                ErrType::InvalidSyntaxError,
                &t.start_pos,
                &t.end_pos,
                format!(
                    "Parser: expected INT, FLOAT, IDENTIFIER, '+', '-' or '(, found: {:?}",
                    t.token_type
                ),
                None,
            )),
        };
    }

    fn power(&mut self) -> Result<Node, Error> {
        self.binary_operation(
            Parser::call,
            vec![TokenType::POW],
            Vec::new(),
            Parser::factor,
        )
    }

    fn call(&mut self) -> Result<Node, Error> {
        let res = self.atom()?;

        if matches!(self.current_token.token_type, TokenType::LROUND) {
            self.advance();
            let mut arg_nodes: Vec<Node> = Vec::new();

            if matches!(self.current_token.token_type, TokenType::RROUND) {
                self.advance();
            } else {
                arg_nodes.push(self.expression()?);

                while matches!(self.current_token.token_type, TokenType::COMMA) {
                    self.advance();
                    arg_nodes.push(self.expression()?);
                }

                if !matches!(self.current_token.token_type, TokenType::RROUND,) {
                    return Err(Error::new(
                        ErrType::InvalidSyntaxError,
                        &self.current_token.start_pos,
                        &self.current_token.end_pos,
                        format!("Parser: expected RROUND found '{:?}'", self.current_token),
                        None,
                    ));
                }

                self.advance();
            }
            Ok(Node::CALL(res.into(), arg_nodes))
        } else {
            Ok(res)
        }
    }

    fn factor(&mut self) -> Result<Node, Error> {
        let t = self.current_token.clone();

        return match t.token_type {
            TokenType::SUB | TokenType::ADD => {
                self.advance();
                let factor = self.factor()?;
                Ok(Node::UNRYOP(t, factor.into()))
            }
            _ => self.power(),
        };
    }

    fn binary_operation(
        &mut self,
        func_a: fn(parser: &mut Parser) -> Result<Node, Error>,
        ops: Vec<TokenType>,
        keywords: Vec<Keyword>,
        func_b: fn(parser: &mut Parser) -> Result<Node, Error>,
    ) -> Result<Node, Error> {
        let mut left_node = func_a(self)?;

        while {
            let mut found = false;
            for t in &ops {
                if match_enum_type(t, &self.current_token.token_type) {
                    found = true;
                    break;
                }
            }
            if !found {
                if let TokenType::KEYWRD(k) = &self.current_token.token_type {
                    for key in &keywords {
                        if match_enum_type(key, &k) {
                            found = true;
                            break;
                        }
                    }
                }
            }
            found
        } {
            let op_token = self.current_token.clone();
            self.advance();
            let right_node = func_b(self)?;

            left_node = Node::BINOP(left_node.into(), op_token, right_node.into());
        }

        Ok(left_node)
    }

    fn expect_token(&self, token: TokenType) -> Result<(), Error> {
        if !match_enum_type(&self.current_token.token_type, &token) {
            Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!(
                    "Parser: expected {:?} found '{:?}'",
                    token, self.current_token
                ),
                None,
            ))
        } else {
            Ok(())
        }
    }

    fn if_expression(&mut self) -> Result<Node, Error> {
        let mut cases: Vec<(Node, Node)> = Vec::new();
        let mut else_case = None;

        if let TokenType::KEYWRD(Keyword::IF) = self.current_token.token_type {
        } else {
            return Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!("Parser: expected IF found '{:?}'", self.current_token),
                None,
            ));
        }

        self.advance();

        let condition = self.expression()?;

        if !match_enum_type(&self.current_token.token_type, &TokenType::LCURLY) {
            return Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!("Parser: expected LCURLY found '{:?}'", self.current_token),
                None,
            ));
        }

        self.advance();
        let expr = self.expression()?;
        cases.push((condition, expr));

        if !matches!(self.current_token.token_type, TokenType::RCURLY) {
            return Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!("Parser: expected RCURLY found '{:?}'", self.current_token),
                None,
            ));
        }
        self.advance();

        while matches!(
            self.current_token.token_type,
            TokenType::KEYWRD(Keyword::ELIF)
        ) {
            self.advance();
            let cond = self.expression()?;

            if !matches!(self.current_token.token_type, TokenType::LCURLY) {
                return Err(Error::new(
                    ErrType::InvalidSyntaxError,
                    &self.current_token.start_pos,
                    &self.current_token.end_pos,
                    format!("Parser: expected LCURLY found '{:?}'", self.current_token),
                    None,
                ));
            }
            self.advance();

            let expr = self.expression()?;
            cases.push((cond, expr));

            if !matches!(self.current_token.token_type, TokenType::RCURLY) {
                return Err(Error::new(
                    ErrType::InvalidSyntaxError,
                    &self.current_token.start_pos,
                    &self.current_token.end_pos,
                    format!("Parser: expected RCURLY found '{:?}'", self.current_token),
                    None,
                ));
            }
            self.advance();
        }

        if matches!(
            self.current_token.token_type,
            TokenType::KEYWRD(Keyword::ELSE)
        ) {
            self.advance();
            if !matches!(self.current_token.token_type, TokenType::LCURLY) {
                return Err(Error::new(
                    ErrType::InvalidSyntaxError,
                    &self.current_token.start_pos,
                    &self.current_token.end_pos,
                    format!("Parser: expected LCURLY found '{:?}'", self.current_token),
                    None,
                ));
            }
            self.advance();

            else_case = Some(Box::new(self.expression()?));

            if !matches!(&self.current_token.token_type, &TokenType::RCURLY) {
                return Err(Error::new(
                    ErrType::InvalidSyntaxError,
                    &self.current_token.start_pos,
                    &self.current_token.end_pos,
                    format!("Parser: expected RCURLY found '{:?}'", self.current_token),
                    None,
                ));
            }

            self.advance();
        }

        Ok(Node::IF(cases, else_case))
    }

    fn func_expression(&mut self) -> Result<Node, Error> {
        let mut start: Option<Position> = None;
        let end: Option<Position>;

        if !matches!(
            self.current_token.token_type,
            TokenType::KEYWRD(Keyword::FUNC)
        ) {
            return Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!("Parser: expected FUNC found '{:?}'", self.current_token),
                None,
            ));
        }

        self.advance();

        let mut var_name: Option<Token> = None;

        if matches!(self.current_token.token_type, TokenType::ID(_)) {
            var_name = Some(self.current_token.clone());
            start = Some(self.current_token.start_pos.clone());
            self.advance();
        }

        self.expect_token(TokenType::LROUND)?;
        self.advance();

        let mut arg_tokens: Vec<Token> = Vec::new();

        if matches!(self.current_token.token_type, TokenType::ID(_),) {
            arg_tokens.push(self.current_token.clone());
            if let None = start {
                start = Some(self.current_token.start_pos.clone());
            }

            self.advance();

            while matches!(self.current_token.token_type, TokenType::COMMA) {
                self.advance();
                self.expect_token(TokenType::ID(String::from("")))?;

                arg_tokens.push(self.current_token.clone());
                self.advance();
            }

            self.expect_token(TokenType::RROUND)?;
        } else {
            self.expect_token(TokenType::RROUND)?;
        }

        self.advance();

        self.expect_token(TokenType::LCURLY)?;

        if let None = start {
            start = Some(self.current_token.start_pos.clone());
        }

        self.advance();

        let body = self.expression()?;

        self.expect_token(TokenType::RCURLY)?;

        end = Some(self.current_token.end_pos.clone());

        self.advance();

        Ok(Node::FUNCDEF(
            var_name,
            arg_tokens,
            body.into(),
            start.unwrap_or(Position::empty()),
            end.unwrap_or(Position::empty()),
        ))
    }

    fn for_expression(&mut self) -> Result<Node, Error> {
        let mut c1: Option<Box<Node>> = None;
        let mut c3: Option<Box<Node>> = None;

        let start: Position;
        let end: Position;

        if !matches!(
            self.current_token.token_type,
            TokenType::KEYWRD(Keyword::FOR)
        ) {
            return Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!("Parser: expected FOR found '{:?}'", self.current_token),
                None,
            ));
        }

        self.advance();

        start = self.current_token.start_pos.clone();
        if !match_enum_type(&self.current_token.token_type, &TokenType::SEMICLN) {
            c1 = Some(self.expression()?.into());
        }

        self.expect_token(TokenType::SEMICLN)?;
        self.advance();

        let c2 = self.expression()?;

        self.expect_token(TokenType::SEMICLN)?;
        self.advance();

        if !match_enum_type(&self.current_token.token_type, &TokenType::LCURLY) {
            c3 = Some(self.expression()?.into());
        }

        self.expect_token(TokenType::LCURLY)?;
        self.advance();

        let body = self.expression()?;

        self.expect_token(TokenType::RCURLY)?;
        end = self.current_token.start_pos.clone();
        self.advance();

        Ok(Node::FOR(c1, c2.into(), c3, body.into(), start, end))
    }

    fn while_expression(&mut self) -> Result<Node, Error> {
        if !matches!(
            self.current_token.token_type,
            TokenType::KEYWRD(Keyword::WHILE)
        ) {
            return Err(Error::new(
                ErrType::InvalidSyntaxError,
                &self.current_token.start_pos,
                &self.current_token.end_pos,
                format!("Parser: expected WHILE found '{:?}'", self.current_token),
                None,
            ));
        }

        let start = self.current_token.start_pos.clone();

        self.advance();
        let cond = self.expression()?;

        self.expect_token(TokenType::LCURLY)?;
        self.advance();

        let body = self.expression()?;

        self.expect_token(TokenType::RCURLY)?;

        let end = self.current_token.end_pos.clone();

        self.advance();

        Ok(Node::WHILE(cond.into(), body.into(), start, end))
    }

    fn arith_expression(&mut self) -> Result<Node, Error> {
        self.binary_operation(
            Parser::term,
            vec![TokenType::ADD, TokenType::SUB],
            Vec::new(),
            Parser::term,
        )
    }

    fn comp_expression(&mut self) -> Result<Node, Error> {
        match self.current_token.token_type {
            TokenType::KEYWRD(Keyword::NOT) => {
                let op = self.current_token.clone();
                self.advance();
                let node = self.comp_expression()?;
                Ok(Node::UNRYOP(op, Box::new(node)))
            }

            _ => match self.binary_operation(
                Parser::arith_expression,
                vec![
                    TokenType::EQUAL,
                    TokenType::INCRMNT,
                    TokenType::DECRMNT,
                    TokenType::NEQUAL,
                    TokenType::LESS,
                    TokenType::LESSEQ,
                    TokenType::GREATER,
                    TokenType::GREATEREQ,
                ],
                Vec::new(),
                Parser::arith_expression,
            ) {
                Ok(node) => Ok(node),
                Err(e) => Err(e),
                //TODO: look at error handling again
                //Err(_) => Err(Error::new(
                //    ErrType::InvalidSyntaxError,
                //    &self.current_token.start_pos,
                //    &self.current_token.end_pos,
                //    format!("Parser: expected INT, FLOAT, IDENTIFIER, '+', '-', '(' or '!'"),
                //    None,
                //)),
            },
        }
    }

    fn term(&mut self) -> Result<Node, Error> {
        self.binary_operation(
            Parser::factor,
            vec![TokenType::MUL, TokenType::DIV],
            Vec::new(),
            Parser::factor,
        )
    }

    fn expression(&mut self) -> Result<Node, Error> {
        match self.current_token.token_type {
            TokenType::ID(_) => {
                let var = self.current_token.clone();
                self.advance();

                match self.current_token.token_type {
                    TokenType::ASSIGN => {
                        self.advance();
                        Ok(Node::ASSIGN(var, Box::new(self.expression()?)))
                    }
                    _ => {
                        self.retreat();
                        self.binary_operation(
                            Parser::comp_expression,
                            Vec::new(),
                            vec![Keyword::AND, Keyword::OR],
                            Parser::comp_expression,
                        )
                    }
                }
            }
            _ => self.binary_operation(
                Parser::comp_expression,
                Vec::new(),
                vec![Keyword::AND, Keyword::OR],
                Parser::comp_expression,
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum NumberType {
    INT(ChInt),
    FLOAT(ChFloat),
}

#[derive(Clone, Debug)]
pub struct ChNone {
    start_pos: Position,
    end_pos: Position,
}

impl HasContext for ChNone {}

impl HasPosition for ChNone {
    fn get_start(&self) -> Position {
        self.start_pos.clone()
    }

    fn get_end(&self) -> Position {
        self.end_pos.clone()
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        self.start_pos = start_pos;
        self.end_pos = end_pos;
    }
}

impl ChOperators for ChNone {
    fn equal(self, other: ChType) -> Result<ChType, Error> {
        Ok(ChBool {
            value: match other {
                ChType::NONE(_) => true,
                _ => false,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn not_equal(self, other: ChType) -> Result<ChType, Error> {
        Ok(ChBool {
            value: match other {
                ChType::NONE(_) => false,
                _ => true,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn is_true(&self) -> bool {
        false
    }
}

impl IsChValue for ChNone {
    fn get_desc(&self) -> String {
        String::from("None")
    }

    fn as_type(self) -> ChType {
        ChType::NONE(self)
    }
}

impl AsNumberType for ChNone {
    fn convert(self) -> Result<NumberType, Error> {
        Err(Error::new(
            ErrType::RuntimeError,
            &self.get_start(),
            &self.get_end(),
            String::from("can not convert 'none' to Numbertype"),
            None,
        ))
    }
}

impl Display for ChNone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "none")
    }
}

pub trait ChOperators {
    fn add(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'add' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn add_equal(self, other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        self.add(other)
        //Err(Error::new(
        //    ErrType::UndefinedOperator,
        //    &self.get_start(),
        //    &self.get_end(),
        //    format!(
        //        "operation 'increment' not defined for type: {}",
        //        self.get_desc()
        //    ),
        //    None,
        //))
    }
    fn sub_equal(self, other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        self.sub(other)
        //Err(Error::new(
        //    ErrType::UndefinedOperator,
        //    &self.get_start(),
        //    &self.get_end(),
        //    format!(
        //        "operation 'decrement' not defined for type: {}",
        //        self.get_desc()
        //    ),
        //    None,
        //))
    }
    fn sub(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'sub' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn mult(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'mult' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn div(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'div' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn pow(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'pow' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn equal(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!(
                "operation 'equal' not defined for type: {}",
                self.get_desc()
            ),
            None,
        ))
    }
    fn not_equal(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!(
                "operation 'not equal' not defined for type: {}",
                self.get_desc()
            ),
            None,
        ))
    }
    fn less(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'less' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn less_equal(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!(
                "operation 'less equal' not defined for type: {}",
                self.get_desc()
            ),
            None,
        ))
    }
    fn greater(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!(
                "operation 'greater' not defined for type: {}",
                self.get_desc()
            ),
            None,
        ))
    }
    fn greater_equal(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!(
                "operation 'greater equal' not defined for type: {}",
                self.get_desc()
            ),
            None,
        ))
    }
    fn and(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'and' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn or(self, _other: ChType) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'or' not defined for type: {}", self.get_desc()),
            None,
        ))
    }
    fn not(self) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!("operation 'not' not defined for type: {}", self.get_desc()),
            None,
        ))
    }

    fn is_true(&self) -> bool
    where
        Self: IsChValue + Sized,
    {
        false
    }

    fn negate(self) -> Result<ChType, Error>
    where
        Self: IsChValue + Sized,
    {
        Err(Error::new(
            ErrType::UndefinedOperator,
            &self.get_start(),
            &self.get_end(),
            format!(
                "operation 'negate' not defined for type: {}",
                self.get_desc()
            ),
            None,
        ))
    }
}

trait IsFunction {
    fn execute(&mut self, args: Vec<ChType>, name: Option<String>) -> Result<ChType, Error>;
}

#[derive(Clone, Debug)]
enum FuncType {
    RUSTFUNC(RustFunc),
    CHRONFUNC(ChronosFunc),
}

#[derive(Clone)]
pub struct RustFunc {
    name: String,
    function: fn(args: Vec<ChType>, name: Option<String>) -> Result<ChType, Error>,
}

impl HasContext for RustFunc {
    fn set_context(&mut self, _context: Rc<RefCell<Context>>) {}
}

impl IsFunction for RustFunc {
    fn execute(&mut self, args: Vec<ChType>, name: Option<String>) -> Result<ChType, Error> {
        (self.function)(args, name)
    }
}

impl Display for RustFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rust function<{}>", self.name)
    }
}

impl Debug for RustFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function implemented in rust")
    }
}

#[derive(Clone, Debug)]
pub struct ChronosFunc {
    name: String,
    args_name: Vec<Token>,
    body: Node,
    start_pos: Position,
    end_pos: Position,
    context: Rc<RefCell<Context>>,
}

impl IsFunction for ChronosFunc {
    fn execute(&mut self, mut args: Vec<ChType>, name: Option<String>) -> Result<ChType, Error> {
        let mut n_context = Context::from_parent(
            format!("<function: {}>", name.unwrap_or(self.name.clone())),
            self.context.clone(),
            self.start_pos.clone(),
        );

        if args.len() != self.args_name.len() {
            return Err(Error::new(
                ErrType::RuntimeError,
                &self.start_pos,
                &self.end_pos,
                format!(
                    "expected {} arguments, found {} in function '{}'",
                    self.args_name.len(),
                    args.len(),
                    self.name,
                ),
                Some(self.context.clone()),
            ));
        }

        for i in 0..args.len() {
            let value = args.get_mut(i).unwrap();
            let n = &self.args_name[i];
            let name = match &n.token_type {
                TokenType::ID(s) => s,
                _ => {
                    return Err(Error::new(
                        ErrType::RuntimeError,
                        &n.start_pos,
                        &n.end_pos,
                        format!("could not resolve {:?} as an argument", n),
                        Some(n_context.clone()),
                    ))
                }
            };
            value.set_context(self.context.clone());
            n_context.borrow_mut().set_mut(&name, value.clone());
        }

        visit_node(&mut self.body, &mut n_context)
    }
}

impl Display for ChronosFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function<{}, {:?}>", self.name, self.args_name)
    }
}

impl HasContext for ChronosFunc {
    fn set_context(&mut self, context: Rc<RefCell<Context>>) {
        self.context = context;
    }
}

impl HasPosition for ChronosFunc {
    fn get_start(&self) -> Position {
        self.start_pos.clone()
    }

    fn get_end(&self) -> Position {
        self.end_pos.clone()
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        self.start_pos = start_pos;
        self.end_pos = end_pos;
    }
}

#[derive(Clone, Debug)]
pub struct ChFunction {
    func_type: FuncType,
}

impl Display for ChFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.func_type {
            FuncType::CHRONFUNC(func) => write!(f, "{}", func),
            FuncType::RUSTFUNC(func) => write!(f, "{}", func),
        }
    }
}

impl HasContext for ChFunction {
    fn set_context(&mut self, context: Rc<RefCell<Context>>) {
        match &mut self.func_type {
            FuncType::CHRONFUNC(func) => func.set_context(context),
            FuncType::RUSTFUNC(func) => func.set_context(context),
        }
    }
}

impl AsNumberType for ChFunction {}

impl HasPosition for ChFunction {
    fn get_start(&self) -> Position {
        match &self.func_type {
            FuncType::CHRONFUNC(func) => func.get_start(),
            FuncType::RUSTFUNC(func) => Position::from_name(func.name.clone()),
        }
    }

    fn get_end(&self) -> Position {
        match &self.func_type {
            FuncType::CHRONFUNC(func) => func.get_end(),
            FuncType::RUSTFUNC(func) => Position::from_name(func.name.clone()),
        }
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        match &mut self.func_type {
            FuncType::CHRONFUNC(func) => func.set_position(start_pos, end_pos),
            FuncType::RUSTFUNC(_) => (),
        }
    }
}

impl IsChValue for ChFunction {
    fn get_desc(&self) -> String {
        String::from("function")
    }

    fn as_type(self) -> ChType {
        ChType::FUNCTION(self)
    }
}

impl ChOperators for ChFunction {
    fn is_true(&self) -> bool {
        true
    }
}

impl IsFunction for ChFunction {
    fn execute(&mut self, args: Vec<ChType>, name: Option<String>) -> Result<ChType, Error> {
        match &mut self.func_type {
            FuncType::CHRONFUNC(func) => func.execute(args, name),
            FuncType::RUSTFUNC(func) => func.execute(args, name),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChBool {
    value: bool,
    start_pos: Position,
    end_pos: Position,
}

impl ChBool {
    fn from(v: bool) -> Self {
        ChBool {
            value: v,
            start_pos: Position::empty(),
            end_pos: Position::empty(),
        }
    }
}

impl HasContext for ChBool {}

impl Display for ChBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.value { "true" } else { "false" })
    }
}

impl HasPosition for ChBool {
    fn get_start(&self) -> Position {
        self.start_pos.clone()
    }

    fn get_end(&self) -> Position {
        self.end_pos.clone()
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        self.start_pos = start_pos;
        self.end_pos = end_pos;
    }
}

impl ChOperators for ChBool {
    fn equal(self, other: ChType) -> Result<ChType, Error> {
        Ok(ChBool {
            value: match other {
                ChType::BOOL(b) => self.value == b.value,
                _ => false,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn not_equal(self, other: ChType) -> Result<ChType, Error> {
        Ok(ChBool {
            value: match other {
                ChType::BOOL(b) => self.value != b.value,
                _ => true,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn not(mut self) -> Result<ChType, Error> {
        self.value = !self.value;
        Ok(self.as_type())
    }

    fn is_true(&self) -> bool {
        self.value
    }

    fn and(self, other: ChType) -> Result<ChType, Error> {
        Ok(ChBool {
            value: self.value && other.is_true(),
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn or(self, other: ChType) -> Result<ChType, Error> {
        Ok(ChBool {
            value: self.value || other.is_true(),
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }
}

impl AsNumberType for ChBool {
    fn as_number_type(self) -> NumberType {
        NumberType::INT(self.value as i32)
    }

    fn convert(self) -> Result<NumberType, Error> {
        Ok(NumberType::INT(self.value as i32))
    }

    fn get_value_type(&self) -> NumberType {
        NumberType::INT(self.value as i32)
    }
}

impl IsChValue for ChBool {
    fn get_desc(&self) -> String {
        String::from("Bool")
    }

    fn as_type(self) -> ChType {
        ChType::BOOL(self)
    }
}

#[derive(Debug, Clone)]
pub struct ChNumber {
    value: NumberType,
    start_pos: Position,
    end_pos: Position,
}

impl HasContext for ChNumber {}

pub trait IsChValue: Display + HasPosition + HasContext + ChOperators + AsNumberType {
    fn get_desc(&self) -> String;
    fn as_type(self) -> ChType;
}

impl IsChValue for ChNumber {
    fn get_desc(&self) -> String {
        String::from("Number")
    }

    fn as_type(self) -> ChType {
        ChType::NUMBER(self)
    }
}

impl Display for ChNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            NumberType::INT(v) => write!(f, "{}", v),
            NumberType::FLOAT(v) => write!(f, "{}", v),
        }
    }
}

pub trait AsNumberType {
    fn as_number_type(self) -> NumberType
    where
        Self: Sized,
    {
        panic!("value can't be converted");
    }
    fn convert(self) -> Result<NumberType, Error>
    where
        Self: Sized + HasPosition,
    {
        Err(Error::new(
            ErrType::RuntimeError,
            &self.get_start(),
            &self.get_end(),
            format!("could not convert to Number"),
            None,
        ))
    }

    fn get_value_type(&self) -> NumberType
    where
        Self: Sized,
    {
        panic!("value can't be converted");
    }
}

pub trait HasPosition {
    fn get_start(&self) -> Position;
    fn get_end(&self) -> Position;
    fn set_position(&mut self, start_pos: Position, end_pos: Position);
}

pub trait HasContext {
    fn set_context(&mut self, _context: Rc<RefCell<Context>>) {}
}

impl AsNumberType for ChType {
    fn as_number_type(self) -> NumberType {
        match self {
            ChType::NUMBER(n) => n.as_number_type(),
            ChType::STRING(s) => s.as_number_type(),
            ChType::BOOL(b) => b.as_number_type(),
            ChType::FUNCTION(f) => f.as_number_type(),
            ChType::NONE(_) => 0.as_number_type(),
        }
    }

    fn convert(self) -> Result<NumberType, Error> {
        match self {
            ChType::NUMBER(n) => Ok(n.as_number_type()),
            ChType::STRING(s) => Ok(s.as_number_type()),
            ChType::BOOL(b) => Ok(b.as_number_type()),
            ChType::FUNCTION(f) => Ok(f.as_number_type()),
            ChType::NONE(_) => Err(Error::new(
                ErrType::RuntimeError,
                &self.get_start(),
                &self.get_end(),
                format!("Could not convert '{}' to Number", self),
                None,
            )),
        }
    }

    fn get_value_type(&self) -> NumberType {
        match self {
            ChType::NUMBER(n) => n.get_value_type(),
            ChType::STRING(s) => s.get_value_type(),
            ChType::BOOL(b) => b.get_value_type(),
            ChType::FUNCTION(_) | ChType::NONE(_) => {
                panic!("could not get value type of type {}", self)
            }
        }
    }
}

impl AsNumberType for bool {
    fn as_number_type(self) -> NumberType {
        NumberType::INT(if self { 1 } else { 0 })
    }

    fn get_value_type(&self) -> NumberType {
        NumberType::INT(if *self { 1 } else { 0 })
    }

    fn convert(self) -> Result<NumberType, Error> {
        Ok(NumberType::INT(if self { 1 } else { 0 }))
    }
}

impl AsNumberType for ChInt {
    fn as_number_type(self) -> NumberType {
        NumberType::INT(self)
    }

    fn get_value_type(&self) -> NumberType {
        NumberType::INT(self.clone())
    }

    fn convert(self) -> Result<NumberType, Error> {
        Ok(NumberType::INT(self))
    }
}

impl AsNumberType for ChFloat {
    fn as_number_type(self) -> NumberType {
        NumberType::FLOAT(self)
    }

    fn get_value_type(&self) -> NumberType {
        NumberType::FLOAT(self.clone())
    }

    fn convert(self) -> Result<NumberType, Error> {
        Ok(NumberType::FLOAT(self))
    }
}

impl AsNumberType for ChNumber {
    fn as_number_type(self) -> NumberType {
        self.value
    }

    fn get_value_type(&self) -> NumberType {
        self.value.clone()
    }

    fn convert(self) -> Result<NumberType, Error> {
        Ok(self.value)
    }
}

impl HasPosition for ChNumber {
    fn get_start(&self) -> Position {
        self.start_pos.clone()
    }

    fn get_end(&self) -> Position {
        self.end_pos.clone()
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        self.start_pos = start_pos;
        self.end_pos = end_pos;
    }
}

impl ChNumber {
    fn from(value: NumberType, _context: &mut Rc<RefCell<Context>>) -> Self {
        ChNumber {
            value,
            start_pos: Position::empty(),
            end_pos: Position::empty(),
        }
    }

    fn as_token(self) -> Token {
        match self.value {
            NumberType::INT(v) => Token::new(TokenType::INT(v), self.start_pos, Some(self.end_pos)),
            NumberType::FLOAT(v) => {
                Token::new(TokenType::FLOAT(v), self.start_pos, Some(self.end_pos))
            }
        }
    }

    fn operate_on(
        mut self,
        other: NumberType,
        int_op: fn(ChInt, ChInt) -> ChInt,
        float_op: fn(ChFloat, ChFloat) -> ChFloat,
    ) -> Self {
        self.value = match (self.value, other) {
            (NumberType::INT(v1), NumberType::INT(v2)) => NumberType::INT(int_op(v1, v2)),
            (NumberType::FLOAT(v1), NumberType::INT(v2)) => {
                NumberType::FLOAT(float_op(v1, v2 as ChFloat))
            }
            (NumberType::INT(v1), NumberType::FLOAT(v2)) => {
                NumberType::FLOAT(float_op(v1 as ChFloat, v2))
            }
            (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => NumberType::FLOAT(float_op(v1, v2)),
        };

        self
    }
}

impl ChOperators for ChNumber {
    fn add(self, other: ChType) -> Result<ChType, Error> {
        Ok(self
            .operate_on(
                other.convert()?,
                |v1: ChInt, v2: ChInt| v1 + v2,
                |v1: ChFloat, v2: ChFloat| v1 + v2,
            )
            .as_type())
    }

    fn sub(self, other: ChType) -> Result<ChType, Error> {
        Ok(self
            .operate_on(
                other.convert()?,
                |v1: ChInt, v2: ChInt| v1 - v2,
                |v1: ChFloat, v2: ChFloat| v1 - v2,
            )
            .as_type())
    }

    fn mult(self, other: ChType) -> Result<ChType, Error> {
        Ok(self
            .operate_on(
                other.convert()?,
                |v1: ChInt, v2: ChInt| v1 * v2,
                |v1: ChFloat, v2: ChFloat| v1 * v2,
            )
            .as_type())
    }

    fn div(self, other: ChType) -> Result<ChType, Error> {
        if match other.get_value_type() {
            NumberType::INT(v) => v == 0,
            NumberType::FLOAT(v) => v == 0.0,
        } {
            return Err(Error::new(
                ErrType::RuntimeError,
                &self.start_pos,
                &self.end_pos,
                String::from("Division by 0"),
                None,
            ));
        } else {
            Ok(self
                .operate_on(
                    other.convert()?,
                    |v1: ChInt, v2: ChInt| v1 / v2,
                    |v1: ChFloat, v2: ChFloat| v1 / v2,
                )
                .as_type())
        }
    }

    fn pow(mut self, other: ChType) -> Result<ChType, Error> {
        if match other.get_value_type() {
            NumberType::INT(v) => v == 0,
            _ => false,
        } {
            self.value = 0.as_number_type();
            Ok(self.as_type())
        } else {
            Ok(self
                .operate_on(
                    other.convert()?,
                    |v1: ChInt, v2: ChInt| v1.pow(v2.try_into().unwrap_or(0)),
                    |v1: ChFloat, v2: ChFloat| v1.powf(v2),
                )
                .as_type())
        }
    }

    fn equal(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert();

        Ok(ChBool {
            value: match value {
                Ok(v) => match (self.value, v) {
                    (NumberType::INT(v1), NumberType::INT(v2)) => v1 == v2,
                    (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 == v2,
                    (NumberType::INT(v1), NumberType::FLOAT(v2)) => v1 as ChFloat == v2,
                    (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 == v2 as ChFloat,
                },
                Err(_) => false,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn not_equal(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert();

        Ok(ChBool {
            value: match value {
                Ok(v) => match (self.value, v) {
                    (NumberType::INT(v1), NumberType::INT(v2)) => v1 != v2,
                    (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 != v2,
                    (NumberType::INT(v1), NumberType::FLOAT(v2)) => v1 as ChFloat != v2,
                    (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 != v2 as ChFloat,
                },
                Err(_) => false,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn less(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert()?;

        Ok(ChBool {
            value: match (self.value, value) {
                (NumberType::INT(v1), NumberType::INT(v2)) => v1 < v2,
                (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 < v2,
                (NumberType::INT(v1), NumberType::FLOAT(v2)) => (v1 as ChFloat) < v2,
                (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 < v2 as ChFloat,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn less_equal(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert()?;

        Ok(ChBool {
            value: match (self.value, value) {
                (NumberType::INT(v1), NumberType::INT(v2)) => v1 <= v2,
                (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 <= v2,
                (NumberType::INT(v1), NumberType::FLOAT(v2)) => (v1 as ChFloat) <= v2,
                (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 <= v2 as ChFloat,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn greater(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert()?;

        Ok(ChBool {
            value: match (self.value, value) {
                (NumberType::INT(v1), NumberType::INT(v2)) => v1 > v2,
                (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 > v2,
                (NumberType::INT(v1), NumberType::FLOAT(v2)) => (v1 as ChFloat) > v2,
                (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 > v2 as ChFloat,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn greater_equal(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert()?;

        Ok(ChBool {
            value: match (self.value, value) {
                (NumberType::INT(v1), NumberType::INT(v2)) => v1 >= v2,
                (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 >= v2,
                (NumberType::INT(v1), NumberType::FLOAT(v2)) => (v1 as ChFloat) >= v2,
                (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 >= v2 as ChFloat,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn and(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert()?;

        Ok(ChBool {
            value: match (self.value, value) {
                (NumberType::INT(v1), NumberType::INT(v2)) => v1 >= 1 && v2 >= 1,
                (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 >= 1.0 && v2 >= 1.0,
                (NumberType::INT(v1), NumberType::FLOAT(v2)) => v1 >= 1 && v2 >= 1.0,
                (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 >= 1.0 && v2 >= 1,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn or(self, other: ChType) -> Result<ChType, Error> {
        let value = other.convert()?;

        Ok(ChBool {
            value: match (self.value, value) {
                (NumberType::INT(v1), NumberType::INT(v2)) => v1 >= 1 || v2 >= 1,
                (NumberType::FLOAT(v1), NumberType::FLOAT(v2)) => v1 >= 1.0 || v2 >= 1.0,
                (NumberType::INT(v1), NumberType::FLOAT(v2)) => v1 >= 1 || v2 >= 1.0,
                (NumberType::FLOAT(v1), NumberType::INT(v2)) => v1 >= 1.0 || v2 >= 1,
            },
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn not(mut self) -> Result<ChType, Error> {
        self.value = match self.value {
            NumberType::INT(value) => if value != 0 { 0 } else { 1 }.as_number_type(),
            NumberType::FLOAT(value) => if value != 0.0 { 0.0 } else { 1.0 }.as_number_type(),
        };
        Ok(ChBool {
            value: self.is_true(),
            start_pos: self.start_pos,
            end_pos: self.end_pos,
        }
        .as_type())
    }

    fn is_true(&self) -> bool {
        match self.value {
            NumberType::INT(value) => value != 0,
            NumberType::FLOAT(value) => value != 0.0,
        }
    }

    fn negate(mut self) -> Result<ChType, Error> {
        match self.value {
            NumberType::INT(v) => self.value = NumberType::INT(-v),
            NumberType::FLOAT(v) => self.value = NumberType::FLOAT(-v),
        }
        Ok(self.as_type())
    }
}

#[derive(Debug, Clone)]
pub struct ChString {
    string: String,
    start_pos: Position,
    end_pos: Position,
}

impl Display for ChString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl HasPosition for ChString {
    fn get_start(&self) -> Position {
        self.start_pos.clone()
    }

    fn get_end(&self) -> Position {
        self.end_pos.clone()
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        self.start_pos = start_pos;
        self.end_pos = end_pos;
    }
}

impl HasContext for ChString {}

impl ChOperators for ChString {
    fn is_true(&self) -> bool {
        self.string.len() != 0
    }

    fn add(mut self, other: ChType) -> Result<ChType, Error> {
        let other_string = match other {
            ChType::NUMBER(n) => format!("{}", n),
            ChType::STRING(s) => s.to_string(),
            _ => {
                return Err(Error::new(
                    ErrType::RuntimeError,
                    &self.start_pos,
                    &self.end_pos,
                    format!("can't add {:?} to String", other),
                    None,
                ))
            }
        };

        self.string += &other_string;
        Ok(ChType::STRING(self))
    }

    fn add_equal(self, other: ChType) -> Result<ChType, Error> {
        self.add(other)
    }
}

impl AsNumberType for ChString {}

impl IsChValue for ChString {
    fn as_type(self) -> ChType {
        ChType::STRING(self)
    }

    fn get_desc(&self) -> String {
        String::from("String")
    }
}

#[derive(Debug, Clone)]
pub enum ChType {
    NUMBER(ChNumber),
    STRING(ChString),
    FUNCTION(ChFunction),
    BOOL(ChBool),
    NONE(ChNone),
}

impl HasContext for ChType {
    fn set_context(&mut self, context: Rc<RefCell<Context>>) {
        match self {
            ChType::FUNCTION(func) => func.set_context(context),
            _ => (),
        }
    }
}

impl Display for ChType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChType::NUMBER(n) => write!(f, "{}", n),
            ChType::STRING(s) => write!(f, "{}", s),
            ChType::NONE(n) => write!(f, "{}", n),
            ChType::BOOL(b) => write!(f, "{}", b),
            ChType::FUNCTION(func) => write!(f, "{}", func),
        }
    }
}

impl HasPosition for ChType {
    fn get_start(&self) -> Position {
        match self {
            ChType::NUMBER(num) => num.start_pos.clone(),
            ChType::STRING(s) => s.start_pos.clone(),
            ChType::BOOL(b) => b.start_pos.clone(),
            ChType::NONE(none) => none.start_pos.clone(),
            ChType::FUNCTION(f) => f.get_start().clone(),
        }
    }

    fn get_end(&self) -> Position {
        match self {
            ChType::NUMBER(num) => num.end_pos.clone(),
            ChType::STRING(s) => s.end_pos.clone(),
            ChType::BOOL(b) => b.end_pos.clone(),
            ChType::FUNCTION(f) => f.get_end().clone(),
            ChType::NONE(none) => none.end_pos.clone(),
        }
    }

    fn set_position(&mut self, start_pos: Position, end_pos: Position) {
        match self {
            ChType::NUMBER(num) => num.set_position(start_pos, end_pos),
            ChType::STRING(s) => s.set_position(start_pos, end_pos),
            ChType::BOOL(b) => b.set_position(start_pos, end_pos),
            ChType::FUNCTION(f) => f.set_position(start_pos, end_pos),
            ChType::NONE(none) => none.set_position(start_pos, end_pos),
        }
    }
}

impl ChType {
    pub fn is_true(&self) -> bool {
        match self {
            ChType::NUMBER(n) => n.is_true(),
            ChType::STRING(s) => s.is_true(),
            ChType::NONE(n) => n.is_true(),
            ChType::BOOL(n) => n.is_true(),
            ChType::FUNCTION(f) => f.is_true(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub display_name: String,
    pub parent: Option<Rc<RefCell<Context>>>,
    pub position: Option<Position>,
    pub symbol_table: SymbolTable,
}

impl Context {
    fn empty(display_name: String) -> Self {
        Context {
            display_name,
            parent: None,
            position: None,
            symbol_table: SymbolTable::empty(),
        }
    }

    fn from_parent(
        display_name: String,
        parent: Rc<RefCell<Context>>,
        position: Position,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Context {
            display_name,
            parent: Some(parent.clone()),
            position: Some(position),
            symbol_table: SymbolTable::empty(),
        }))
    }

    fn set_pos(&mut self, position: Position) {
        match &mut self.position {
            Some(p) => {
                *p = position;
            }
            _ => (),
        }
    }

    fn count_parents(&self) -> i32 {
        if let Some(p) = &self.parent {
            p.borrow().count_parents() + 1
        } else {
            0
        }
    }

    fn get(&self, key: &String) -> Option<ChType> {
        match self.symbol_table.get(key) {
            Some(v) => Some(v.clone()),
            None => match &self.parent {
                Some(p) => p.borrow().get(key),
                None => None,
            },
        }
    }

    fn set_mut(&mut self, key: &String, value: ChType) -> bool {
        self.symbol_table.set_mut(key, value)
    }

    fn set(&mut self, key: &String, value: ChType) -> bool {
        self.symbol_table.set(key, value)
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    table: HashMap<String, ChType>,
    immutable: Vec<String>,
}

impl SymbolTable {
    pub fn empty() -> Self {
        SymbolTable {
            table: HashMap::new(),
            immutable: Vec::new(),
        }
    }

    fn get(&self, key: &String) -> Option<ChType> {
        match self.table.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    fn set_mut(&mut self, key: &String, value: ChType) -> bool {
        if self.table.contains_key(key) {
            if self.immutable.contains(&key) {
                false
            } else {
                *self.table.get_mut(key).unwrap() = value;
                true
            }
        } else {
            self.table.insert(key.to_string(), value);
            true
        }
    }

    fn set(&mut self, key: &String, value: ChType) -> bool {
        let b = self.set_mut(key, value);
        if b {
            self.immutable.push(key.to_string())
        };
        b
    }

    fn remove(&mut self, key: &String) {
        self.table.remove(key);
    }
}

fn visit_node(node: &mut Node, context: &mut Rc<RefCell<Context>>) -> Result<ChType, Error> {
    match node {
        Node::NUM(token) => visit_numb_node(token, context),
        Node::STRING(s) => visit_string_node(s, context),
        Node::UNRYOP(op, node) => visit_unryop_node(op, node, context),
        Node::BINOP(left, op, right) => visit_binop_node(left, op, right, context),
        Node::ACCESS(id) => visit_access_node(id, context),
        Node::ASSIGN(id, value) => visit_assign_node(id, value, context),
        Node::IF(cases, else_case) => visit_if_node(cases, else_case, context),
        Node::WHILE(cond, body, start, end) => visit_while_node(cond, body, context, start, end),
        Node::FOR(c1, c2, c3, body, start, end) => {
            visit_for_node(c1, c2, c3, body, context, start, end)
        }
        Node::FUNCDEF(name, args, body, start, end) => {
            visit_funcdef_node(name, args, body, start, end, context)
        }
        Node::CALL(name, args) => visit_call_node(name, args, context),
    }
}

fn visit_numb_node(
    token: &mut Token,
    _context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    match token.token_type {
        TokenType::INT(value) => Ok(ChType::NUMBER(ChNumber {
            value: value.as_number_type(),
            start_pos: token.start_pos.clone(),
            end_pos: token.end_pos.clone(),
        })),
        TokenType::FLOAT(value) => Ok(ChType::NUMBER(ChNumber {
            value: value.as_number_type(),
            start_pos: token.start_pos.clone(),
            end_pos: token.end_pos.clone(),
        })),
        _ => panic!("called visit_numb_node on a number node that has a non number token"),
    }
}

fn visit_string_node(
    token: &mut Token,
    _context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    match &token.token_type {
        TokenType::STRING(s) => Ok(ChType::STRING(ChString {
            string: s.to_string(),
            start_pos: token.start_pos.clone(),
            end_pos: token.end_pos.clone(),
        })),
        _ => panic!("called visit_string_node on a string node that has a non string token"),
    }
}

fn visit_access_node(
    token: &mut Token,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let var = &token.token_type;
    match var {
        TokenType::ID(var_name) => {
            let mut entry = context.borrow().get(&var_name);

            match &mut entry {
                Some(num) => {
                    num.set_position(token.start_pos.clone(), token.end_pos.clone());
                    num.set_context(context.clone());
                    Ok(num.clone())
                }
                None => Err(Error::new(
                    ErrType::RuntimeError,
                    &token.start_pos,
                    &token.end_pos,
                    format!("{:?} is not defined", var_name),
                    Some(context.clone()),
                )),
            }
        }
        _ => panic!("called visit_access_node on a non ID token"),
    }
}

fn visit_assign_node(
    id: &mut Token,
    value: &mut Node,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let t = id.clone();
    let ch_type = visit_node(value, context)?;

    match t.token_type {
        TokenType::ID(var_name) => {
            if !context.borrow_mut().set_mut(&var_name, ch_type.clone()) {
                return Err(Error::new(
                    ErrType::RuntimeError,
                    &ch_type.get_start(),
                    &ch_type.get_end(),
                    format!("cannot assign {} to const {:?}", ch_type, var_name),
                    Some(context.clone()),
                ));
            }
            Ok(ch_type)
        }
        _ => panic!("called visit_assign_node on {:?}", value),
    }
}

fn unryop_chvalue<T: IsChValue>(op_token: &Token, value: T) -> Result<ChType, Error> {
    match op_token.token_type {
        TokenType::SUB => value.negate(),
        TokenType::KEYWRD(Keyword::NOT) => value.not(),
        _ => panic!("called unryop_self on {:?}", op_token),
    }
}

fn visit_unryop_node(
    op: &mut Token,
    node: &mut Node,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let mut ch_type = visit_node(node, context)?;
    ch_type.set_position(op.start_pos.clone(), ch_type.get_end());

    match ch_type {
        ChType::NUMBER(n) => unryop_chvalue(op, n),
        ChType::STRING(s) => unryop_chvalue(op, s),
        ChType::BOOL(n) => unryop_chvalue(op, n),
        ChType::FUNCTION(n) => unryop_chvalue(op, n),
        ChType::NONE(n) => unryop_chvalue(op, n),
    }
}

fn binop_chvalue<T: IsChValue>(left: T, op_token: &Token, right: ChType) -> Result<ChType, Error> {
    match op_token.token_type {
        TokenType::ADD => left.add(right),
        TokenType::SUB => left.sub(right),
        TokenType::MUL => left.mult(right),
        TokenType::DIV => left.div(right),
        TokenType::POW => left.pow(right),
        TokenType::LESS => left.less(right),
        TokenType::EQUAL => left.equal(right),
        TokenType::NEQUAL => left.not_equal(right),
        TokenType::LESSEQ => left.less_equal(right),
        TokenType::GREATER => left.greater(right),
        TokenType::GREATEREQ => left.greater_equal(right),
        TokenType::KEYWRD(Keyword::AND) => left.and(right),
        TokenType::KEYWRD(Keyword::OR) => left.or(right),
        _ => panic!("called binop_bool on {:?}", op_token.token_type),
    }
}

fn visit_binop_node(
    left: &mut Node,
    op: &mut Token,
    right: &mut Node,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    if matches!(op.token_type, TokenType::INCRMNT) || matches!(op.token_type, TokenType::DECRMNT) {
        return in_de_crement(left, op, right, context);
    }
    let mut left = visit_node(left, context)?;
    let right = visit_node(right, context)?;

    left.set_position(left.get_start(), right.get_end());

    match match left {
        ChType::NUMBER(n) => binop_chvalue(n, op, right),
        ChType::STRING(s) => binop_chvalue(s, op, right),
        ChType::NONE(n) => binop_chvalue(n, op, right),
        ChType::FUNCTION(n) => binop_chvalue(n, op, right),
        ChType::BOOL(n) => binop_chvalue(n, op, right),
    } {
        Ok(res) => Ok(res),
        Err(mut e) => {
            e.set_context(context.clone());
            Err(e)
        }
    }
}

fn in_de_crement(
    left_node: &mut Node,
    op: &mut Token,
    right_node: &mut Node,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let mut left = visit_node(left_node, context)?;
    let right = visit_node(right_node, context)?;
    let start = left.get_start();
    let end = left.get_end();

    match left_node {
        Node::ACCESS(var_name) => {
            left.set_position(left.get_start(), right.get_end());

            match op.token_type {
                TokenType::INCRMNT => {
                    let res = match left {
                        ChType::NUMBER(n) => n.add_equal(right),
                        ChType::STRING(s) => s.add_equal(right),
                        ChType::NONE(n) => n.add_equal(right),
                        ChType::FUNCTION(func) => func.add_equal(right),
                        ChType::BOOL(b) => b.add_equal(right),
                    }?;

                    let name = match &var_name.token_type {
                        TokenType::ID(n) => n,
                        _ => panic!("could not resolve name"),
                    };

                    context.borrow_mut().set_mut(name, res.clone());
                    Ok(res)
                }
                TokenType::DECRMNT => {
                    let res = match left {
                        ChType::NUMBER(n) => n.sub_equal(right),
                        ChType::STRING(s) => s.sub_equal(right),
                        ChType::NONE(n) => n.sub_equal(right),
                        ChType::FUNCTION(func) => func.sub_equal(right),
                        ChType::BOOL(b) => b.sub_equal(right),
                    }?;

                    let name = match &var_name.token_type {
                        TokenType::ID(n) => n,
                        _ => panic!("could not resolve name"),
                    };

                    context.borrow_mut().set_mut(name, res.clone());
                    Ok(res)
                }
                _ => panic!("called in/decrement on {:?}", op),
            }
        }
        _ => Err(Error::new(
            ErrType::RuntimeError,
            &start,
            &end,
            format!("expected LVALUE, found {:?}", left_node),
            Some(context.clone()),
        )),
    }
}

fn visit_if_node(
    cases: &mut Vec<(Node, Node)>,
    else_case: &mut Option<Box<Node>>,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let mut start = Position::empty();
    let mut end = Position::empty();
    let mut first_cond = true;

    for (condition, expr) in cases {
        let cond = visit_node(condition, context)?;

        if first_cond {
            start = cond.get_start();
            end = cond.get_end();
            first_cond = false;
        }

        if cond.is_true() {
            return visit_node(expr, context);
        }
    }

    match else_case {
        Some(node) => visit_node(node, context),
        _ => Ok(ChType::NONE(ChNone {
            start_pos: start,
            end_pos: end,
        })),
    }
}

fn visit_for_node(
    c1: &mut Option<Box<Node>>,
    c2: &mut Box<Node>,
    c3: &mut Option<Box<Node>>,
    body: &mut Node,
    context: &mut Rc<RefCell<Context>>,
    start: &mut Position,
    end: &mut Position,
) -> Result<ChType, Error> {
    let mut n_context = Context::from_parent(String::from("<for>"), context.clone(), start.clone());

    if let Some(c) = c1 {
        visit_node(c, &mut n_context)?;
    }

    while visit_node(c2, &mut n_context)?.is_true() {
        visit_node(body, &mut n_context)?;
        if let Some(c) = c3 {
            visit_node(c, &mut n_context)?;
        }
    }

    Ok(ChType::NONE(ChNone {
        start_pos: start.clone(),
        end_pos: end.clone(),
    }))
}

fn visit_while_node(
    condition: &mut Node,
    body: &mut Node,
    context: &mut Rc<RefCell<Context>>,
    start: &mut Position,
    end: &mut Position,
) -> Result<ChType, Error> {
    let mut n_context =
        Context::from_parent(String::from("<while>"), context.clone(), start.clone());

    while visit_node(condition, context)?.is_true() {
        visit_node(body, &mut n_context)?;
    }

    Ok(ChType::NONE(ChNone {
        start_pos: start.clone(),
        end_pos: end.clone(),
    }))
}

fn visit_funcdef_node(
    func_name: &mut Option<Token>,
    args: &mut Vec<Token>,
    body: &mut Node,
    start: &mut Position,
    end: &mut Position,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let name = match func_name {
        Some(tok) => match &tok.token_type {
            TokenType::ID(s) => s,
            _ => {
                return Err(Error::new(
                    ErrType::InvalidSyntaxError,
                    &start,
                    &end,
                    format!("expected ID found '{:?}'", tok),
                    Some(context.clone()),
                ))
            }
        },
        _ => "lambda",
    }
    .to_string();

    let func = ChType::FUNCTION(ChFunction {
        func_type: FuncType::CHRONFUNC(ChronosFunc {
            name: name.clone(),
            args_name: args.clone(),
            body: body.clone(),
            start_pos: start.clone(),
            end_pos: end.clone(),
            context: context.clone(),
        }),
    });

    if let Some(_) = func_name {
        context.borrow_mut().set_mut(&name, func.clone());
    }

    Ok(func)
}

fn visit_call_node(
    func_name: &mut Node,
    args: &mut Vec<Node>,
    context: &mut Rc<RefCell<Context>>,
) -> Result<ChType, Error> {
    let name = match func_name {
        Node::ACCESS(tok) => {
            if let TokenType::ID(s) = &tok.token_type {
                Some(s.to_string())
            } else {
                None
            }
        }
        _ => None,
    };

    let c = visit_node(func_name, context)?;

    let mut call = match c {
        ChType::FUNCTION(func) => func,
        _ => {
            return Err(Error::new(
                ErrType::RuntimeError,
                &c.get_start(),
                &c.get_end(),
                format!("expected FUNCTION found {}", c),
                Some(context.clone()),
            ))
        }
    }
    .clone();

    let mut arg_values: Vec<ChType> = Vec::new();

    for arg in args {
        arg_values.push(visit_node(arg, context)?);
    }

    call.set_context(context.clone());
    call.execute(arg_values, name)
}

fn ch_print(args: Vec<ChType>, _name: Option<String>) -> Result<ChType, Error> {
    let ret = ChType::NONE(ChNone {
        start_pos: Position::empty(),
        end_pos: Position::empty(),
    });
    if args.len() == 0 {
        return Ok(ret);
    }

    let mut it = args.iter();
    let first = it.next();
    print!("{}", first.unwrap());

    for arg in it {
        print!(", {}", arg);
    }
    println!();

    Ok(ret)
}

fn ch_len(args: Vec<ChType>, _name: Option<String>) -> Result<ChType, Error> {
    let mut start = Position::empty();
    let mut end = Position::empty();

    if args.len() != 1 {
        return Err(Error::new(
            ErrType::RuntimeError,
            &start,
            &end,
            format!("Expected 1 argument found: {}", args.len()),
            None,
        ));
    }

    let arg = args.first().unwrap();
    start = arg.get_start();
    end = arg.get_end();

    match arg {
        ChType::STRING(s) => Ok(ChType::NUMBER(ChNumber {
            value: (s.string.len() as i32).as_number_type(),
            start_pos: start,
            end_pos: end,
        })),
        _ => Err(Error::new(
            ErrType::RuntimeError,
            &start,
            &end,
            format!("{} has no len", arg),
            None,
        )),
    }
}

pub struct Compiler {
    pub global_context: Rc<RefCell<Context>>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut table = SymbolTable::empty();
        table.set(&String::from("false"), ChType::BOOL(ChBool::from(false)));
        table.set(&String::from("true"), ChType::BOOL(ChBool::from(true)));
        table.set(
            &String::from("none"),
            ChType::NONE(ChNone {
                start_pos: Position::empty(),
                end_pos: Position::empty(),
            }),
        );

        table.set(
            &String::from("print"),
            ChType::FUNCTION(ChFunction {
                func_type: FuncType::RUSTFUNC(RustFunc {
                    name: "print".to_string(),
                    function: ch_print,
                }),
            }),
        );

        table.set(
            &String::from("len"),
            ChType::FUNCTION(ChFunction {
                func_type: FuncType::RUSTFUNC(RustFunc {
                    name: "len".to_string(),
                    function: ch_len,
                }),
            }),
        );

        Compiler {
            global_context: Rc::new(RefCell::new(Context {
                display_name: "<module>".to_string(),
                parent: None,
                position: None,
                symbol_table: table,
            })),
        }
    }

    pub fn interpret(&mut self, file_name: String, text: String) -> Result<ChType, Error> {
        let mut lexer = Lexer::new(file_name, text);
        let tokens = lexer.parse_tokens()?;
        let mut parser = Parser::new(tokens);
        let mut ast = parser.parse()?;

        visit_node(&mut ast, &mut self.global_context)
    }
}
