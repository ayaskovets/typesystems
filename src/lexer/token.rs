#[derive(Debug)] // TODO
pub enum Token {
    // single-char tokens
    Comma,
    Dot,
    Lparen,
    Rparen,
    Lbracket,
    Rbracket,
    Equals,
    Spacing,
    Newline,
    // operators
    Arrow,
    // keywords
    Fn,
    Forall,
    In,
    Let,
    // variable-size tokens
    Ident(String),
    Integer(i32),
    Floating(f32),
    Comment(String),
}
