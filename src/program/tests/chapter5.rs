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
            ByteCode::VariadicArgumentPrepare(0),
            // local i = 100
            ByteCode::LoadInt(0, 100),
            // local f = 3.14
            ByteCode::LoadConstant(1, 0),
            // a = "iamastring"
            ByteCode::SetUpTableConstant(0, 1, 2),
            // print(~100)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::LoadInt(3, -101),
            ByteCode::Call(2, 2, 1),
            // print(~i)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::BitNot(3, 0),
            ByteCode::Call(2, 2, 1),
            // print(-3.14)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::LoadConstant(3, 4),
            ByteCode::Call(2, 2, 1),
            // print(-f)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::Neg(3, 1),
            ByteCode::Call(2, 2, 1),
            // print(#"iamastring")
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::LoadInt(3, 10), // Optimized taking the len of a constant string
            ByteCode::Call(2, 2, 1),
            // print(#a)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::GetUpTable(3, 0, 1),
            ByteCode::Len(3, 3),
            ByteCode::Call(2, 2, 1),
            // print(not false)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::LoadTrue(3),
            ByteCode::Call(2, 2, 1),
            // print(not nil)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::LoadTrue(3),
            ByteCode::Call(2, 2, 1),
            // print(not not nil)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::LoadFalse(3),
            ByteCode::Call(2, 2, 1),
            // print(not print)
            ByteCode::GetUpTable(2, 0, 3),
            ByteCode::GetUpTable(3, 0, 3),
            ByteCode::Not(3, 3),
            ByteCode::Call(2, 2, 1),
            // EOF
            ByteCode::Return(2, 1, 1),
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
        &["g".into(), 10i64.into(), 1.1f64.into(), "print".into()]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            ByteCode::VariadicArgumentPrepare(0),
            // g = 10
            ByteCode::SetUpTableConstant(0, 0, 1),
            // local a,b,c = 1.1, 2.0, 100
            ByteCode::LoadConstant(0, 2),
            ByteCode::LoadFloat(1, 2),
            ByteCode::LoadInt(2, 100),
            // print(100+g) -- commutative, AddInt
            ByteCode::GetUpTable(3, 0, 3),
            ByteCode::GetUpTable(4, 0, 0),
            ByteCode::AddInteger(4, 4, 100),
            ByteCode::Call(3, 2, 1),
            // print(a-1)
            ByteCode::GetUpTable(3, 0, 3),
            ByteCode::AddInteger(4, 0, -1),
            ByteCode::Call(3, 2, 1),
            // print(100/c) -- result is float
            ByteCode::GetUpTable(3, 0, 3),
            ByteCode::LoadInt(4, 100),
            ByteCode::Div(4, 4, 2),
            ByteCode::Call(3, 2, 1),
            // print(100>>b) -- 2.0 will be convert to int 2
            ByteCode::GetUpTable(3, 0, 3),
            ByteCode::LoadInt(4, 100),
            ByteCode::ShiftRight(4, 4, 1),
            ByteCode::Call(3, 2, 1),
            // print(100>>a) -- panic
            ByteCode::GetUpTable(3, 0, 3),
            ByteCode::LoadInt(4, 100),
            ByteCode::ShiftRight(4, 4, 0),
            ByteCode::Call(3, 2, 1),
            // EOF
            ByteCode::Return(3, 1, 1),
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
            ByteCode::VariadicArgumentPrepare(0),
            // print('hello, '..'world')
            ByteCode::GetUpTable(0, 0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadConstant(2, 2),
            ByteCode::Concat(1, 2),
            ByteCode::Call(0, 2, 1),
            // print('hello, ' .. 123)
            ByteCode::GetUpTable(0, 0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadInt(2, 123),
            ByteCode::Concat(1, 2),
            ByteCode::Call(0, 2, 1),
            // print(3.14 .. 15926)
            ByteCode::GetUpTable(0, 0, 0),
            ByteCode::LoadConstant(1, 3),
            ByteCode::LoadInt(2, 15926),
            ByteCode::Concat(1, 2),
            ByteCode::Call(0, 2, 1),
            // print('hello' .. true) -- panic
            ByteCode::LoadTrue(0),
            ByteCode::GetUpTable(1, 0, 0),
            ByteCode::LoadConstant(2, 4),
            ByteCode::Move(3, 0),
            ByteCode::Concat(2, 2),
            ByteCode::Call(1, 2, 1),
            // EOF
            ByteCode::Return(1, 1, 1),
        ]
    );
    crate::Lua::execute(&program)
        .inspect_err(|err| log::error!("{err}"))
        .expect_err("Last print should fail");
}
