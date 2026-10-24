//! An honest recursive-descent parser. The lying happens later.
//!
//! Grammar:
//!   expr    = term (("+" | "-") term)*
//!   term    = unary (("*" | "/" | "%") unary)*
//!   unary   = ("-" | "+") unary | power
//!   power   = postfix ("^" unary)?
//!   postfix = primary "!"*
//!   primary = number | ident | ident "(" expr ")" | "(" expr ")"

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcError(pub String);

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn err<T>(msg: impl Into<String>) -> Result<T, CalcError> {
    Err(CalcError(msg.into()))
}

fn tokenize(input: &str) -> Result<Vec<Token>, CalcError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                // Scientific notation, e.g. 1e3 or 2.5E-4
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    let mut j = i + 1;
                    if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                        j += 1;
                    }
                    if j < chars.len() && chars[j].is_ascii_digit() {
                        while j < chars.len() && chars[j].is_ascii_digit() {
                            j += 1;
                        }
                        i = j;
                    }
                }
                let text: String = chars[start..i].iter().collect();
                let value = text
                    .parse::<f64>()
                    .map_err(|_| CalcError(format!("'{text}' is not a number")))?;
                tokens.push(Token::Num(value));
            }
            'a'..='z' | 'A'..='Z' | '_' | 'π' => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == 'π')
                {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(name.to_lowercase()));
            }
            '+' | '-' | '*' | '/' | '%' | '^' | '!' => {
                tokens.push(Token::Op(c));
                i += 1;
            }
            '×' => {
                tokens.push(Token::Op('*'));
                i += 1;
            }
            '÷' => {
                tokens.push(Token::Op('/'));
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            other => return err(format!("unexpected character '{other}'")),
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    ans: f64,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self) -> Result<f64, CalcError> {
        let mut value = self.term()?;
        while let Some(Token::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.pos += 1;
            let rhs = self.term()?;
            value = if op == '+' { value + rhs } else { value - rhs };
        }
        Ok(value)
    }

    fn term(&mut self) -> Result<f64, CalcError> {
        let mut value = self.unary()?;
        loop {
            match self.peek().cloned() {
                Some(Token::Op(op @ ('*' | '/' | '%'))) => {
                    self.pos += 1;
                    let rhs = self.unary()?;
                    value = match op {
                        '*' => value * rhs,
                        '/' => {
                            if rhs == 0.0 {
                                return err("division by zero (even I won't lie about that)");
                            }
                            value / rhs
                        }
                        _ => {
                            if rhs == 0.0 {
                                return err("modulo by zero");
                            }
                            value % rhs
                        }
                    };
                }
                // Implicit multiplication: 2(3), 2pi, (1+1)(2+2)
                Some(Token::LParen) | Some(Token::Ident(_)) | Some(Token::Num(_)) => {
                    let rhs = self.unary()?;
                    value *= rhs;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn unary(&mut self) -> Result<f64, CalcError> {
        match self.peek() {
            Some(Token::Op('-')) => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            Some(Token::Op('+')) => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<f64, CalcError> {
        let base = self.postfix()?;
        if let Some(Token::Op('^')) = self.peek() {
            self.pos += 1;
            // Right-associative: 2^3^2 = 2^(3^2)
            let exponent = self.unary()?;
            return Ok(base.powf(exponent));
        }
        Ok(base)
    }

    fn postfix(&mut self) -> Result<f64, CalcError> {
        let mut value = self.primary()?;
        while let Some(Token::Op('!')) = self.peek() {
            self.pos += 1;
            value = factorial(value)?;
        }
        Ok(value)
    }

    fn primary(&mut self) -> Result<f64, CalcError> {
        match self.next() {
            Some(Token::Num(n)) => Ok(n),
            Some(Token::LParen) => {
                let value = self.expr()?;
                match self.next() {
                    Some(Token::RParen) => Ok(value),
                    _ => err("missing closing ')'"),
                }
            }
            Some(Token::Ident(name)) => {
                if let Some(Token::LParen) = self.peek() {
                    self.pos += 1;
                    let arg = self.expr()?;
                    match self.next() {
                        Some(Token::RParen) => {}
                        _ => return err(format!("missing ')' after {name}(...")),
                    }
                    apply_function(&name, arg)
                } else {
                    constant(&name, self.ans)
                }
            }
            Some(Token::Op(op)) => err(format!("unexpected operator '{op}'")),
            Some(Token::RParen) => err("unexpected ')'"),
            None => err("expression ended too soon"),
        }
    }
}

fn factorial(n: f64) -> Result<f64, CalcError> {
    if n < 0.0 || n.fract() != 0.0 {
        return err("factorial needs a whole, non-negative number");
    }
    if n > 170.0 {
        return err("factorial too large");
    }
    Ok((1..=n as u64).map(|k| k as f64).product())
}

fn constant(name: &str, ans: f64) -> Result<f64, CalcError> {
    match name {
        "pi" | "π" => Ok(std::f64::consts::PI),
        "e" => Ok(std::f64::consts::E),
        "tau" => Ok(std::f64::consts::TAU),
        "phi" => Ok(1.618_033_988_749_895),
        "ans" => Ok(ans),
        _ => err(format!("unknown name '{name}'")),
    }
}

fn apply_function(name: &str, x: f64) -> Result<f64, CalcError> {
    let value = match name {
        "sqrt" => {
            if x < 0.0 {
                return err("sqrt of a negative number");
            }
            x.sqrt()
        }
        "cbrt" => x.cbrt(),
        "abs" => x.abs(),
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "asin" => x.asin(),
        "acos" => x.acos(),
        "atan" => x.atan(),
        "ln" => {
            if x <= 0.0 {
                return err("ln needs a positive number");
            }
            x.ln()
        }
        "log" => {
            if x <= 0.0 {
                return err("log needs a positive number");
            }
            x.log10()
        }
        "exp" => x.exp(),
        "floor" => x.floor(),
        "ceil" => x.ceil(),
        "round" => x.round(),
        _ => return err(format!("unknown function '{name}'")),
    };
    Ok(value)
}

/// Evaluate an expression honestly. `ans` is the previous result.
pub fn evaluate(input: &str, ans: f64) -> Result<f64, CalcError> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return err("nothing to calculate");
    }
    let mut parser = Parser {
        tokens,
        pos: 0,
        ans,
    };
    let value = parser.expr()?;
    if parser.pos < parser.tokens.len() {
        return err("unexpected input after the expression");
    }
    if !value.is_finite() {
        return err("the result is not a finite number");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(s: &str) -> f64 {
        evaluate(s, 0.0).unwrap()
    }

    #[test]
    fn arithmetic_is_honest() {
        assert_eq!(eval("2+2"), 4.0);
        assert_eq!(eval("2+3*4"), 14.0);
        assert_eq!(eval("(2+3)*4"), 20.0);
        assert_eq!(eval("10/4"), 2.5);
        assert_eq!(eval("10 % 3"), 1.0);
        assert_eq!(eval("-3^2"), -9.0);
        assert_eq!(eval("2^3^2"), 512.0);
        assert_eq!(eval("5!"), 120.0);
        assert_eq!(eval("2(3+1)"), 8.0);
        assert_eq!(eval("1e3 + 1"), 1001.0);
        assert_eq!(eval("6 × 7"), 42.0);
    }

    #[test]
    fn functions_and_constants() {
        assert_eq!(eval("sqrt(16)"), 4.0);
        assert!((eval("2pi") - std::f64::consts::TAU).abs() < 1e-12);
        assert!((eval("sin(pi/2)") - 1.0).abs() < 1e-12);
        assert_eq!(eval("log(1000)"), 3.0);
        assert_eq!(evaluate("ans*2", 21.0).unwrap(), 42.0);
    }

    #[test]
    fn errors() {
        assert!(evaluate("1/0", 0.0).is_err());
        assert!(evaluate("(1+2", 0.0).is_err());
        assert!(evaluate("2 +", 0.0).is_err());
        assert!(evaluate("foo(2)", 0.0).is_err());
        assert!(evaluate("", 0.0).is_err());
    }
}
