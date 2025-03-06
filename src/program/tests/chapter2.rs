use crate::{bytecode::Bytecode, Program};

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
    assert_eq!(
        &program.constants,
        &["print".into(), 123456i64.into(), 123456.0.into(),]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // print(nil)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_nil(1, 0),
            Bytecode::call(0, 2, 1),
            // print(false)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_false(1),
            Bytecode::call(0, 2, 1),
            // print(123)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_integer(1, 123),
            Bytecode::call(0, 2, 1),
            // print(123456)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // print(123456.0)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 2),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
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
            Bytecode::variadic_arguments_prepare(0),
            // local a = "hello, local!"
            Bytecode::load_constant(0, 0),
            // local b = a
            Bytecode::move_bytecode(1, 0),
            // print(b)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::move_bytecode(3, 1),
            Bytecode::call(2, 2, 1),
            // print(print)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::get_uptable(3, 0, 1),
            Bytecode::call(2, 2, 1),
            // local print = print
            Bytecode::get_uptable(2, 0, 1),
            // print "I'm local-print!"
            Bytecode::move_bytecode(3, 2),
            Bytecode::load_constant(4, 2),
            Bytecode::call(3, 2, 1),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
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
    assert_eq!(
        &program.constants,
        &["print".into(), "g".into(), 123i64.into(), "g2".into(),]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local a = 456
            Bytecode::load_integer(0, 456),
            // a = 123
            Bytecode::load_integer(0, 123),
            // print(a)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // a = a
            // print(a)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // a = g
            Bytecode::get_uptable(0, 0, 1),
            // print(a)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // g = 123
            Bytecode::set_uptable(0, 1, 2, 1),
            // print(g)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::call(1, 2, 1),
            // g = a
            Bytecode::set_uptable(0, 1, 0, 0),
            // print(g)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::call(1, 2, 1),
            // g = g2
            Bytecode::get_uptable(1, 0, 3),
            Bytecode::set_uptable(0, 1, 1, 0),
            // print(g)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::call(1, 2, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
