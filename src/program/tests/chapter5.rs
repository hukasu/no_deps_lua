use crate::{bytecode::Bytecode, Error, Program};

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
            Bytecode::variadic_arguments_prepare(0),
            // local i = 100
            Bytecode::load_integer(0, 100),
            // local f = 3.14
            Bytecode::load_constant(1, 0),
            // a = "iamastring"
            Bytecode::set_uptable(0, 1, 2, 1),
            // print(~100)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::load_integer(3, -101),
            Bytecode::call(2, 2, 1),
            // print(~i)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::bit_not(3, 0),
            Bytecode::call(2, 2, 1),
            // print(-3.14)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::load_constant(3, 4),
            Bytecode::call(2, 2, 1),
            // print(-f)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::neg(3, 1),
            Bytecode::call(2, 2, 1),
            // print(#"iamastring")
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::load_integer(3, 10), // Optimized taking the len of a constant string
            Bytecode::call(2, 2, 1),
            // print(#a)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::get_uptable(3, 0, 1),
            Bytecode::len(3, 3),
            Bytecode::call(2, 2, 1),
            // print(not false)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // print(not nil)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // print(not not nil)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::load_false(3),
            Bytecode::call(2, 2, 1),
            // print(not print)
            Bytecode::get_uptable(2, 0, 3),
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::not(3, 3),
            Bytecode::call(2, 2, 1),
            // EOF
            Bytecode::return_bytecode(2, 1, 1),
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
            Bytecode::variadic_arguments_prepare(0),
            // g = 10
            Bytecode::set_uptable(0, 0, 1, 1),
            // local a,b,c = 1.1, 2.0, 100
            Bytecode::load_constant(0, 2),
            Bytecode::load_float(1, 2),
            Bytecode::load_integer(2, 100),
            // print(100+g) -- commutative, AddInt
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::add_integer(4, 4, 100),
            Bytecode::call(3, 2, 1),
            // print(a-1)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::add_integer(4, 0, -1),
            Bytecode::call(3, 2, 1),
            // print(100/c) -- result is float
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::load_integer(4, 100),
            Bytecode::div(4, 4, 2),
            Bytecode::call(3, 2, 1),
            // print(100>>b) -- 2.0 will be convert to int 2
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::load_integer(4, 100),
            Bytecode::shift_right(4, 4, 1),
            Bytecode::call(3, 2, 1),
            // print(100>>a) -- panic
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::load_integer(4, 100),
            Bytecode::shift_right(4, 4, 0),
            Bytecode::call(3, 2, 1),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
        ],
        &["g".into(), 10i64.into(), 1.1f64.into(), "print".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // print('hello, '..'world')
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::load_constant(2, 2),
            Bytecode::concat(1, 2),
            Bytecode::call(0, 2, 1),
            // print('hello, ' .. 123)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::load_integer(2, 123),
            Bytecode::concat(1, 2),
            Bytecode::call(0, 2, 1),
            // print(3.14 .. 15926)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 3),
            Bytecode::load_integer(2, 15926),
            Bytecode::concat(1, 2),
            Bytecode::call(0, 2, 1),
            // print('hello' .. true) -- panic
            Bytecode::load_true(0),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::load_constant(2, 4),
            Bytecode::move_bytecode(3, 0),
            Bytecode::concat(2, 2),
            Bytecode::call(1, 2, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &[
            "print".into(),
            "hello, ".into(),
            "world".into(),
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "hello".into(),
        ],
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
            Bytecode::variadic_arguments_prepare(0),
            // print(('hello, '..'world').." "..("3." .. 14 .. 15))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::load_constant(2, 2),
            Bytecode::concat(1, 2),
            Bytecode::load_constant(2, 3),
            Bytecode::load_constant(3, 4),
            Bytecode::load_integer(4, 14),
            Bytecode::load_integer(5, 15),
            Bytecode::concat(1, 5),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &[
            "print".into(),
            "hello, ".into(),
            "world".into(),
            " ".into(),
            "3.".into(),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}
