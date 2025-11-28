use crate::error::{ConfigError, ParseResult};
use std::collections::HashMap;

/// Expression evaluator for arithmetic expressions
pub struct ExpressionEvaluator {
    variables: HashMap<String, i64>,
}

impl ExpressionEvaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: i64) {
        self.variables.insert(name, value);
    }

    /// Evaluate an expression string
    pub fn evaluate(&self, expr: &str) -> ParseResult<i64> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(ConfigError::expression(expr, "empty expression"));
        }

        self.parse_expression(expr)
    }

    fn parse_expression(&self, input: &str) -> ParseResult<i64> {
        // Parse addition and subtraction (lowest precedence)
        let mut tokens = self.tokenize(input)?;
        self.parse_additive(&mut tokens)
    }

    fn parse_additive(&self, tokens: &mut Vec<Token>) -> ParseResult<i64> {
        let mut result = self.parse_multiplicative(tokens)?;

        while !tokens.is_empty() {
            match tokens.first() {
                Some(Token::Plus) => {
                    tokens.remove(0);
                    let right = self.parse_multiplicative(tokens)?;
                    result = result
                        .checked_add(right)
                        .ok_or_else(|| ConfigError::expression("", "integer overflow"))?;
                }
                Some(Token::Minus) => {
                    tokens.remove(0);
                    let right = self.parse_multiplicative(tokens)?;
                    result = result
                        .checked_sub(right)
                        .ok_or_else(|| ConfigError::expression("", "integer overflow"))?;
                }
                _ => break,
            }
        }

        Ok(result)
    }

    fn parse_multiplicative(&self, tokens: &mut Vec<Token>) -> ParseResult<i64> {
        let mut result = self.parse_primary(tokens)?;

        while !tokens.is_empty() {
            match tokens.first() {
                Some(Token::Multiply) => {
                    tokens.remove(0);
                    let right = self.parse_primary(tokens)?;
                    result = result
                        .checked_mul(right)
                        .ok_or_else(|| ConfigError::expression("", "integer overflow"))?;
                }
                Some(Token::Divide) => {
                    tokens.remove(0);
                    let right = self.parse_primary(tokens)?;
                    if right == 0 {
                        return Err(ConfigError::expression("", "division by zero"));
                    }
                    result = result
                        .checked_div(right)
                        .ok_or_else(|| ConfigError::expression("", "integer overflow"))?;
                }
                _ => break,
            }
        }

        Ok(result)
    }

    fn parse_primary(&self, tokens: &mut Vec<Token>) -> ParseResult<i64> {
        if tokens.is_empty() {
            return Err(ConfigError::expression("", "unexpected end of expression"));
        }

        let token = tokens.remove(0);
        match token {
            Token::Number(n) => Ok(n),
            Token::Variable(name) => self
                .variables
                .get(&name)
                .copied()
                .ok_or_else(|| ConfigError::variable_not_found(&name)),
            Token::LeftParen => {
                let result = self.parse_additive(tokens)?;
                if tokens.is_empty() || !matches!(tokens.first(), Some(Token::RightParen)) {
                    return Err(ConfigError::expression("", "missing closing parenthesis"));
                }
                tokens.remove(0); // consume )
                Ok(result)
            }
            _ => Err(ConfigError::expression(
                "",
                format!("unexpected token: {:?}", token),
            )),
        }
    }

    fn tokenize(&self, input: &str) -> ParseResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next();
                }
                '+' => {
                    chars.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    chars.next();
                    // Check if this is a negative number
                    if chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        let num = self.read_number(&mut chars, true)?;
                        tokens.push(Token::Number(num));
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => {
                    chars.next();
                    tokens.push(Token::Multiply);
                }
                '/' => {
                    chars.next();
                    tokens.push(Token::Divide);
                }
                '(' => {
                    chars.next();
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::RightParen);
                }
                '$' => {
                    chars.next();
                    let var_name = self.read_identifier(&mut chars)?;
                    tokens.push(Token::Variable(var_name));
                }
                _ if ch.is_ascii_digit() => {
                    let num = self.read_number(&mut chars, false)?;
                    tokens.push(Token::Number(num));
                }
                _ if ch.is_ascii_alphabetic() || ch == '_' => {
                    let var_name = self.read_identifier(&mut chars)?;
                    tokens.push(Token::Variable(var_name));
                }
                _ => {
                    return Err(ConfigError::expression(
                        input,
                        format!("unexpected character: {}", ch),
                    ));
                }
            }
        }

        Ok(tokens)
    }

    fn read_number(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        negative: bool,
    ) -> ParseResult<i64> {
        let mut num_str = String::new();
        if negative {
            num_str.push('-');
        }

        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        num_str
            .parse::<i64>()
            .map_err(|_| ConfigError::expression(&num_str, "invalid number"))
    }

    fn read_identifier(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> ParseResult<String> {
        let mut ident = String::new();

        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        if ident.is_empty() {
            return Err(ConfigError::expression("", "expected identifier"));
        }

        Ok(ident)
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(i64),
    Variable(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let eval = ExpressionEvaluator::new();
        assert_eq!(eval.evaluate("1 + 2").unwrap(), 3);
        assert_eq!(eval.evaluate("10 - 3").unwrap(), 7);
        assert_eq!(eval.evaluate("4 * 5").unwrap(), 20);
        assert_eq!(eval.evaluate("20 / 4").unwrap(), 5);
    }

    #[test]
    fn test_precedence() {
        let eval = ExpressionEvaluator::new();
        assert_eq!(eval.evaluate("2 + 3 * 4").unwrap(), 14);
        assert_eq!(eval.evaluate("10 - 2 * 3").unwrap(), 4);
    }

    #[test]
    fn test_parentheses() {
        let eval = ExpressionEvaluator::new();
        assert_eq!(eval.evaluate("(2 + 3) * 4").unwrap(), 20);
        assert_eq!(eval.evaluate("10 / (2 + 3)").unwrap(), 2);
    }

    #[test]
    fn test_variables() {
        let mut eval = ExpressionEvaluator::new();
        eval.set_variable("x".to_string(), 10);
        eval.set_variable("y".to_string(), 5);

        assert_eq!(eval.evaluate("x + y").unwrap(), 15);
        assert_eq!(eval.evaluate("x * y").unwrap(), 50);
    }

    #[test]
    fn test_complex_expression() {
        let mut eval = ExpressionEvaluator::new();
        eval.set_variable("a".to_string(), 3);
        eval.set_variable("b".to_string(), 4);

        assert_eq!(eval.evaluate("(a + b) * 2 - 3").unwrap(), 11);
    }
}
