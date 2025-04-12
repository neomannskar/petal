use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::front::ast::Ast;
use crate::front::token::Token;

use super::nodes::expr::{BinaryExpr, Expr};
use super::nodes::function::{
    FunctionBody, FunctionDefinition, FunctionParameter, FunctionReturnType, Return,
};

use super::nodes::node::Node;
use super::nodes::operator::Operator;
use super::nodes::r#type::{BasicType, Type};
use super::semantic::SemanticContext;
use super::token::Position;

macro_rules! here {
    () => {
        println!(
            "Execution passed through here:\n\tfile: {}\n\tline: {}",
            file!(),
            line!()
        )
    };
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken {
        token: Token,
        file: String,
        position: Position,
    },
    MissingToken {
        expected: String,
        file: String,
        position: Position,
    },
    SyntaxError {
        message: String,
        file: String,
        position: Position,
    },
    InvalidParameter {
        message: String,
        file: String,
        position: Position,
    },
    GenericError(String),
}

use std::fmt;

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken {
                token,
                file,
                position,
            } => {
                write!(
                    f,
                    "Unexpected token '{:?}' in file: {} on line {} at position {}",
                    token, file, position.line, position.index
                )
            }
            ParserError::MissingToken {
                expected,
                file,
                position,
            } => {
                write!(
                    f,
                    "Missing token '{}', expected in file: {} on line {} at position {}",
                    expected, file, position.line, position.index
                )
            }
            ParserError::SyntaxError {
                message,
                file,
                position,
            } => {
                write!(
                    f,
                    "Syntax error in file {} on line {} at position {}: {}",
                    file, position.line, position.line, message
                )
            }
            ParserError::InvalidParameter {
                message,
                file,
                position,
            } => {
                write!(
                    f,
                    "Invalid parameter: {} in file {} on line {} at position {}",
                    message, file, position.line, position.index
                )
            }
            ParserError::GenericError(message) => {
                write!(f, "Error: {}", message)
            }
        }
    }
}

pub struct Parser {
    file: String,
    tokens: Vec<(Token, Position)>,
    position: usize,
}

impl Parser {
    pub fn new(file: String, tokens: Vec<(Token, Position)>) -> Self {
        Parser {
            file,
            tokens: tokens.to_vec(),
            position: 0,
        }
    }

    pub fn parse(&mut self, ctx: &mut SemanticContext) -> Result<Box<Ast>, ParserError> {
        let mut ast = Box::new(Ast::new());

        while let Ok((token, pos)) = self.consume() {
            match token {
                Token::Fn => {
                    match self.parse_fn(ctx) {
                        Ok(func) => {
                            ast.push_child(Box::new(func));
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                    // Add the parsed function to the AST
                }
                token => {
                    // Skip unexpected tokens or handle other cases
                    println!("Token: {:?} on line {} at index {}", token, pos.line, pos.index);
                    todo!("[token] parse()")
                }
            }
        }

        Ok(ast)
    }

    pub fn parse_fn<'a>(&mut self, ctx: &mut SemanticContext) -> Result<FunctionDefinition, ParserError> {
        // Expect a function name
        let func_name = match self.consume() {
            Ok((Token::Identifier(name), _)) => name.clone(),
            Ok((token, pos)) => {
                return Err(ParserError::UnexpectedToken {
                    token: token,
                    file: self.file.clone(),
                    position: pos,
                })
            }
            Err(e) => return Err(e),
        };

        let parameters = match self.parse_fn_parameters(ctx) {
            Ok(params) => params,
            Err(e) => {
                return Err(e);
            }
        };

        // Parse return type
        let return_type = match self.parse_fn_return_type() {
            Ok(ret) => ret,
            Err(e) => {
                // Change later
                return Err(e);
            }
        };

        // Parse the function body
        let body = match self.parse_fn_body(ctx) {
            Ok(bod) => bod,
            Err(e) => {
                // Change later
                return Err(e);
            }
        };

        Ok(FunctionDefinition {
            id: func_name,
            parameters,
            return_type,
            body: Box::new(body),
        })
    }

    fn parse_fn_parameters(&mut self, ctx: &mut SemanticContext) -> Result<Vec<FunctionParameter>, ParserError> {
        let mut parameters = Vec::new();

        // Expect an opening parenthesis.
        match self.consume()? {
            (Token::LPar, _) => {
                // If the next token is a right parenthesis immediately, it's an empty parameter list.
                if let Some((Token::RPar, _)) = self.peek() {
                    self.consume()?; // Consume the closing parenthesis.
                    return Ok(parameters);
                }

                loop {
                    // Parse the parameter name: it must be an identifier.
                    let (token, pos) = self.consume()?;
                    let param_name = match token {
                        Token::Identifier(name) => name,
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                token,
                                file: self.file.clone(),
                                position: pos,
                            });
                        }
                    };

                    // Expect a colon ':' after the parameter name.
                    let (colon_token, pos) = self.consume()?;
                    if colon_token != Token::Colon {
                        return Err(ParserError::SyntaxError {
                            message: "Expected ':' after parameter name.".to_string(),
                            file: self.file.clone(),
                            position: pos,
                        });
                    }

                    // Parse the parameter type.
                    let (type_token, pos) = self.consume()?;
                    let param_type = match type_token {
                        Token::Identifier(type_name) => Type {
                            name: type_name.clone(),
                            basic: None,
                        },
                        Token::I32 => Type {
                            name: "i32".to_string(),
                            basic: Some(BasicType::I32),
                        },
                        // Add more type tokens as needed.
                        _ => {
                            return Err(ParserError::MissingToken {
                                expected: "parameter type".to_string(),
                                file: self.file.clone(),
                                position: pos,
                            });
                        }
                    };

                    // Add the parameter to our collection.
                    parameters.push(FunctionParameter {
                        id: param_name,
                        r#type: param_type,
                    });

                    // Peek at the next token to decide if another parameter follows.
                    if let Some((next_token, pos)) = self.peek() {
                        match next_token {
                            Token::Comma => {
                                // Consume the comma and continue with the next parameter.
                                self.consume()?;
                            }
                            Token::RPar => {
                                // Consume the closing parenthesis and break out of the loop.
                                self.consume()?;
                                break;
                            }
                            _ => {
                                return Err(ParserError::UnexpectedToken {
                                    token: next_token,
                                    file: self.file.clone(),
                                    // Here, we clone self.position as a placeholder. You may want to improve this.
                                    position: pos,
                                });
                            }
                        }
                    } else {
                        return Err(ParserError::GenericError(String::from("',' or ')'")));
                    }
                }
            }
            (token, pos) => {
                return Err(ParserError::MissingToken {
                    expected: "opening parenthesis '('".to_string(),
                    file: self.file.clone(),
                    position: pos,
                });
            }
        }

        Ok(parameters)
    }

    fn parse_fn_return_type(&mut self) -> Result<FunctionReturnType, ParserError> {
        let mut return_type = FunctionReturnType(Type {
            name: "void".to_string(),
            basic: Some(BasicType::Void),
        });

        match self.consume() {
            Ok((Token::Arrow, _)) => match self.consume() {
                Ok((Token::I32, _)) => {
                    return_type.0 = Type {
                        name: "i32".to_string(),
                        basic: Some(BasicType::I32),
                    };
                }
                x => {
                    dbg!(x);
                    todo!("[x] parse_fn_return_type()");
                }
            },
            Ok((Token::Semicolon, _)) => {
                return Ok(return_type);
            }
            Ok((Token::LCurl, _)) => {
                return Ok(return_type);
            }
            Ok((token, _)) => {
                dbg!(token);
                todo!("[Some(x)] parse_fn_return_type()")
            }
            Err(e) => {
                println!("{:?}", e);
                /* return Err(ParserError::MissingToken {
                    expected: String::from("'->' or '{' or ';'"),
                    file: self.file.clone(),
                    position: pos,
                }); */

                return Err(e);
            }
        }

        return Ok(return_type);
    }

    fn parse_fn_body(&mut self, ctx: &mut SemanticContext) -> Result<FunctionBody, ParserError> {
        let mut body = FunctionBody {
            children: Vec::new(),
        };

        if let Ok((Token::LCurl, _)) = self.current() {
            loop {
                match self.consume() {
                    Ok((Token::RCurl, _)) => break,
                    Ok(_) => {
                        let statement = self.parse_statement(ctx)?;
                        body.children.push(statement);
                    }
                    Err(e) => {
                        return Err(ParserError::GenericError(String::from(
                            "Unexpected end of input in function body.",
                        )))
                    }
                }
            }
        }

        Ok(body)
    }

    fn parse_fn_call(&mut self, ctx: &mut SemanticContext, function_id: String) -> Result<Expr, ParserError> {
        // Consume the left parenthesis. We already know the next token is LPar.
        let (lpar, pos) = self.consume()?;
        if lpar != Token::LPar {
            return Err(ParserError::SyntaxError {
                message: "Expected '(' after function name".to_string(),
                file: self.file.clone(),
                position: pos,
            });
        }
        
        let mut arguments = Vec::new();
        
        // If the next token is immediately a right parenthesis, then there are no arguments.
        if let Some((Token::RPar, _)) = self.peek() {
            self.consume()?; // Consume RPar
            return Ok(Expr::FunctionCall { 
                function: function_id, 
                arguments,
            });
        }
        
        // Otherwise, loop to parse arguments.
        loop {
            // Parse an expression argument.
            let arg = self.parse_expression(ctx)?;
            arguments.push(arg);
            
            // Peek at the next token to decide what to do.
            if let Some((next_token, pos)) = self.peek() {
                match next_token {
                    Token::Comma => {
                        self.consume()?; // Consume the comma and continue
                    }
                    Token::RPar => {
                        self.consume()?; // Consume the closing parenthesis and exit the loop.
                        break;
                    }
                    _ => {
                        return Err(ParserError::SyntaxError {
                            message: "Expected ',' or ')' in function call".to_string(),
                            file: self.file.clone(),
                            position: pos, // or better, use the position from peek
                        });
                    }
                }
            } else {
                return Err(ParserError::MissingToken {
                    expected: "',' or ')' in function call".to_string(),
                    file: self.file.clone(),
                    position: pos,
                });
            }
        }
        
        Ok(Expr::FunctionCall {
            function: function_id,
            arguments,
        })
    }
    

    // --- Expression Parsing Functions ---

    /// Parses an expression, handling addition and subtraction.
    fn parse_expression(&mut self, ctx: &mut SemanticContext) -> Result<Expr, ParserError> {
        let mut expr = self.parse_term(ctx)?;
        while let Some((token, _)) = self.peek() {
            match token {
                Token::Plus | Token::Minus => {
                    // Consume the operator.
                    let (op_token, _) = self.consume()?;
                    // Parse the right-hand side.
                    let right = self.parse_term(ctx)?;
                    let op = match op_token {
                        Token::Plus => Operator::Plus,
                        Token::Minus => Operator::Minus,
                        _ => unreachable!(),
                    };
                    expr = Expr::Binary(Box::new(BinaryExpr {
                        op,
                        left: expr,
                        right,
                    }));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Parses a term, handling multiplication, division, and modulus.
    fn parse_term(&mut self, ctx: &mut SemanticContext) -> Result<Expr, ParserError> {
        let mut expr = self.parse_factor(ctx)?;
        while let Some((token, _)) = self.peek() {
            match token {
                Token::Asterisk | Token::Fslash | Token::Percent => {
                    let (op_token, _) = self.consume()?; // consume the operator
                    let right = self.parse_factor(ctx)?;
                    let op = match op_token {
                        Token::Asterisk => Operator::Asterisk,
                        Token::Fslash => Operator::Fslash,
                        Token::Percent => Operator::Percent,
                        _ => unreachable!(),
                    };
                    expr = Expr::Binary(Box::new(BinaryExpr {
                        op,
                        left: expr,
                        right,
                    }));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Parses a factor: a number, an identifier, or a parenthesized expression.
    fn parse_factor(&mut self, ctx: &mut SemanticContext) -> Result<Expr, ParserError> {
        let (token, pos) = self.consume()?;
        match token {
            Token::Number(num) => {
                Ok(Expr::Number(num.parse::<i64>().unwrap()))
            }
            Token::Identifier(name) => {
                // Create an Identifier (often you'll do more error checking here)

                // If the next token is an LPar then this is a function call.
                if let Some((next_token, _)) = self.peek() {
                    if next_token == Token::LPar {
                        return self.parse_fn_call(ctx, name);
                    }
                }
                // Otherwise it is just a variable/identifier reference.
                Ok(Expr::Identifier(name))
            }
            Token::LPar => {
                // Parenthesized expression
                let expr = self.parse_expression(ctx)?;
                match self.consume()? {
                    (Token::RPar, _) => Ok(expr),
                    (unexpected, pos) => Err(ParserError::UnexpectedToken {
                        token: unexpected,
                        file: self.file.clone(),
                        position: pos,
                    }),
                }
            }
            _ => Err(ParserError::UnexpectedToken {
                token,
                file: self.file.clone(),
                position: pos,
            }),
        }
    }

    fn parse_statement(&mut self, ctx: &mut SemanticContext) -> Result<Box<dyn Node>, ParserError> {
        let (token, pos) = self.consume()?;
        match token {
            Token::Ret => {
                // Parse an expression for the return statement.
                let expr = self.parse_expression(ctx)?;
                // Expect a semicolon after the expression.
                match self.consume()? {
                    (Token::Semicolon, _) => Ok(Box::new(Return { value: expr })),
                    (_unexpected, pos) => Err(ParserError::SyntaxError {
                        message: "Expected ';' after return expression.".to_string(),
                        file: self.file.clone(),
                        position: pos,
                    }),
                }
            }
            // You can add more statement kinds here.
            token => Err(ParserError::UnexpectedToken {
                token,
                file: self.file.clone(),
                position: pos,
            }),
        }
    }

    fn peek(&self) -> Option<(Token, Position)> {
        self.tokens.get(self.position).cloned()
    }

    fn expect(&self, t: Token) -> Result<bool, ParserError> {
        if let Some(tok) = self.tokens.get(self.position + 1) {
            if tok.0 == t {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(ParserError::GenericError(
                "End of program reached (no more tokens)".to_string(),
            ))
        }
    }

    fn current(&self) -> Result<(Token, Position), ParserError> {
        if let Some((token, pos)) = self.tokens.get(self.position).cloned() {
            match token {
                Token::Eof => Err(ParserError::UnexpectedToken {
                    token: Token::Eof,
                    file: self.file.clone(),
                    position: pos.clone(),
                }),
                _ => Ok((token, pos)),
            }
        } else {
            Err(ParserError::GenericError(String::from("Reached end of Vec<(Token, Position)> for unknown reason, it should have stopped at `Token::Eof`")))
        }
    }

    // Helper method to consume the current token and advance the position
    fn consume(&mut self) -> Result<(Token, Position), ParserError> {
        if let Some((token, pos)) = self.tokens.get(self.position).cloned() {
            match token {
                Token::Eof => Err(ParserError::UnexpectedToken {
                    token: Token::Eof,
                    file: self.file.clone(),
                    position: pos.clone(),
                }),
                _ => {
                    self.position += 1;
                    Ok((token, pos))
                }
            }
        } else {
            Err(ParserError::GenericError(String::from("Reached end of Vec<(Token, Position)> for unknown reason, it should have stopped at `Token::Eof`")))
        }
    }
}
