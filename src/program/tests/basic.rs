use crate::{Error, bytecode::Bytecode, program::Local};

#[test]
fn print_and_warn() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = crate::Program::parse(
        r#"
print "hello, world!"
warn "hello, warn!"
warn "@on"
warn "hello, warn!"
warn "@off"
warn "hello, warn!"
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // print "hello, world!"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1u8),
            Bytecode::call(0, 2, 1),
            // warn "hello, warn!"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3u8),
            Bytecode::call(0, 2, 1),
            // warn "@on"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 4u8),
            Bytecode::call(0, 2, 1),
            // warn "hello, warn!"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3u8),
            Bytecode::call(0, 2, 1),
            // warn "@off"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 5u8),
            Bytecode::call(0, 2, 1),
            // warn "hello, warn!"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3u8),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &[
            "print".into(),
            "hello, world!".into(),
            "warn".into(),
            "hello, warn!".into(),
            "@on".into(),
            "@off".into(),
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}

#[test]
fn assert() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = crate::Program::parse(
        r#"
local a, b = 10, 100

assert(b > a, "b was smaller than a")
local c,d,e = assert(b == a * 10, "b was different from 10 times a", "something else")
assert(c)
assert(d == "b was different from 10 times a")
assert(e == "something else")
assert(a > b, "a was smaller than b")
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local a, b = 10, 100
            Bytecode::load_integer(0, 10i8),
            Bytecode::load_integer(1, 100i8),
            // assert(b > a, "b was smaller than a")
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::less_than(0, 1, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::load_constant(4, 1u8),
            Bytecode::call(2, 3, 1),
            // local c,d,e = assert(b == a * 10, "b was different from 10 times a", "something else")
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::mul_constant(3, 0, 2),
            // TODO MMBINK
            Bytecode::equal(1, 3, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::load_constant(4, 3u8),
            Bytecode::load_constant(5, 4u8),
            Bytecode::call(2, 4, 4),
            // assert(c)
            Bytecode::get_uptable(5, 0, 0),
            Bytecode::move_bytecode(6, 2),
            Bytecode::call(5, 2, 1),
            // assert(d == "b was different from 10 times a")
            Bytecode::get_uptable(5, 0, 0),
            Bytecode::equal_constant(3, 3, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(6),
            Bytecode::load_true(6),
            Bytecode::call(5, 2, 1),
            // assert(e == "something else")
            Bytecode::get_uptable(5, 0, 0),
            Bytecode::equal_constant(4, 4, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(6),
            Bytecode::load_true(6),
            Bytecode::call(5, 2, 1),
            // assert(a > b, "a was smaller than b")
            Bytecode::get_uptable(5, 0, 0),
            Bytecode::less_than(1, 0, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(6),
            Bytecode::load_true(6),
            Bytecode::load_constant(7, 5u8),
            Bytecode::call(5, 3, 1),
            // EOF
            Bytecode::return_bytecode(5, 1, 1),
        ],
        &[
            "assert".into(),
            "b was smaller than a".into(),
            10i64.into(),
            "b was different from 10 times a".into(),
            "something else".into(),
            "a was smaller than b".into(),
        ],
        &[
            Local::new("a".into(), 4, 43),
            Local::new("b".into(), 4, 43),
            Local::new("c".into(), 20, 43),
            Local::new("d".into(), 20, 43),
            Local::new("e".into(), 20, 43),
        ],
        &["_ENV".into()],
        0,
    );

    // TODO improve this when there is better error handling
    match crate::Lua::run_program(program) {
        Ok(_) => panic!("Should fail"),
        Err(Error::IntegerConversion) => (),
        Err(err) => panic!(
            "Should fail with IntegerConversion, but failed with {}",
            err
        ),
    }
}
