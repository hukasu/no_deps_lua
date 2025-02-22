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

    let Value::Closure(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
        // local function hello()
        //     local a = 4
        ByteCode::LoadInt(0, 4),
        //     print (a)
        ByteCode::GetGlobal(1, 0),
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
        // local function hello()
        ByteCode::Closure(0, 0),
        // print(hello)
        ByteCode::GetGlobal(1, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
    ];
    assert_eq!(program.constants, &["print".into()]);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Closure(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "hello, function!".into()];
    let expected_bytecodes = &[
        // local function hello()
        //     print "hello, function!"
        ByteCode::GetGlobal(0, 0),
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
        // local function f1()
        ByteCode::Closure(0, 0),
        // print (f1)
        ByteCode::GetGlobal(1, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
    ];
    assert_eq!(program.constants, &["print".into()]);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Closure(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // local function f1()
        //     local f2 = function() print "internal" end
        ByteCode::Closure(0, 0),
        //     print (f2)
        ByteCode::GetGlobal(1, 0),
        ByteCode::Move(2, 0),
        ByteCode::Call(1, 2, 1),
        // end
        ByteCode::ZeroReturn,
    ];
    assert_eq!(func.program().constants, &["print".into()]);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert_eq!(func.program().functions.len(), 1);

    let Value::Closure(func) = &func.program().functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "internal".into()];
    let expected_bytecodes = &[
        // local f2 = function()
        //      print "internal"
        ByteCode::GetGlobal(0, 0),
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
        // local t = {}
        ByteCode::NewTable(0, 0, 0),
        // function t.f() print "hello" end
        ByteCode::Closure(1, 0),
        ByteCode::SetField(0, 0, 1),
        // print(t.f)
        ByteCode::GetGlobal(1, 1),
        ByteCode::GetField(2, 0, 0),
        ByteCode::Call(1, 2, 1),
        // EOF
        ByteCode::Return(1, 1, 1),
    ];
    assert_eq!(program.constants, &["f".into(), "print".into()]);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Closure(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_bytecodes = &[
        // function t.f()
        //      print "hello"
        ByteCode::GetGlobal(0, 0),
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

    let Value::Closure(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
        // local function f(a, b)
        // print(a+b)
        ByteCode::GetGlobal(2, 0),
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
        // local function f(a, b)
        ByteCode::Closure(0, 0),
        // print(f(1,2))
        ByteCode::GetGlobal(1, 0),
        ByteCode::Move(2, 0),
        ByteCode::LoadInt(3, 1),
        ByteCode::LoadInt(4, 2),
        ByteCode::Call(2, 3, 0),
        ByteCode::Call(1, 0, 1),
        // print(f(100,200))
        ByteCode::GetGlobal(1, 0),
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

    let Value::Closure(func) = &program.functions[0] else {
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

    assert!(matches!(vm.stack[0], Value::Closure(_)));
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
        // print(type(123))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 1),
        ByteCode::LoadInt(2, 123),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type(123.123))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 1),
        ByteCode::LoadConstant(2, 2),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type("123"))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 1),
        ByteCode::LoadConstant(2, 3),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type({}))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 1),
        ByteCode::NewTable(2, 0, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type(print))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 1),
        ByteCode::GetGlobal(2, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // print(type(function()end))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 1),
        ByteCode::Closure(2, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Closure(closure) = &program.functions[0] else {
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
        // function f(n)
        ByteCode::Closure(0, 0),
        ByteCode::SetGlobal(0, 0),
        // print(f(0))
        ByteCode::GetGlobal(0, 1),
        ByteCode::GetGlobal(1, 0),
        ByteCode::LoadInt(2, 0),
        ByteCode::Call(1, 2, 0),
        ByteCode::Call(0, 0, 1),
        // EOF
        ByteCode::Return(0, 1, 1),
    ];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Closure(closure) = &program.functions[0] else {
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
        ByteCode::GetGlobal(1, 0),
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
        // print(1,2,3)
        ByteCode::GetGlobal(0, 0),
        ByteCode::LoadInt(1, 1),
        ByteCode::LoadInt(2, 2),
        ByteCode::LoadInt(3, 3),
        ByteCode::Call(0, 4, 1),
        // function f(...)
        ByteCode::Closure(0, 0),
        ByteCode::SetGlobal(1, 0),
        // f(100,200,"hello")
        ByteCode::GetGlobal(0, 1),
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

    let Value::Closure(closure) = &program.functions[0] else {
        panic!("Closure must be a closure")
    };
    let expected_constants: &[Value] = &["print".into()];
    let expected_bytecodes = &[
        // function f(...)
        //     print(print(...))
        ByteCode::GetGlobal(0, 0),
        ByteCode::GetGlobal(1, 0),
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
