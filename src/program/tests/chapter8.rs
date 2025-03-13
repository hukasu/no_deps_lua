use core::cell::RefCell;

use alloc::rc::Rc;

use crate::{
    bytecode::Bytecode,
    table::Table,
    value::{Value, ValueKey},
    Error, Program,
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
            Bytecode::variadic_arguments_prepare(0),
            // local a, b = 1, 2
            Bytecode::load_integer(0, 1),
            Bytecode::load_integer(1, 2),
            // local function hello()
            Bytecode::closure(2, 0),
            // hello()
            Bytecode::move_bytecode(3, 2),
            Bytecode::call(3, 1, 1),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
        ],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function hello()
            //     local a = 4
            Bytecode::load_integer(0, 4),
            //     print (a)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local function hello()
            Bytecode::closure(0, 0),
            // hello()
            Bytecode::move_bytecode(1, 0),
            Bytecode::call(1, 1, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function hello()
            // print "hello, function!"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello, function!".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local function hello()
            Bytecode::closure(0, 0),
            // print(hello)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &["print".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function hello()
            //     print "hello, function!"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello, function!".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local function f1()
            Bytecode::closure(0, 0),
            // print (f1)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &["print".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function f1()
            //     local f2 = function() print "internal" end
            Bytecode::closure(0, 0),
            //     print (f2)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(1, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(closure, 0);
    super::compare_program(
        closure,
        &[
            // local f2 = function()
            //      print "internal"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "internal".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local t = {}
            Bytecode::new_table(0, 0, 0),
            // function t.f() print "hello" end
            Bytecode::closure(1, 0),
            Bytecode::set_field(0, 0, 1, 0),
            // print(t.f)
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::get_field(2, 0, 0),
            Bytecode::call(1, 2, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &["f".into(), "print".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function t.f()
            //      print "hello"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local function f(a, b)
            Bytecode::closure(0, 0),
            // f(1,2)
            Bytecode::move_bytecode(1, 0),
            Bytecode::load_integer(2, 1),
            Bytecode::load_integer(3, 2),
            Bytecode::call(1, 3, 1),
            // f(100,200)
            Bytecode::move_bytecode(1, 0),
            Bytecode::load_integer(2, 100),
            Bytecode::load_integer(3, 200),
            Bytecode::call(1, 3, 1),
            // f(1,2,3)
            Bytecode::move_bytecode(1, 0),
            Bytecode::load_integer(2, 1),
            Bytecode::load_integer(3, 2),
            Bytecode::load_integer(4, 3),
            Bytecode::call(1, 4, 1),
            // f(1)
            Bytecode::move_bytecode(1, 0),
            Bytecode::load_integer(2, 1),
            Bytecode::call(1, 2, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &[],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function f(a, b)
            // print(a+b)
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::add(3, 0, 1),
            Bytecode::call(2, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local function f(a, b)
            Bytecode::closure(0, 0),
            // print(f(1,2))
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_integer(3, 1),
            Bytecode::load_integer(4, 2),
            Bytecode::call(2, 3, 0),
            Bytecode::call(1, 0, 1),
            // print(f(100,200))
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_integer(3, 100),
            Bytecode::load_integer(4, 200),
            Bytecode::call(2, 3, 0),
            Bytecode::call(1, 0, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &["print".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function f(a, b)
            //     return a+b
            Bytecode::add(2, 0, 1),
            Bytecode::one_return(2),
            // end
            Bytecode::zero_return(),
        ],
        &[],
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
            Bytecode::variadic_arguments_prepare(0),
            // print(type(123))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::load_integer(2, 123),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // print(type(123.123))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::load_constant(2, 2),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // print(type("123"))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::load_constant(2, 3),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // print(type({}))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::new_table(2, 0, 0),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // print(type(print))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // print(type(function()end))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::closure(2, 0),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &["print".into(), "type".into(), 123.123.into(), "123".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(closure, &[Bytecode::zero_return()], &[], &[], 0);

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
            Bytecode::variadic_arguments_prepare(0),
            // function f(n)
            Bytecode::closure(0, 0),
            Bytecode::set_uptable(0, 0, 0, 0),
            // print(f(0))
            Bytecode::get_uptable(0, 0, 1),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::load_integer(2, 0),
            Bytecode::call(1, 2, 0),
            Bytecode::call(0, 0, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &["f".into(), "print".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f(n)
            //     if n > 10000 then return n end
            Bytecode::load_integer(1, 10000),
            Bytecode::less_than(1, 0, 0),
            Bytecode::jump(1),
            Bytecode::one_return(0),
            //     return f(n+1)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::add_integer(2, 0, 1),
            Bytecode::tail_call(1, 2, 0),
            Bytecode::return_bytecode(1, 0, 0),
            // end
            Bytecode::zero_return(),
        ],
        &["f".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // print(1,2,3)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_integer(1, 1),
            Bytecode::load_integer(2, 2),
            Bytecode::load_integer(3, 3),
            Bytecode::call(0, 4, 1),
            // function f(...)
            Bytecode::closure(0, 0),
            Bytecode::set_uptable(0, 1, 0, 0),
            // f(100,200,"hello")
            Bytecode::get_uptable(0, 0, 1),
            Bytecode::load_integer(1, 100),
            Bytecode::load_integer(2, 200),
            Bytecode::load_constant(3, 2),
            Bytecode::call(0, 4, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &["print".into(), "f".into(), "hello".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f(...)
            Bytecode::variadic_arguments_prepare(0),
            //     print(print(...))
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::variadic_arguments(2, 0),
            Bytecode::call(1, 0, 0),
            Bytecode::call(0, 0, 1),
            // end
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &["print".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // function f(x, ...)
            Bytecode::closure(0, 0),
            Bytecode::set_uptable(0, 0, 0, 0),
            // function f2(x, ...)
            Bytecode::closure(0, 1),
            Bytecode::set_uptable(0, 1, 0, 0),
            // function f3(x, ...)
            Bytecode::closure(0, 2),
            Bytecode::set_uptable(0, 2, 0, 0),
            // f('x', 1,2,3)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 3),
            Bytecode::load_integer(2, 1),
            Bytecode::load_integer(3, 2),
            Bytecode::load_integer(4, 3),
            Bytecode::call(0, 5, 1),
            // f('x', 1,2)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 3),
            Bytecode::load_integer(2, 1),
            Bytecode::load_integer(3, 2),
            Bytecode::call(0, 4, 1),
            // f2('x', 1,2,3,4)
            Bytecode::get_uptable(0, 0, 1),
            Bytecode::load_constant(1, 3),
            Bytecode::load_integer(2, 1),
            Bytecode::load_integer(3, 2),
            Bytecode::load_integer(4, 3),
            Bytecode::load_integer(5, 4),
            Bytecode::call(0, 6, 1),
            // f3('x', 1,2,3,4)
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3),
            Bytecode::load_integer(2, 1),
            Bytecode::load_integer(3, 2),
            Bytecode::load_integer(4, 3),
            Bytecode::load_integer(5, 4),
            Bytecode::call(0, 6, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &["f".into(), "f2".into(), "f3".into(), "x".into()],
        &["_ENV".into()],
        3,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f(x, ...)
            Bytecode::variadic_arguments_prepare(1),
            //     local a,b,c = ...
            Bytecode::variadic_arguments(1, 4),
            //     print(x)
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::move_bytecode(5, 0),
            Bytecode::call(4, 2, 1),
            //     print(a)
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::move_bytecode(5, 1),
            Bytecode::call(4, 2, 1),
            //     print(b)
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::move_bytecode(5, 2),
            Bytecode::call(4, 2, 1),
            //     print(c)
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::move_bytecode(5, 3),
            Bytecode::call(4, 2, 1),
            // end
            Bytecode::return_bytecode(4, 1, 2),
        ],
        &["print".into()],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function f2(x, ...)
            Bytecode::variadic_arguments_prepare(1),
            //     f(x,...)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::variadic_arguments(3, 0),
            Bytecode::call(1, 0, 1),
            // end
            Bytecode::return_bytecode(1, 1, 2),
        ],
        &["f".into()],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 2);
    super::compare_program(
        closure,
        &[
            // function f3(x, ...)
            Bytecode::variadic_arguments_prepare(1),
            //     f(...,x)
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::variadic_arguments(2, 2),
            Bytecode::move_bytecode(3, 0),
            Bytecode::call(1, 3, 1),
            // end
            Bytecode::return_bytecode(1, 1, 2),
        ],
        &["f".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // function foo(a, b, ...)
            Bytecode::closure(0, 0),
            Bytecode::set_uptable(0, 0, 0, 0),
            // foo(1)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_integer(1, 1),
            Bytecode::call(0, 2, 1),
            // foo(1,2,100,200,300)
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_integer(1, 1),
            Bytecode::load_integer(2, 2),
            Bytecode::load_integer(3, 100),
            Bytecode::load_integer(4, 200),
            Bytecode::load_integer(5, 300),
            Bytecode::call(0, 6, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &["foo".into()],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function foo(a, b, ...)
            Bytecode::variadic_arguments_prepare(2),
            //     local t = {a, ...}
            Bytecode::new_table(2, 0, 1),
            Bytecode::move_bytecode(3, 0),
            Bytecode::variadic_arguments(4, 0),
            Bytecode::set_list(2, 0, 0),
            //     print(t[1], t[2], t[3], t[4])
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_index(4, 2, 1),
            Bytecode::get_index(5, 2, 2),
            Bytecode::get_index(6, 2, 3),
            Bytecode::get_index(7, 2, 4),
            Bytecode::call(3, 5, 1),
            //     local t = {a, ..., b}
            Bytecode::new_table(3, 0, 3),
            Bytecode::move_bytecode(4, 0),
            Bytecode::variadic_arguments(5, 2),
            Bytecode::move_bytecode(6, 1),
            Bytecode::set_list(3, 3, 0),
            //     print(t[1], t[2], t[3], t[4])
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::get_index(5, 3, 1),
            Bytecode::get_index(6, 3, 2),
            Bytecode::get_index(7, 3, 3),
            Bytecode::get_index(8, 3, 4),
            Bytecode::call(4, 5, 1),
            // end
            Bytecode::return_bytecode(4, 1, 3),
        ],
        &["print".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // function f1(a, b)
            Bytecode::closure(0, 0),
            Bytecode::set_uptable(0, 0, 0, 0),
            // function f2(a, b)
            Bytecode::closure(0, 1),
            Bytecode::set_uptable(0, 1, 0, 0),
            // x,y = f2(f2(3, 10)) -- MULTRET arguments
            Bytecode::get_uptable(0, 0, 1),
            Bytecode::get_uptable(1, 0, 1),
            Bytecode::load_integer(2, 3),
            Bytecode::load_integer(3, 10),
            Bytecode::call(1, 3, 0),
            Bytecode::call(0, 0, 3),
            Bytecode::set_uptable(0, 3, 1, 0),
            Bytecode::set_uptable(0, 2, 0, 0),
            // print(x)
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::get_uptable(1, 0, 2),
            Bytecode::call(0, 2, 1),
            // print(y)
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::get_uptable(1, 0, 3),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &[
            "f1".into(),
            "f2".into(),
            "x".into(),
            "y".into(),
            "print".into(),
        ],
        &["_ENV".into()],
        2,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function f1(a, b)
            //     return a+b, a-b
            Bytecode::add(2, 0, 1),
            Bytecode::sub(3, 0, 1),
            Bytecode::return_bytecode(2, 3, 0),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function f2(a, b)
            //     return f1(a+b, a-b) -- return MULTRET
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::add(3, 0, 1),
            Bytecode::sub(4, 0, 1),
            Bytecode::tail_call(2, 3, 0),
            Bytecode::return_bytecode(2, 0, 0),
            // end
            Bytecode::zero_return(),
        ],
        &["f1".into()],
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
            Bytecode::variadic_arguments_prepare(0),
            // local t = {11,12,13, ['methods']={7, 8, 9}}
            Bytecode::new_table(0, 1, 3),
            Bytecode::load_integer(1, 11),
            Bytecode::load_integer(2, 12),
            Bytecode::load_integer(3, 13),
            Bytecode::new_table(4, 0, 3),
            Bytecode::load_integer(5, 7),
            Bytecode::load_integer(6, 8),
            Bytecode::load_integer(7, 9),
            Bytecode::set_list(4, 3, 0),
            Bytecode::set_field(0, 0, 4, 0),
            Bytecode::set_list(0, 3, 0),
            // function t.methods.foo(a,b)
            Bytecode::get_field(1, 0, 0),
            Bytecode::closure(2, 0),
            Bytecode::set_field(1, 1, 2, 0),
            // function t.methods:bar(a,b)
            Bytecode::get_field(1, 0, 0),
            Bytecode::closure(2, 1),
            Bytecode::set_field(1, 2, 2, 0),
            // t.methods.foo(100, 200)
            Bytecode::get_field(1, 0, 0),
            Bytecode::get_field(1, 1, 1),
            Bytecode::load_integer(2, 100),
            Bytecode::load_integer(3, 200),
            Bytecode::call(1, 3, 1),
            // t.methods:bar(100, 200)
            Bytecode::get_field(1, 0, 0),
            Bytecode::table_self(1, 1, 2),
            Bytecode::load_integer(3, 100),
            Bytecode::load_integer(4, 200),
            Bytecode::call(1, 4, 1),
            // t.methods.bar(t, 100, 200)
            Bytecode::get_field(1, 0, 0),
            Bytecode::get_field(1, 1, 2),
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_integer(3, 100),
            Bytecode::load_integer(4, 200),
            Bytecode::call(1, 4, 1),
            // EOF
            Bytecode::return_bytecode(1, 1, 1),
        ],
        &["methods".into(), "foo".into(), "bar".into()],
        &["_ENV".into()],
        2,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function t.methods.foo(a,b)
            //     print(a+b)
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::add(3, 0, 1),
            Bytecode::call(2, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function t.methods:bar(a,b)
            //     print(self[1]+self[2]+a+b)
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_index(4, 0, 1),
            Bytecode::get_index(5, 0, 2),
            Bytecode::add(4, 4, 5),
            Bytecode::add(4, 4, 1),
            Bytecode::add(4, 4, 2),
            Bytecode::call(3, 2, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}
