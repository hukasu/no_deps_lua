use crate::{bytecode::Bytecode, value::Value, Error, Program};

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

    let expected_bytecodes = &[
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
    ];
    assert!(program.constants.is_empty());
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
        // local function hello()
        //     local a = 4
        Bytecode::load_integer(0, 4),
        //     print (a)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::move_bytecode(2, 0),
        Bytecode::call(1, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
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

    let expected_bytecodes = &[
        Bytecode::variadic_arguments_prepare(0),
        // local function hello()
        Bytecode::closure(0, 0),
        // hello()
        Bytecode::move_bytecode(1, 0),
        Bytecode::call(1, 1, 1),
        // EOF
        Bytecode::return_bytecode(1, 1, 1),
    ];
    assert!(program.constants.is_empty());
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "hello, function!".into()];
    let expected_bytecodes = &[
        // local function hello()
        // print "hello, function!"
        Bytecode::get_uptable(0, 0, 0),
        Bytecode::load_constant(1, 1),
        Bytecode::call(0, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
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

    let expected_bytecodes = &[
        Bytecode::variadic_arguments_prepare(0),
        // local function hello()
        Bytecode::closure(0, 0),
        // print(hello)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::move_bytecode(2, 0),
        Bytecode::call(1, 2, 1),
        // EOF
        Bytecode::return_bytecode(1, 1, 1),
    ];
    assert_eq!(program.constants, &["print".into()]);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "hello, function!".into()];
    let expected_bytecodes = &[
        // local function hello()
        //     print "hello, function!"
        Bytecode::get_uptable(0, 0, 0),
        Bytecode::load_constant(1, 1),
        Bytecode::call(0, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
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

    let expected_bytecodes = &[
        Bytecode::variadic_arguments_prepare(0),
        // local function f1()
        Bytecode::closure(0, 0),
        // print (f1)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::move_bytecode(2, 0),
        Bytecode::call(1, 2, 1),
        // EOF
        Bytecode::return_bytecode(1, 1, 1),
    ];
    assert_eq!(program.constants, &["print".into()]);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // local function f1()
        //     local f2 = function() print "internal" end
        Bytecode::closure(0, 0),
        //     print (f2)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::move_bytecode(2, 0),
        Bytecode::call(1, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, &["print".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert_eq!(func.program().functions.len(), 1);

    let Value::Function(func) = &func.program().functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "internal".into()];
    let expected_bytecodes = &[
        // local f2 = function()
        //      print "internal"
        Bytecode::get_uptable(0, 0, 0),
        Bytecode::load_constant(1, 1),
        Bytecode::call(0, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
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

    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, &["f".into(), "print".into()]);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // function t.f()
        //      print "hello"
        Bytecode::get_uptable(0, 0, 0),
        Bytecode::load_constant(1, 1),
        Bytecode::call(0, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, &["print".into(), "hello".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
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

    let expected_bytecodes = &[
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
    ];
    assert!(program.constants.is_empty());
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
        // local function f(a, b)
        // print(a+b)
        Bytecode::get_uptable(2, 0, 0),
        Bytecode::add(3, 0, 1),
        Bytecode::call(2, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    match crate::Lua::execute(&program).inspect_err(|err| log::error!("{err}")) {
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

    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // local function f(a, b)
        //     return a+b
        Bytecode::add(2, 0, 1),
        Bytecode::one_return(2),
        // end
        Bytecode::zero_return(),
    ];
    assert!(func.program().constants.is_empty());
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    let mut vm = crate::Lua::new();
    vm.run_program(&program).expect("Should work");
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

    let expected_constants: &[Value] =
        &["print".into(), "type".into(), 123.123.into(), "123".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    assert!(closure.program().constants.is_empty());
    assert_eq!(closure.program().byte_codes, &[Bytecode::zero_return()]);
    assert!(closure.program().functions.is_empty());

    let mut vm = crate::Lua::new();
    vm.run_program(&program).expect("Should work");
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

    let expected_constants: &[Value] = &["f".into(), "print".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["f".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(closure.program().constants, expected_constants);
    assert_eq!(closure.program().byte_codes, expected_bytecodes);
    assert!(closure.program().functions.is_empty());

    let mut vm = crate::Lua::new();
    vm.run_program(&program).expect("Should work");
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

    let expected_constants: &[Value] = &["print".into(), "f".into(), "hello".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(closure.program().constants, expected_constants);
    assert_eq!(closure.program().byte_codes, expected_bytecodes);
    assert!(closure.program().functions.is_empty());

    let mut vm = crate::Lua::new();
    vm.run_program(&program).expect("Should work");
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

    let expected_constants: &[Value] = &["f".into(), "f2".into(), "f3".into(), "x".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 3);

    let Value::Function(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(closure.program().constants, expected_constants);
    assert_eq!(closure.program().byte_codes, expected_bytecodes);
    assert!(closure.program().functions.is_empty());

    let Value::Function(closure) = &program.functions[1] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["f".into()];
    let expected_bytecodes = &[
        // function f2(x, ...)
        Bytecode::variadic_arguments_prepare(1),
        //     f(x,...)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::move_bytecode(2, 0),
        Bytecode::variadic_arguments(3, 0),
        Bytecode::call(1, 0, 1),
        // end
        Bytecode::return_bytecode(1, 1, 2),
    ];
    assert_eq!(closure.program().constants, expected_constants);
    assert_eq!(closure.program().byte_codes, expected_bytecodes);
    assert!(closure.program().functions.is_empty());

    let Value::Function(closure) = &program.functions[2] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["f".into()];
    let expected_bytecodes = &[
        // function f3(x, ...)
        Bytecode::variadic_arguments_prepare(1),
        //     f(...,x)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::variadic_arguments(2, 2),
        Bytecode::move_bytecode(3, 0),
        Bytecode::call(1, 3, 1),
        // end
        Bytecode::return_bytecode(1, 1, 2),
    ];
    assert_eq!(closure.program().constants, expected_constants);
    assert_eq!(closure.program().byte_codes, expected_bytecodes);
    assert!(closure.program().functions.is_empty());

    let mut vm = crate::Lua::new();
    vm.run_program(&program).expect("Should work");
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

    let expected_constants: &[Value] = &["foo".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
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
    ];
    assert_eq!(closure.program().constants, expected_constants);
    assert_eq!(closure.program().byte_codes, expected_bytecodes);
    assert!(closure.program().functions.is_empty());

    let mut vm = crate::Lua::new();
    vm.run_program(&program).expect("Should work");
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

    let expected_bytecodes = &[
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
    ];
    assert_eq!(
        program.constants,
        &[
            "f1".into(),
            "f2".into(),
            "x".into(),
            "y".into(),
            "print".into()
        ]
    );
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 2);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // function f1(a, b)
        //     return a+b, a-b
        Bytecode::add(2, 0, 1),
        Bytecode::sub(3, 0, 1),
        Bytecode::return_bytecode(2, 3, 0),
        // end
        Bytecode::zero_return(),
    ];
    assert!(func.program().constants.is_empty());
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    let Value::Function(func) = &program.functions[1] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // function f2(a, b)
        //     return f1(a+b, a-b) -- return MULTRET
        Bytecode::get_uptable(2, 0, 0),
        Bytecode::add(3, 0, 1),
        Bytecode::sub(4, 0, 1),
        Bytecode::tail_call(2, 3, 0),
        Bytecode::return_bytecode(2, 0, 0),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, &["f1".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
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

    let expected_bytecodes = &[
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
    ];
    assert_eq!(
        program.constants,
        &["methods".into(), "foo".into(), "bar".into(),]
    );
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 2);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // function t.methods.foo(a,b)
        //     print(a+b)
        Bytecode::get_uptable(2, 0, 0),
        Bytecode::add(3, 0, 1),
        Bytecode::call(2, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, &["print".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    let Value::Function(func) = &program.functions[1] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
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
    ];
    assert_eq!(func.program().constants, &["print".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
}
