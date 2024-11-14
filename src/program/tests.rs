use super::*;

#[test]
fn chapter1() {
    simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default()).unwrap();
    let program = Program::parse(
        r#"
print "hello world"
print "hello again..."
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            Value::String("print"),
            Value::String("hello world"),
            Value::String("hello again...")
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
        ]
    );

    let err = Program::parse(
        r#"
print "hello world"
print "hello again...
"#,
    )
    .expect_err("This program should fail");
    assert_eq!(err, Error::Parse);
}

#[test]
fn chapter2_1() {
    let program = Program::parse(
        r#"
print(nil)
print(false)
print(123)
print(123456)
print(123456.0)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            Value::String("print"),
            Value::Integer(123456),
            Value::Float(123456.0)
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadNil(1),
            ByteCode::Call(0, 1),
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadBool(1, false),
            ByteCode::Call(0, 1),
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadInt(1, 123),
            ByteCode::Call(0, 1),
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
        ]
    );
}
