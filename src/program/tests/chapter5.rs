use crate::{byte_code::ByteCode, Program};

#[test]
fn unops() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    let program = Program::parse(
        r#"
local i = 100
local f = 3.14
a = "iamastring"
print(~100)
print(~i)
print(-3.14)
print(-f)
print(#"iamastring")
print(#a)
print(not false)
print(not nil)
print(not not nil)
print(not print)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "a".into(),
            "iamastring".into(),
            "print".into(),
            #[allow(clippy::approx_constant)]
            (-3.14f64).into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local i = 100
            ByteCode::LoadInt(0, 100),
            // local f = 3.14
            ByteCode::LoadConstant(1, 0),
            // a = "iamastring"
            ByteCode::SetGlobalConstant(1, 2),
            // print(~100)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadInt(3, -101),
            ByteCode::Call(2, 1),
            // print(~i)
            ByteCode::GetGlobal(2, 3),
            ByteCode::BitNot(3, 0),
            ByteCode::Call(2, 1),
            // print(-3.14)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadConstant(3, 4),
            ByteCode::Call(2, 1),
            // print(-f)
            ByteCode::GetGlobal(2, 3),
            ByteCode::Neg(3, 1),
            ByteCode::Call(2, 1),
            // print(#"iamastring")
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadInt(3, 10),
            ByteCode::Call(2, 1),
            // print(#a)
            ByteCode::GetGlobal(2, 3),
            ByteCode::GetGlobal(3, 1),
            ByteCode::Len(3, 3),
            ByteCode::Call(2, 1),
            // print(not false)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadTrue(3),
            ByteCode::Call(2, 1),
            // print(not nil)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadTrue(3),
            ByteCode::Call(2, 1),
            // print(not not nil)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadFalse(3),
            ByteCode::Call(2, 1),
            // print(not print)
            ByteCode::GetGlobal(2, 3),
            ByteCode::GetGlobal(3, 3),
            ByteCode::Not(3, 3),
            ByteCode::Call(2, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn binops() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    let program = Program::parse(
        r#"
g = 10
local a,b,c = 1.1, 2.0, 100

print(100+g) -- commutative, AddInt
print(a-1)
print(100/c) -- result is float
print(100>>b) -- 2.0 will be convert to int 2
print(100>>a) -- panic
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &["g".into(), 1.1f64.into(), "print".into()]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // g = 10
            ByteCode::SetGlobalInteger(0, 10),
            // local a,b,c = 1.1, 2.0, 100
            ByteCode::LoadConstant(0, 1),
            ByteCode::LoadFloat(1, 2),
            ByteCode::LoadInt(2, 100),
            // print(100+g) -- commutative, AddInt
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::GetGlobal(5, 0),
            ByteCode::Add(4, 4, 5),
            ByteCode::Call(3, 1),
            // print(a-1)
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 1),
            ByteCode::Sub(4, 0, 4),
            ByteCode::Call(3, 1),
            // print(100/c) -- result is float
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::Div(4, 4, 2),
            ByteCode::Call(3, 1),
            // print(100>>b) -- 2.0 will be convert to int 2
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::ShiftRight(4, 4, 1),
            ByteCode::Call(3, 1),
            // print(100>>a) -- panic
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::ShiftRight(4, 4, 0),
            ByteCode::Call(3, 1),
        ]
    );
    crate::Lua::execute(&program)
        .inspect_err(|err| log::error!("{err}"))
        .expect_err("Last print should fail");
}

#[test]
fn concat() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    let program = Program::parse(
        r#"
print('hello, '..'world')
print('hello, ' .. 123)
print(3.14 .. 15926)
local a = true
print('hello' .. a) -- panic
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "hello, ".into(),
            "world".into(),
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "hello".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print('hello, '..'world')
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadConstant(2, 2),
            ByteCode::Concat(1, 1, 2),
            ByteCode::Call(0, 1),
            // print('hello, ' .. 123)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadInt(2, 123),
            ByteCode::Concat(1, 1, 2),
            ByteCode::Call(0, 1),
            // print(3.14 .. 15926)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 3),
            ByteCode::LoadInt(2, 15926),
            ByteCode::Concat(1, 1, 2),
            ByteCode::Call(0, 1),
            // print('hello' .. true) -- panic
            ByteCode::LoadTrue(0),
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 4),
            ByteCode::Concat(2, 2, 0),
            ByteCode::Call(1, 1),
        ]
    );
    crate::Lua::execute(&program)
        .inspect_err(|err| log::error!("{err}"))
        .expect_err("Last print should fail");
}
