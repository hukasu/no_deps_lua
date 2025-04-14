use crate::{Program, bytecode::Bytecode, program::Local};

#[test]
fn types() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print(nil)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_nil(1.into(), 0.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print(false)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_false(1.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print(123)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_integer(1.into(), 123i8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print(123456)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print(123456.0)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 2u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["print".into(), 123456i64.into(), 123456.0.into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}

#[test]
fn local() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local a = "hello, local!"  -- define a local by string
local b = a  -- define a local by another local
print(b)  -- print local variable
print(print)  -- print global variable
local print = print  -- define a local by global variable with same name
print "I'm local-print!"  -- call local function
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a = "hello, local!"
            Bytecode::load_constant(0.into(), 0u8.into()),
            // local b = a
            Bytecode::move_bytecode(1.into(), 0.into()),
            // print(b)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::move_bytecode(3.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(print)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // local print = print
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            // print "I'm local-print!"
            Bytecode::move_bytecode(3.into(), 2.into()),
            Bytecode::load_constant(4.into(), 2u8.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(3.into(), 1.into(), 1.into()),
        ],
        &[
            "hello, local!".into(),
            "print".into(),
            "I'm local-print!".into(),
        ],
        &[
            Local::new("a".into(), 3, 15),
            Local::new("b".into(), 4, 15),
            Local::new("print".into(), 11, 15),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}

#[test]
fn assign() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a = 456
            Bytecode::load_integer(0.into(), 456i16.into()),
            // a = 123
            Bytecode::load_integer(0.into(), 123i16.into()),
            // print(a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // a = a
            // print(a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // a = g
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            // print(a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // g = 123
            Bytecode::set_uptable(0.into(), 1.into(), 2.into(), true.into()),
            // print(g)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // g = a
            Bytecode::set_uptable(0.into(), 1.into(), 0.into(), false.into()),
            // print(g)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // g = g2
            Bytecode::get_uptable(1.into(), 0.into(), 3.into()),
            Bytecode::set_uptable(0.into(), 1.into(), 1.into(), false.into()),
            // print(g)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["print".into(), "g".into(), 123i64.into(), "g2".into()],
        &[Local::new("a".into(), 3, 28)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}
