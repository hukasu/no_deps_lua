use core::cell::RefCell;

use alloc::rc::Rc;

use crate::{
    Error, Program,
    bytecode::Bytecode,
    program::Local,
    table::Table,
    value::{Value, ValueKey},
};

#[test]
fn base_function() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local a, b = 1, 2

local function hello()
    local a = 4
    print (a)
end

hello()
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a, b = 1, 2
            Bytecode::load_integer(0.into(), 1i8.into()),
            Bytecode::load_integer(1.into(), 2i8.into()),
            // local function hello()
            Bytecode::closure(2.into(), 0u8.into()),
            // hello()
            Bytecode::move_bytecode(3.into(), 2.into()),
            Bytecode::call(3.into(), 1.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(3.into(), 1.into(), 1.into()),
        ],
        &[],
        &[
            Local::new("a".into(), 4, 8),
            Local::new("b".into(), 4, 8),
            Local::new("hello".into(), 5, 8),
        ],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function hello()
            //     local a = 4
            Bytecode::load_integer(0.into(), 4i8.into()),
            //     print (a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &[Local::new("a".into(), 2, 6)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn call() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function hello()
    print "hello, function!"
end

hello()
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function hello()
            Bytecode::closure(0.into(), 0u8.into()),
            // hello()
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::call(1.into(), 1.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &[],
        &[Local::new("hello".into(), 3, 6)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function hello()
            // print "hello, function!"
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello, function!".into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn func1() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function hello()
    print "hello, function!"
end

print(hello)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function hello()
            Bytecode::closure(0.into(), 0u8.into()),
            // print(hello)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[Local::new("hello".into(), 3, 7)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function hello()
            //     print "hello, function!"
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello, function!".into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn func2() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function f1()
    local f2 = function() print "internal" end
    print (f2)
end

print (f1)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function f1()
            Bytecode::closure(0.into(), 0u8.into()),
            // print (f1)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[Local::new("f1".into(), 3, 7)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function f1()
            //     local f2 = function() print "internal" end
            Bytecode::closure(0.into(), 0u8.into()),
            //     print (f2)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &[Local::new("f2".into(), 2, 6)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(closure, 0);
    super::compare_program(
        closure,
        &[
            // local f2 = function()
            //      print "internal"
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "internal".into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn func3() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local t = {}
function t.f() print "hello" end
print(t.f)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local t = {}
            Bytecode::new_table(0.into(), 0.into(), 0.into()),
            // TODO EXTRAARG
            // function t.f() print "hello" end
            Bytecode::closure(1.into(), 0u8.into()),
            Bytecode::set_field(0.into(), 0.into(), 1.into(), false.into()),
            // print(t.f)
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::get_field(2.into(), 0.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["f".into(), "print".into()],
        &[
            // TODO update when implementing EXTRAARG
            Local::new("t".into(), 3, 9),
        ],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function t.f()
            //      print "hello"
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello".into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn args() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function f(a, b)
    print(a+b)
end

f(1,2)
f(100,200)
f(1,2,3)
f(1)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function f(a, b)
            Bytecode::closure(0.into(), 0u8.into()),
            // f(1,2)
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::load_integer(3.into(), 2i8.into()),
            Bytecode::call(1.into(), 3.into(), 1.into()),
            // f(100,200)
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::load_integer(2.into(), 100i8.into()),
            Bytecode::load_integer(3.into(), 200i16.into()),
            Bytecode::call(1.into(), 3.into(), 1.into()),
            // f(1,2,3)
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::load_integer(3.into(), 2i8.into()),
            Bytecode::load_integer(4.into(), 3i8.into()),
            Bytecode::call(1.into(), 4.into(), 1.into()),
            // f(1)
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &[],
        &[Local::new("f".into(), 3, 20)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function f(a, b)
            // print(a+b)
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::add(3.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &[
            // TODO update when implementing MMBIN
            Local::new("a".into(), 1, 5),
            Local::new("b".into(), 1, 5),
        ],
        &["_ENV".into()],
        0,
    );

    match crate::Lua::run_program(program).inspect_err(|err| log::error!("{err}")) {
        Ok(_) => panic!("Program should fail"),
        Err(Error::ArithmeticOperand("add", "integer", "nil")) => (),
        Err(err) => panic!("Program raised wrong error `{err}`."),
    }
}

#[test]
fn ret() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function f(a, b)
    return a+b
end

print(f(1,2))
print(f(100,200))
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function f(a, b)
            Bytecode::closure(0.into(), 0u8.into()),
            // print(f(1,2))
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::load_integer(3.into(), 1i8.into()),
            Bytecode::load_integer(4.into(), 2i8.into()),
            Bytecode::call(2.into(), 3.into(), 0.into()),
            Bytecode::call(1.into(), 0.into(), 1.into()),
            // print(f(100,200))
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::load_integer(3.into(), 100i8.into()),
            Bytecode::load_integer(4.into(), 200i16.into()),
            Bytecode::call(2.into(), 3.into(), 0.into()),
            Bytecode::call(1.into(), 0.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[Local::new("f".into(), 3, 16)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function f(a, b)
            //     return a+b
            Bytecode::add(2.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::one_return(2.into()),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[
            // TODO update when implementing MMBIN
            Local::new("a".into(), 1, 4),
            Local::new("b".into(), 1, 4),
        ],
        &[],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}

#[test]
fn rustf() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
print(type(123))
print(type(123.123))
print(type("123"))
print(type({}))
print(type(print))
print(type(function()end))
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print(type(123))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::load_integer(2.into(), 123i8.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // print(type(123.123))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::load_constant(2.into(), 2u8.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // print(type("123"))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::load_constant(2.into(), 3u8.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // print(type({}))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::new_table(2.into(), 0.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // print(type(print))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // print(type(function()end))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::closure(2.into(), 0u8.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["print".into(), "type".into(), 123.123.into(), "123".into()],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(closure, &[Bytecode::zero_return()], &[], &[], &[], 0);

    pub fn lib_type(vm: &mut crate::Lua) -> i32 {
        let type_name = vm.get_stack(0).unwrap();
        vm.set_stack(0, type_name.static_type_name().into())
            .unwrap();

        1
    }

    let mut env = Table::new(0, 2);
    env.set(
        ValueKey::from(Value::from("print")),
        Value::from(crate::std::lib_print as fn(&mut crate::Lua) -> i32),
    )
    .unwrap();
    env.set(
        ValueKey::from(Value::from("type")),
        Value::from(lib_type as fn(&mut crate::Lua) -> i32),
    )
    .unwrap();

    crate::Lua::run_program_with_env(program, Rc::new(RefCell::new(env))).expect("Should work");
}

#[test]
fn tailcall() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
function f(n)
    if n > 10000 then return n end
    return f(n+1)
end

print(f(0))
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // function f(n)
            Bytecode::closure(0.into(), 0u8.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 0.into(), false.into()),
            // print(f(0))
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_integer(2.into(), 0i8.into()),
            Bytecode::call(1.into(), 2.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["f".into(), "print".into()],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f(n)
            //     if n > 10000 then return n end
            Bytecode::load_integer(1.into(), 10000i16.into()),
            Bytecode::less_than(1.into(), 0.into(), false.into()),
            Bytecode::jump(1i16.into()),
            Bytecode::one_return(0.into()),
            //     return f(n+1)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::add_integer(2.into(), 0.into(), 1.into()),
            // TODO MMBINI
            Bytecode::tail_call(1.into(), 2.into(), 0.into()),
            Bytecode::return_bytecode(1.into(), 0.into(), 0.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["f".into()],
        &[
            // TODO update when implementing MMBINI
            Local::new("n".into(), 1, 10),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}

#[test]
fn print() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
print(1,2,3)
function f(...)
    print(print(...))
end
f(100,200,"hello")
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print(1,2,3)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_integer(1.into(), 1i8.into()),
            Bytecode::load_integer(2.into(), 2i8.into()),
            Bytecode::load_integer(3.into(), 3i8.into()),
            Bytecode::call(0.into(), 4.into(), 1.into()),
            // function f(...)
            Bytecode::closure(0.into(), 0u8.into()),
            Bytecode::set_uptable(0.into(), 1.into(), 0.into(), false.into()),
            // f(100,200,"hello")
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::load_integer(1.into(), 100i8.into()),
            Bytecode::load_integer(2.into(), 200i16.into()),
            Bytecode::load_constant(3.into(), 2u8.into()),
            Bytecode::call(0.into(), 4.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["print".into(), "f".into(), "hello".into()],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f(...)
            Bytecode::variadic_arguments_prepare(0.into()),
            //     print(print(...))
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::variadic_arguments(2.into(), 0.into()),
            Bytecode::call(1.into(), 0.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 1.into()),
            // end
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}

#[test]
fn varargs() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
function f(x, ...)
    local a,b,c = ...
    print(x)
    print(a)
    print(b)
    print(c)
end
function f2(x, ...)
    f(x,...)
end
function f3(x, ...)
    f(...,x)
end

f('x', 1,2,3)
f('x', 1,2)
f2('x', 1,2,3,4)
f3('x', 1,2,3,4)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // function f(x, ...)
            Bytecode::closure(0.into(), 0u8.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 0.into(), false.into()),
            // function f2(x, ...)
            Bytecode::closure(0.into(), 1u8.into()),
            Bytecode::set_uptable(0.into(), 1.into(), 0.into(), false.into()),
            // function f3(x, ...)
            Bytecode::closure(0.into(), 2u8.into()),
            Bytecode::set_uptable(0.into(), 2.into(), 0.into(), false.into()),
            // f('x', 1,2,3)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::load_integer(3.into(), 2i8.into()),
            Bytecode::load_integer(4.into(), 3i8.into()),
            Bytecode::call(0.into(), 5.into(), 1.into()),
            // f('x', 1,2)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::load_integer(3.into(), 2i8.into()),
            Bytecode::call(0.into(), 4.into(), 1.into()),
            // f2('x', 1,2,3,4)
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::load_integer(3.into(), 2i8.into()),
            Bytecode::load_integer(4.into(), 3i8.into()),
            Bytecode::load_integer(5.into(), 4i8.into()),
            Bytecode::call(0.into(), 6.into(), 1.into()),
            // f3('x', 1,2,3,4)
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::load_integer(3.into(), 2i8.into()),
            Bytecode::load_integer(4.into(), 3i8.into()),
            Bytecode::load_integer(5.into(), 4i8.into()),
            Bytecode::call(0.into(), 6.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["f".into(), "f2".into(), "f3".into(), "x".into()],
        &[],
        &["_ENV".into()],
        3,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f(x, ...)
            Bytecode::variadic_arguments_prepare(1.into()),
            //     local a,b,c = ...
            Bytecode::variadic_arguments(1.into(), 4.into()),
            //     print(x)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 0.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            //     print(a)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 1.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            //     print(b)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 2.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            //     print(c)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 3.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            // end
            Bytecode::return_bytecode(4.into(), 1.into(), 2.into()),
        ],
        &["print".into()],
        &[
            Local::new("x".into(), 1, 16),
            Local::new("a".into(), 3, 16),
            Local::new("b".into(), 3, 16),
            Local::new("c".into(), 3, 16),
        ],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function f2(x, ...)
            Bytecode::variadic_arguments_prepare(1.into()),
            //     f(x,...)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::variadic_arguments(3.into(), 0.into()),
            Bytecode::call(1.into(), 0.into(), 1.into()),
            // end
            Bytecode::return_bytecode(1.into(), 1.into(), 2.into()),
        ],
        &["f".into()],
        &[Local::new("x".into(), 1, 7)],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 2);
    super::compare_program(
        closure,
        &[
            // function f3(x, ...)
            Bytecode::variadic_arguments_prepare(1.into()),
            //     f(...,x)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::variadic_arguments(2.into(), 2.into()),
            Bytecode::move_bytecode(3.into(), 0.into()),
            Bytecode::call(1.into(), 3.into(), 1.into()),
            // end
            Bytecode::return_bytecode(1.into(), 1.into(), 2.into()),
        ],
        &["f".into()],
        &[Local::new("x".into(), 1, 7)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}

#[test]
fn varargs_table() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
function foo(a, b, ...)
    local t = {a, ...}
    print(t[1], t[2], t[3], t[4])
    local t = {a, ..., b}
    print(t[1], t[2], t[3], t[4])
end

foo(1)
foo(1,2,100,200,300)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // function foo(a, b, ...)
            Bytecode::closure(0.into(), 0u8.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 0.into(), false.into()),
            // foo(1)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_integer(1.into(), 1i8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // foo(1,2,100,200,300)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_integer(1.into(), 1i8.into()),
            Bytecode::load_integer(2.into(), 2i8.into()),
            Bytecode::load_integer(3.into(), 100i8.into()),
            Bytecode::load_integer(4.into(), 200i16.into()),
            Bytecode::load_integer(5.into(), 300i16.into()),
            Bytecode::call(0.into(), 6.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &["foo".into()],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function foo(a, b, ...)
            Bytecode::variadic_arguments_prepare(2.into()),
            //     local t = {a, ...}
            Bytecode::new_table(2.into(), 0.into(), 1.into()),
            // TODO EXTRAARG
            Bytecode::move_bytecode(3.into(), 0.into()),
            Bytecode::variadic_arguments(4.into(), 0.into()),
            Bytecode::set_list(2.into(), 0.into(), 0.into()),
            //     print(t[1], t[2], t[3], t[4])
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_index(4.into(), 2.into(), 1.into()),
            Bytecode::get_index(5.into(), 2.into(), 2.into()),
            Bytecode::get_index(6.into(), 2.into(), 3.into()),
            Bytecode::get_index(7.into(), 2.into(), 4.into()),
            Bytecode::call(3.into(), 5.into(), 1.into()),
            //     local t = {a, ..., b}
            Bytecode::new_table(3.into(), 0.into(), 3.into()),
            // TODO EXTRAARG
            Bytecode::move_bytecode(4.into(), 0.into()),
            Bytecode::variadic_arguments(5.into(), 2.into()),
            Bytecode::move_bytecode(6.into(), 1.into()),
            Bytecode::set_list(3.into(), 3.into(), 0.into()),
            //     print(t[1], t[2], t[3], t[4])
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::get_index(5.into(), 3.into(), 1.into()),
            Bytecode::get_index(6.into(), 3.into(), 2.into()),
            Bytecode::get_index(7.into(), 3.into(), 3.into()),
            Bytecode::get_index(8.into(), 3.into(), 4.into()),
            Bytecode::call(4.into(), 5.into(), 1.into()),
            // end
            Bytecode::return_bytecode(4.into(), 1.into(), 3.into()),
        ],
        &["print".into()],
        &[
            // TODO update when implementing EXTRAARG
            Local::new("a".into(), 1, 24),
            Local::new("b".into(), 1, 24),
            Local::new("t".into(), 6, 24),
            Local::new("t".into(), 17, 24),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should work");
}

#[test]
fn multret() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
function f1(a, b)
    return a+b, a-b
end
function f2(a, b)
    return f1(a+b, a-b) -- return MULTRET
end

x,y = f2(f2(3, 10)) -- MULTRET arguments
print(x)
print(y)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // function f1(a, b)
            Bytecode::closure(0.into(), 0u8.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 0.into(), false.into()),
            // function f2(a, b)
            Bytecode::closure(0.into(), 1u8.into()),
            Bytecode::set_uptable(0.into(), 1.into(), 0.into(), false.into()),
            // x,y = f2(f2(3, 10)) -- MULTRET arguments
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::load_integer(2.into(), 3i8.into()),
            Bytecode::load_integer(3.into(), 10i8.into()),
            Bytecode::call(1.into(), 3.into(), 0.into()),
            Bytecode::call(0.into(), 0.into(), 3.into()),
            Bytecode::set_uptable(0.into(), 3.into(), 1.into(), false.into()),
            Bytecode::set_uptable(0.into(), 2.into(), 0.into(), false.into()),
            // print(x)
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 2.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print(y)
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 3.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &[
            "f1".into(),
            "f2".into(),
            "x".into(),
            "y".into(),
            "print".into(),
        ],
        &[],
        &["_ENV".into()],
        2,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f1(a, b)
            //     return a+b, a-b
            Bytecode::add(2.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::sub(3.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::return_bytecode(2.into(), 3.into(), 0.into()),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[
            // TODO update when implementing MMBIN
            Local::new("a".into(), 1, 5),
            Local::new("b".into(), 1, 5),
        ],
        &[],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function f2(a, b)
            //     return f1(a+b, a-b) -- return MULTRET
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::add(3.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::sub(4.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::tail_call(2.into(), 3.into(), 0.into()),
            Bytecode::return_bytecode(2.into(), 0.into(), 0.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["f1".into()],
        &[Local::new("a".into(), 1, 7), Local::new("b".into(), 1, 7)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn self_keyword() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local t = {11,12,13, ['methods']={7, 8, 9}}
function t.methods.foo(a,b)
    print(a+b)
end
function t.methods:bar(a,b)
    print(self[1]+self[2]+a+b)
end

t.methods.foo(100, 200)
t.methods:bar(100, 200)
t.methods.bar(t, 100, 200)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local t = {11,12,13, ['methods']={7, 8, 9}}
            Bytecode::new_table(0.into(), 1.into(), 3.into()),
            // TODO EXTRAARG
            Bytecode::load_integer(1.into(), 11i8.into()),
            Bytecode::load_integer(2.into(), 12i8.into()),
            Bytecode::load_integer(3.into(), 13i8.into()),
            Bytecode::new_table(4.into(), 0.into(), 3.into()),
            // TODO EXTRAARG
            Bytecode::load_integer(5.into(), 7i8.into()),
            Bytecode::load_integer(6.into(), 8i8.into()),
            Bytecode::load_integer(7.into(), 9i8.into()),
            Bytecode::set_list(4.into(), 3.into(), 0.into()),
            Bytecode::set_field(0.into(), 0.into(), 4.into(), false.into()),
            Bytecode::set_list(0.into(), 3.into(), 0.into()),
            // function t.methods.foo(a,b)
            Bytecode::get_field(1.into(), 0.into(), 0.into()),
            Bytecode::closure(2.into(), 0u8.into()),
            Bytecode::set_field(1.into(), 1.into(), 2.into(), false.into()),
            // function t.methods:bar(a,b)
            Bytecode::get_field(1.into(), 0.into(), 0.into()),
            Bytecode::closure(2.into(), 1u8.into()),
            Bytecode::set_field(1.into(), 2.into(), 2.into(), false.into()),
            // t.methods.foo(100, 200)
            Bytecode::get_field(1.into(), 0.into(), 0.into()),
            Bytecode::get_field(1.into(), 1.into(), 1.into()),
            Bytecode::load_integer(2.into(), 100i8.into()),
            Bytecode::load_integer(3.into(), 200i16.into()),
            Bytecode::call(1.into(), 3.into(), 1.into()),
            // t.methods:bar(100, 200)
            Bytecode::get_field(1.into(), 0.into(), 0.into()),
            Bytecode::table_self(1.into(), 1.into(), 2.into()),
            Bytecode::load_integer(3.into(), 100i8.into()),
            Bytecode::load_integer(4.into(), 200i16.into()),
            Bytecode::call(1.into(), 4.into(), 1.into()),
            // t.methods.bar(t, 100, 200)
            Bytecode::get_field(1.into(), 0.into(), 0.into()),
            Bytecode::get_field(1.into(), 1.into(), 2.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::load_integer(3.into(), 100i8.into()),
            Bytecode::load_integer(4.into(), 200i16.into()),
            Bytecode::call(1.into(), 4.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["methods".into(), "foo".into(), "bar".into()],
        &[
            // TODO update when implementing EXTRAARG
            Local::new("t".into(), 13, 36),
        ],
        &["_ENV".into()],
        2,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function t.methods.foo(a,b)
            //     print(a+b)
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::add(3.into(), 0.into(), 1.into()),
            // TODO MMBIN
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &[
            // TODO update when implementing MMBIN
            Local::new("a".into(), 1, 5),
            Local::new("b".into(), 1, 5),
        ],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function t.methods:bar(a,b)
            //     print(self[1]+self[2]+a+b)
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_index(4.into(), 0.into(), 1.into()),
            Bytecode::get_index(5.into(), 0.into(), 2.into()),
            Bytecode::add(4.into(), 4.into(), 5.into()),
            // TODO MMBIN
            Bytecode::add(4.into(), 4.into(), 1.into()),
            // TODO MMBIN
            Bytecode::add(4.into(), 4.into(), 2.into()),
            // TODO MMBIN
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &[
            // TODO update when implementing MMBIN
            Local::new("self".into(), 1, 9),
            Local::new("a".into(), 1, 9),
            Local::new("b".into(), 1, 9),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}
