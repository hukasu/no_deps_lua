use super::*;

#[test]
fn empty_input() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let mut lex = Lex::new("");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 0,
            start: 0,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.program.len(), 0);

    let mut lex = Lex::new("        \n\n\n\n\t\t\t\t\r\r\r\r");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 4,
            column: 8,
            start: 20,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);
}

#[test]
fn keywords() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let mut lex = Lex::new(
        r#"
and       break     do        else      elseif    end
false     for       function  goto      if        in
local     nil       not       or        repeat    return
then      true      until     while     keyword
"#,
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 3,
            start: 1,
            lexeme_type: LexemeType::And
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 15,
            start: 11,
            lexeme_type: LexemeType::Break
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 22,
            start: 21,
            lexeme_type: LexemeType::Do
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 34,
            start: 31,
            lexeme_type: LexemeType::Else
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 46,
            start: 41,
            lexeme_type: LexemeType::Elseif
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 53,
            start: 51,
            lexeme_type: LexemeType::End
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 2,
            column: 5,
            start: 55,
            lexeme_type: LexemeType::False
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 2,
            column: 13,
            start: 65,
            lexeme_type: LexemeType::For
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 2,
            column: 28,
            start: 75,
            lexeme_type: LexemeType::Function
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 2,
            column: 34,
            start: 85,
            lexeme_type: LexemeType::Goto
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 2,
            column: 42,
            start: 95,
            lexeme_type: LexemeType::If
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 2,
            column: 52,
            start: 105,
            lexeme_type: LexemeType::In
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 3,
            column: 5,
            start: 108,
            lexeme_type: LexemeType::Local
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 3,
            column: 13,
            start: 118,
            lexeme_type: LexemeType::Nil
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 3,
            column: 23,
            start: 128,
            lexeme_type: LexemeType::Not
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 3,
            column: 32,
            start: 138,
            lexeme_type: LexemeType::Or
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 3,
            column: 46,
            start: 148,
            lexeme_type: LexemeType::Repeat
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 3,
            column: 56,
            start: 158,
            lexeme_type: LexemeType::Return
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 4,
            column: 4,
            start: 165,
            lexeme_type: LexemeType::Then
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 4,
            column: 14,
            start: 175,
            lexeme_type: LexemeType::True
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 4,
            column: 25,
            start: 185,
            lexeme_type: LexemeType::Until
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 4,
            column: 35,
            start: 195,
            lexeme_type: LexemeType::While
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 4,
            column: 47,
            start: 205,
            lexeme_type: LexemeType::Name("keyword")
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 5,
            column: 0,
            start: 213,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);
}

#[test]
fn short_comment() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let mut lex = Lex::new("-- abc");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 6,
            start: 6,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);

    let mut lex = Lex::new("-- Lorem ipsum dolor sit amet, consectetur adipiscing elit.");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 59,
            start: 59,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);

    let mut lex = Lex::new("--\x01");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 3,
            start: 3,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);

    let mut lex = Lex::new("-- Lorem ipsum dolor sit amet,\x01consectetur adipiscing elit.");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 59,
            start: 59,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);
}

#[test]
fn hello_world() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let mut lex = Lex::new(r#"print "hello world""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 5,
            start: 0,
            lexeme_type: LexemeType::Name("print")
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 19,
            start: 6,
            lexeme_type: LexemeType::String("hello world")
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 19,
            start: 19,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);

    let mut lex = Lex::new(
        r#"print "hello world"
print "hello again...""#,
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 5,
            start: 0,
            lexeme_type: LexemeType::Name("print")
        }))
    );
    assert_eq!(lex.line, 0);
    assert_eq!(lex.column, 5);
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 19,
            start: 6,
            lexeme_type: LexemeType::String("hello world")
        }))
    );
    assert_eq!(lex.line, 0);
    assert_eq!(lex.column, 19);
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 5,
            start: 20,
            lexeme_type: LexemeType::Name("print")
        }))
    );
    assert_eq!(lex.line, 1);
    assert_eq!(lex.column, 5);
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 22,
            start: 26,
            lexeme_type: LexemeType::String("hello again...")
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 1,
            column: 22,
            start: 42,
            lexeme_type: LexemeType::Eof
        }))
    );
    assert_eq!(lex.line, 1);
    assert_eq!(lex.column, 22);
    assert!(lex.next().is_none());
    assert_eq!(lex.remaining(), 0);

    let mut lex = Lex::new("print \"hello world");
    assert_eq!(
        lex.next(),
        Some(Ok(Lexeme {
            line: 0,
            column: 5,
            start: 0,
            lexeme_type: LexemeType::Name("print")
        }))
    );
    assert_eq!(
        lex.next(),
        Some(Err(Error {
            kind: ErrorKind::EofAtString,
            line: 0,
            column: 18
        }))
    );
}
