use crate::{byte_code::ByteCode, Program};

#[test]
fn types() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
        &["print".into(), 123456i64.into(), 123456.0.into(),]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print(nil)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadNil(1),
            ByteCode::Call(0, 1),
            // print(false)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadFalse(1),
            ByteCode::Call(0, 1),
            // print(123)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadInt(1, 123),
            ByteCode::Call(0, 1),
            // print(123456)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            // print(123456.0)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn local() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a = "hello, local!" -- define a local by string
local b = a -- define a local by another local
print(b) -- print local variable
print(print) -- print global variable
local print = print --define a local by global variable with same name
print "I'm local-print!" -- call local function
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "hello, local!".into(),
            "print".into(),
            "I'm local-print!".into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = "hello, local!"
            ByteCode::LoadConstant(0, 0),
            // local b = a
            ByteCode::Move(1, 0),
            // print(b)
            ByteCode::GetGlobal(2, 1),
            ByteCode::Move(3, 1),
            ByteCode::Call(2, 1),
            // print(print)
            ByteCode::GetGlobal(2, 1),
            ByteCode::GetGlobal(3, 1),
            ByteCode::Call(2, 1),
            // local print = print
            ByteCode::GetGlobal(2, 1),
            // print "I'm local-print!"
            ByteCode::Move(3, 2),
            ByteCode::LoadConstant(4, 2),
            ByteCode::Call(3, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn assign() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a = 456
a = 123
print(a)
a = a
print(a)
a = g
print(a)
g = 123
print(g)
g = a
print(g)
g = g2
print(g)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &["print".into(), "g".into(), "g2".into(),]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = 456
            ByteCode::LoadInt(0, 456),
            // a = 123
            ByteCode::LoadInt(0, 123),
            // print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // a = a
            // print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // a = g
            ByteCode::GetGlobal(0, 1),
            // print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // g = 123
            ByteCode::SetGlobalInteger(1, 123),
            // print(g)
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
            // g = a
            ByteCode::SetGlobal(1, 0),
            // print(g)
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
            // g = g2
            ByteCode::SetGlobalGlobal(1, 2),
            // print(g)
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
