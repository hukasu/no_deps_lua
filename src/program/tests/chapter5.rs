use crate::{Error, Program, bytecode::Bytecode, program::Local};

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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local i = 100
            Bytecode::load_integer(0.into(), 100i8.into()),
            // local f = 3.14
            Bytecode::load_constant(1.into(), 0u8.into()),
            // a = "iamastring"
            Bytecode::set_uptable(0.into(), 1.into(), 2.into(), true.into()),
            // print(~100)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::load_integer(3.into(), (-101i8).into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(~i)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::bit_not(3.into(), 0.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(-3.14)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::load_constant(3.into(), 4u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(-f)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::neg(3.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(#"iamastring")
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::load_integer(3.into(), 10i8.into()), // Optimized taking the len of a constant string
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(#a)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 1.into()),
            Bytecode::len(3.into(), 3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(not false)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(not nil)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(not not nil)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::load_false(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(not print)
            Bytecode::get_uptable(2.into(), 0.into(), 3.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::not(3.into(), 3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(2.into(), 1.into(), 1.into()),
        ],
        &[
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "a".into(),
            "iamastring".into(),
            "print".into(),
            #[allow(clippy::approx_constant)]
            (-3.14f64).into(),
        ],
        &[Local::new("i".into(), 3, 38), Local::new("f".into(), 4, 38)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // g = 10
            Bytecode::set_uptable(0.into(), 0.into(), 1.into(), true.into()),
            // local a,b,c = 1.1, 2.0, 100
            Bytecode::load_constant(0.into(), 2u8.into()),
            Bytecode::load_float(1.into(), 2i8.into()),
            Bytecode::load_integer(2.into(), 100i8.into()),
            // print(100+g) -- commutative, AddInt
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::add_integer(4.into(), 4.into(), 100.into()),
            // TODO MMBINI
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(a-1)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::add_integer(4.into(), 0.into(), (-1i8).into()),
            // TODO MMBINI
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(100/c) -- result is float
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::load_integer(4.into(), 100i8.into()),
            Bytecode::div(4.into(), 4.into(), 2.into()),
            // TODO MMBINI
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(100>>b) -- 2.0 will be convert to int 2
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::load_integer(4.into(), 100i8.into()),
            Bytecode::shift_right(4.into(), 4.into(), 1.into()),
            // TODO MMBINI
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(100>>a) -- panic
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::load_integer(4.into(), 100i8.into()),
            Bytecode::shift_right(4.into(), 4.into(), 0.into()),
            // TODO MMBINI
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(3.into(), 1.into(), 1.into()),
        ],
        &["g".into(), 10i64.into(), 1.1f64.into(), "print".into()],
        &[
            Local::new("a".into(), 6, 26),
            Local::new("b".into(), 6, 26),
            Local::new("c".into(), 6, 26),
        ],
        &["_ENV".into()],
        0,
    );

    match crate::Lua::run_program(program) {
        Err(err @ Error::BitwiseOperand(_, _, _)) => log::error!("{}", err),
        Err(err) => panic!("Expected `BitwiseOperand` error, but got {:?}.", err),
        Ok(_) => panic!("Last print should fail"),
    }
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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print('hello, '..'world')
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::load_constant(2.into(), 2u8.into()),
            Bytecode::concat(1.into(), 2.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print('hello, ' .. 123)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::load_integer(2.into(), 123i8.into()),
            Bytecode::concat(1.into(), 2.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print(3.14 .. 15926)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::load_integer(2.into(), 15926i16.into()),
            Bytecode::concat(1.into(), 2.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print('hello' .. true) -- panic
            Bytecode::load_true(0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 4u8.into()),
            Bytecode::move_bytecode(3.into(), 0.into()),
            Bytecode::concat(2.into(), 2.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "hello, ".into(),
            "world".into(),
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "hello".into(),
        ],
        &[Local::new("a".into(), 18, 24)],
        &["_ENV".into()],
        0,
    );

    match crate::Lua::run_program(program) {
        Err(err @ Error::ConcatOperand(_)) => log::error!("{}", err),
        Err(err) => panic!("Expected `ConcatOperand` error, but got {:?}.", err),
        Ok(_) => panic!("Last print should fail"),
    }
}

#[test]
fn multi_concat() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
print(('hello, '..'world').." "..("3." .. 14 .. 15))
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print(('hello, '..'world').." "..("3." .. 14 .. 15))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::load_constant(2.into(), 2u8.into()),
            Bytecode::concat(1.into(), 2.into()),
            Bytecode::load_constant(2.into(), 3u8.into()),
            Bytecode::load_constant(3.into(), 4u8.into()),
            Bytecode::load_integer(4.into(), 14i8.into()),
            Bytecode::load_integer(5.into(), 15i8.into()),
            Bytecode::concat(1.into(), 5.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "hello, ".into(),
            "world".into(),
            " ".into(),
            "3.".into(),
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}
