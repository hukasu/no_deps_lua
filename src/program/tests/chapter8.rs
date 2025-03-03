use crate::{byte_code::ByteCode, value::Value, Error, Program};

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
        ByteCode::VariadicArgumentPrepare(0),
        // local a, b = 1, 2
        ByteCode::LoadInt(0, 1),
        ByteCode::LoadInt(1, 2),
        // local function hello()
        ByteCode::Closure(2, 0),
        // hello()
        ByteCode::Move(3, 2),
        ByteCode::Call(3, 1, 1),
        // EOF
        ByteCode::Return(3, 1, 1),
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
        ByteCode::LoadInt(0, 4),
        //     print (a)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local function hello()
        ByteCode::Closure(0, 0),
        // hello()
        ByteCode::Move(1, 0),
        ByteCode::Call(1, 1, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadConstant(1, 1),
        ByteCode::Call(0, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local function hello()
        ByteCode::Closure(0, 0),
        // print(hello)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadConstant(1, 1),
        ByteCode::Call(0, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local function f1()
        ByteCode::Closure(0, 0),
        // print (f1)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::Closure(0, 0),
        //     print (f2)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadConstant(1, 1),
        ByteCode::Call(0, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local t = {}
        ByteCode::NewTable(0, 0, 0),
        // function t.f() print "hello" end
        ByteCode::Closure(1, 0),
        ByteCode::SetField(0, 0, 1),
        // print(t.f)
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::GetField(2, 0, 0),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadConstant(1, 1),
        ByteCode::Call(0, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local function f(a, b)
        ByteCode::Closure(0, 0),
        // f(1,2)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::Call(1, 3, 1),
        // f(100,200)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 100),
        ByteCode::LoadInt(3, 200),
        ByteCode::Call(1, 3, 1),
        // f(1,2,3)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::LoadInt(4, 3),
        ByteCode::Call(1, 4, 1),
        // f(1)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 1),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::GetUpTable(2, 0, 0),
        ByteCode::Add(3, 0, 1),
        ByteCode::Call(2, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local function f(a, b)
        ByteCode::Closure(0, 0),
        // print(f(1,2))
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::LoadInt(3, 1),
        ByteCode::LoadInt(4, 2),
        ByteCode::Call(2, 3, 0),
        ByteCode::Call(1, 0, 1),
        // print(f(100,200))
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::LoadInt(3, 100),
        ByteCode::LoadInt(4, 200),
        ByteCode::Call(2, 3, 0),
        ByteCode::Call(1, 0, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::Add(2, 0, 1),
        ByteCode::OneReturn(2),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // print(type(123))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::LoadInt(2, 123),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type(123.123))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::LoadConstant(2, 2),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type("123"))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::LoadConstant(2, 3),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type({}))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::NewTable(2, 0, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type(print))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::GetUpTable(2, 0, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type(function()end))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::Closure(2, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    assert!(closure.program().constants.is_empty());
    assert_eq!(closure.program().byte_codes, &[ByteCode::ZeroReturn]);
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
        ByteCode::VariadicArgumentPrepare(0),
        // function f(n)
        ByteCode::Closure(0, 0),
        ByteCode::SetUpTable(0, 0, 0),
        // print(f(0))
        ByteCode::GetUpTable(0, 0, 1),
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::LoadInt(2, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
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
        ByteCode::LoadInt(1, 10000),
        ByteCode::LessThan(1, 0, 0),
        ByteCode::Jmp(1),
        ByteCode::OneReturn(0),
        //     return f(n+1)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::AddInteger(2, 0, 1),
        ByteCode::TailCall(1, 2, 0),
        ByteCode::Return(1, 0, 0),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // print(1,2,3)
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadInt(1, 1),
        ByteCode::LoadInt(2, 2),
        ByteCode::LoadInt(3, 3),
        ByteCode::Call(0, 4, 1),
        // function f(...)
        ByteCode::Closure(0, 0),
        ByteCode::SetUpTable(0, 1, 0),
        // f(100,200,"hello")
        ByteCode::GetUpTable(0, 0, 1),
        ByteCode::LoadInt(1, 100),
        ByteCode::LoadInt(2, 200),
        ByteCode::LoadConstant(3, 2),
        ByteCode::Call(0, 4, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
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
        ByteCode::VariadicArgumentPrepare(0),
        //     print(print(...))
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::VariadicArguments(2, 0),
        ByteCode::Call(1, 0, 0),
        ByteCode::Call(0, 0, 1),
        // end
        ByteCode::Return(0, 1, 1),
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
        ByteCode::VariadicArgumentPrepare(0),
        // function f(x, ...)
        ByteCode::Closure(0, 0),
        ByteCode::SetUpTable(0, 0, 0),
        // function f2(x, ...)
        ByteCode::Closure(0, 1),
        ByteCode::SetUpTable(0, 1, 0),
        // function f3(x, ...)
        ByteCode::Closure(0, 2),
        ByteCode::SetUpTable(0, 2, 0),
        // f('x', 1,2,3)
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadConstant(1, 3),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::LoadInt(4, 3),
        ByteCode::Call(0, 5, 1),
        // f('x', 1,2)
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadConstant(1, 3),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::Call(0, 4, 1),
        // f2('x', 1,2,3,4)
        ByteCode::GetUpTable(0, 0, 1),
        ByteCode::LoadConstant(1, 3),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::LoadInt(4, 3),
        ByteCode::LoadInt(5, 4),
        ByteCode::Call(0, 6, 1),
        // f3('x', 1,2,3,4)
        ByteCode::GetUpTable(0, 0, 2),
        ByteCode::LoadConstant(1, 3),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::LoadInt(4, 3),
        ByteCode::LoadInt(5, 4),
        ByteCode::Call(0, 6, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
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
        ByteCode::VariadicArgumentPrepare(1),
        //     local a,b,c = ...
        ByteCode::VariadicArguments(1, 4),
        //     print(x)
        ByteCode::GetUpTable(4, 0, 0),
        ByteCode::Move(5, 0),
        ByteCode::Call(4, 2, 1),
        //     print(a)
        ByteCode::GetUpTable(4, 0, 0),
        ByteCode::Move(5, 1),
        ByteCode::Call(4, 2, 1),
        //     print(b)
        ByteCode::GetUpTable(4, 0, 0),
        ByteCode::Move(5, 2),
        ByteCode::Call(4, 2, 1),
        //     print(c)
        ByteCode::GetUpTable(4, 0, 0),
        ByteCode::Move(5, 3),
        ByteCode::Call(4, 2, 1),
        // end
        ByteCode::Return(4, 1, 2),
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
        ByteCode::VariadicArgumentPrepare(1),
        //     f(x,...)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::Move(2, 0),
        ByteCode::VariadicArguments(3, 0),
        ByteCode::Call(1, 0, 1),
        // end
        ByteCode::Return(1, 1, 2),
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
        ByteCode::VariadicArgumentPrepare(1),
        //     f(...,x)
        ByteCode::GetUpTable(1, 0, 0),
        ByteCode::VariadicArguments(2, 2),
        ByteCode::Move(3, 0),
        ByteCode::Call(1, 3, 1),
        // end
        ByteCode::Return(1, 1, 2),
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
        ByteCode::VariadicArgumentPrepare(0),
        // function foo(a, b, ...)
        ByteCode::Closure(0, 0),
        ByteCode::SetUpTable(0, 0, 0),
        // foo(1)
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadInt(1, 1),
        ByteCode::Call(0, 2, 1),
        // foo(1,2,100,200,300)
        ByteCode::GetUpTable(0, 0, 0),
        ByteCode::LoadInt(1, 1),
        ByteCode::LoadInt(2, 2),
        ByteCode::LoadInt(3, 100),
        ByteCode::LoadInt(4, 200),
        ByteCode::LoadInt(5, 300),
        ByteCode::Call(0, 6, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
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
        ByteCode::VariadicArgumentPrepare(2),
        //     local t = {a, ...}
        ByteCode::NewTable(2, 0, 1),
        ByteCode::Move(3, 0),
        ByteCode::VariadicArguments(4, 0),
        ByteCode::SetList(2, 0, 0),
        //     print(t[1], t[2], t[3], t[4])
        ByteCode::GetUpTable(3, 0, 0),
        ByteCode::GetInt(4, 2, 1),
        ByteCode::GetInt(5, 2, 2),
        ByteCode::GetInt(6, 2, 3),
        ByteCode::GetInt(7, 2, 4),
        ByteCode::Call(3, 5, 1),
        //     local t = {a, ..., b}
        ByteCode::NewTable(3, 0, 3),
        ByteCode::Move(4, 0),
        ByteCode::VariadicArguments(5, 2),
        ByteCode::Move(6, 1),
        ByteCode::SetList(3, 3, 0),
        //     print(t[1], t[2], t[3], t[4])
        ByteCode::GetUpTable(4, 0, 0),
        ByteCode::GetInt(5, 3, 1),
        ByteCode::GetInt(6, 3, 2),
        ByteCode::GetInt(7, 3, 3),
        ByteCode::GetInt(8, 3, 4),
        ByteCode::Call(4, 5, 1),
        // end
        ByteCode::Return(4, 1, 3),
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
        ByteCode::VariadicArgumentPrepare(0),
        // function f1(a, b)
        ByteCode::Closure(0, 0),
        ByteCode::SetUpTable(0, 0, 0),
        // function f2(a, b)
        ByteCode::Closure(0, 1),
        ByteCode::SetUpTable(0, 1, 0),
        // x,y = f2(f2(3, 10)) -- MULTRET arguments
        ByteCode::GetUpTable(0, 0, 1),
        ByteCode::GetUpTable(1, 0, 1),
        ByteCode::LoadInt(2, 3),
        ByteCode::LoadInt(3, 10),
        ByteCode::Call(1, 3, 0),
        ByteCode::Call(0, 0, 3),
        ByteCode::SetUpTable(0, 3, 1),
        ByteCode::SetUpTable(0, 2, 0),
        // print(x)
        ByteCode::GetUpTable(0, 0, 4),
        ByteCode::GetUpTable(1, 0, 2),
        ByteCode::Call(0, 2, 1),
        // print(y)
        ByteCode::GetUpTable(0, 0, 4),
        ByteCode::GetUpTable(1, 0, 3),
        ByteCode::Call(0, 2, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
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
        ByteCode::Add(2, 0, 1),
        ByteCode::Sub(3, 0, 1),
        ByteCode::Return(2, 3, 0),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::GetUpTable(2, 0, 0),
        ByteCode::Add(3, 0, 1),
        ByteCode::Sub(4, 0, 1),
        ByteCode::TailCall(2, 3, 0),
        ByteCode::Return(2, 0, 0),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::VariadicArgumentPrepare(0),
        // local t = {11,12,13, ['methods']={7, 8, 9}}
        ByteCode::NewTable(0, 1, 3),
        ByteCode::LoadInt(1, 11),
        ByteCode::LoadInt(2, 12),
        ByteCode::LoadInt(3, 13),
        ByteCode::NewTable(4, 0, 3),
        ByteCode::LoadInt(5, 7),
        ByteCode::LoadInt(6, 8),
        ByteCode::LoadInt(7, 9),
        ByteCode::SetList(4, 3, 0),
        ByteCode::SetField(0, 0, 4),
        ByteCode::SetList(0, 3, 0),
        // function t.methods.foo(a,b)
        ByteCode::GetField(1, 0, 0),
        ByteCode::Closure(2, 0),
        ByteCode::SetField(1, 1, 2),
        // function t.methods:bar(a,b)
        ByteCode::GetField(1, 0, 0),
        ByteCode::Closure(2, 1),
        ByteCode::SetField(1, 2, 2),
        // t.methods.foo(100, 200)
        ByteCode::GetField(1, 0, 0),
        ByteCode::GetField(1, 1, 1),
        ByteCode::LoadInt(2, 100),
        ByteCode::LoadInt(3, 200),
        ByteCode::Call(1, 3, 1),
        // t.methods:bar(100, 200)
        ByteCode::GetField(1, 0, 0),
        ByteCode::TableSelf(1, 1, 2),
        ByteCode::LoadInt(3, 100),
        ByteCode::LoadInt(4, 200),
        ByteCode::Call(1, 4, 1),
        // t.methods.bar(t, 100, 200)
        ByteCode::GetField(1, 0, 0),
        ByteCode::GetField(1, 1, 2),
        ByteCode::Move(2, 0),
        ByteCode::LoadInt(3, 100),
        ByteCode::LoadInt(4, 200),
        ByteCode::Call(1, 4, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
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
        ByteCode::GetUpTable(2, 0, 0),
        ByteCode::Add(3, 0, 1),
        ByteCode::Call(2, 2, 1),
        // end
        ByteCode::ZeroReturn,
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
        ByteCode::GetUpTable(3, 0, 0),
        ByteCode::GetInt(4, 0, 1),
        ByteCode::GetInt(5, 0, 2),
        ByteCode::Add(4, 4, 5),
        ByteCode::Add(4, 4, 1),
        ByteCode::Add(4, 4, 2),
        ByteCode::Call(3, 2, 1),
        // end
        ByteCode::ZeroReturn,
    ];
    assert_eq!(func.program().constants, &["print".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
}
