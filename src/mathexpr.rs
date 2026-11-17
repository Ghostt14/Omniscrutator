// A tiny arithmetic evaluator, written by hand instead of pulling in a crate
// (keeps the dependency tree and binary size down, which matters for the
// "as little RAM/disk as possible" goal). Supports + - * / ^, parentheses,
// unary minus, and -- the thing a real calculator-adjacent launcher needs --
// implicit multiplication: "4(5)" means 4*5, "(2)(3)" means 2*3, "2 3" is
// NOT implicit multiplication (that's just two separate tokens with no
// adjacency rule across whitespace-separated numbers, to avoid treating an
// ordinary two-word search as math).

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Num(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Option<Vec<Tok>> {
    let mut toks = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' => { i += 1; }
            '+' => { toks.push(Tok::Plus); i += 1; }
            '-' => { toks.push(Tok::Minus); i += 1; }
            '*' => { toks.push(Tok::Star); i += 1; }
            '/' => { toks.push(Tok::Slash); i += 1; }
            '^' => { toks.push(Tok::Caret); i += 1; }
            '(' => { toks.push(Tok::LParen); i += 1; }
            ')' => { toks.push(Tok::RParen); i += 1; }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                let n: f64 = s.parse().ok()?;
                toks.push(Tok::Num(n));
            }
            _ => return None, // any other character -- this isn't a math expression
        }
    }
    // Insert implicit multiplication: Num/RParen directly followed by Num/LParen.
    let mut out = Vec::with_capacity(toks.len());
    for (idx, t) in toks.iter().enumerate() {
        if idx > 0 {
            let prev = toks[idx - 1];
            let implicit = matches!(prev, Tok::Num(_) | Tok::RParen)
                && matches!(t, Tok::Num(_) | Tok::LParen);
            if implicit {
                out.push(Tok::Star);
            }
        }
        out.push(*t);
    }
    Some(out)
}

struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Tok> {
        self.toks.get(self.pos).copied()
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.peek();
        self.pos += 1;
        t
    }

    // expr := term (('+'|'-') term)*
    fn expr(&mut self) -> Option<f64> {
        let mut v = self.term()?;
        loop {
            match self.peek() {
                Some(Tok::Plus) => { self.next(); v += self.term()?; }
                Some(Tok::Minus) => { self.next(); v -= self.term()?; }
                _ => break,
            }
        }
        Some(v)
    }

    // term := power (('*'|'/') power)*
    fn term(&mut self) -> Option<f64> {
        let mut v = self.power()?;
        loop {
            match self.peek() {
                Some(Tok::Star) => { self.next(); v *= self.power()?; }
                Some(Tok::Slash) => {
                    self.next();
                    let rhs = self.power()?;
                    if rhs == 0.0 { return None; }
                    v /= rhs;
                }
                _ => break,
            }
        }
        Some(v)
    }

    // power := unary ('^' power)?   -- right associative
    fn power(&mut self) -> Option<f64> {
        let base = self.unary()?;
        if let Some(Tok::Caret) = self.peek() {
            self.next();
            let exp = self.power()?;
            Some(base.powf(exp))
        } else {
            Some(base)
        }
    }

    // unary := '-' unary | primary
    fn unary(&mut self) -> Option<f64> {
        if let Some(Tok::Minus) = self.peek() {
            self.next();
            Some(-self.unary()?)
        } else {
            self.primary()
        }
    }

    // primary := NUMBER | '(' expr ')'
    fn primary(&mut self) -> Option<f64> {
        match self.next()? {
            Tok::Num(n) => Some(n),
            Tok::LParen => {
                let v = self.expr()?;
                match self.next() {
                    Some(Tok::RParen) => Some(v),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Try to evaluate `input` as an arithmetic expression. Returns `None` if it
/// isn't one (contains letters, mismatched parens, etc.) -- callers should
/// fall back to normal search in that case, not show an error.
pub fn try_eval(input: &str) -> Option<f64> {
    let trimmed = input.trim();
    if trimmed.is_empty() || !trimmed.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let toks = tokenize(trimmed)?;
    if toks.is_empty() {
        return None;
    }
    let mut p = Parser { toks: &toks, pos: 0 };
    let v = p.expr()?;
    if p.pos != toks.len() {
        return None; // leftover tokens -- malformed expression
    }
    if v.is_finite() { Some(v) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_multiplication() {
        assert_eq!(try_eval("4(5)-3"), Some(17.0));
    }

    #[test]
    fn basic_ops() {
        assert_eq!(try_eval("2+3*4"), Some(14.0));
        assert_eq!(try_eval("(2+3)*4"), Some(20.0));
        assert_eq!(try_eval("2^10"), Some(1024.0));
        assert_eq!(try_eval("-3(2)"), Some(-6.0));
    }

    #[test]
    fn not_math() {
        assert_eq!(try_eval("resume.docx"), None);
        assert_eq!(try_eval("notepad"), None);
        assert_eq!(try_eval(""), None);
    }
}
