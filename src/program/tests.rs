use super::*;

#[test]
fn chapter1() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
            "print".into(),
            "hello world".into(),
            "hello again...".into()
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
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter2_1() {
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
        &["print".into(), 123456.into(), 123456.0.into(),]
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
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter2_2() {
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
            ByteCode::LoadConstant(0, 0),
            ByteCode::Move(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Move(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::GetGlobal(2, 1),
            ByteCode::GetGlobal(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Move(3, 2),
            ByteCode::LoadConstant(4, 2),
            ByteCode::Call(3, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter2_3() {
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
            ByteCode::LoadInt(0, 456),
            ByteCode::LoadInt(1, 123),
            ByteCode::Move(0, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            ByteCode::Move(1, 0),
            ByteCode::Move(0, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            ByteCode::GetGlobal(1, 1),
            ByteCode::Move(0, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            ByteCode::LoadInt(1, 123),
            ByteCode::SetGlobal(1, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
            ByteCode::Move(1, 0),
            ByteCode::SetGlobal(1, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
            ByteCode::GetGlobal(1, 2),
            ByteCode::SetGlobal(1, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter3_4() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
print "tab:\thi" -- tab
print "\xE4\xBD\xA0\xE5\xA5\xBD" -- 你好
print "\xE4\xBD" -- invalid UTF-8
print "\72\101\108\108\111" -- Hello
print "null: \0." -- '\0'
"#,
    )
    .unwrap();
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter4_2() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local key = "kkk"
print {
    100, 200, 300; -- list style
    x="hello", y="world"; -- record style
    [key]="vvv"; -- general style
}
"#,
    )
    .unwrap();
    crate::Lua::execute(&program).unwrap();
}
