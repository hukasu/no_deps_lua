use crate::{byte_code::ByteCode, value::Value, Lua, Program};

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
        ByteCode::Call(3, 0),
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
        ByteCode::Call(1, 1),
        // end
        ByteCode::ZeroReturn,
    ];
    assert_eq!(func.program().constants, expected_constants);
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
        ByteCode::Call(1, 2),
        // f(100,200)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 100),
        ByteCode::LoadInt(3, 200),
        ByteCode::Call(1, 2),
        // f(1,2,3)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 1),
        ByteCode::LoadInt(3, 2),
        ByteCode::LoadInt(4, 3),
        ByteCode::Call(1, 3),
        // f(1)
        ByteCode::Move(1, 0),
        ByteCode::LoadInt(2, 1),
        ByteCode::Call(1, 1),
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
        ByteCode::Call(2, 1),
        // end
        ByteCode::ZeroReturn,
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program)
        .inspect_err(|err| log::error!("{err}"))
        .expect_err("Last call should fail due to adding 1 to `nil`");
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
        ByteCode::Call(2, 2),
        ByteCode::Call(1, 1),
        // print(f(100,200))
        ByteCode::GetGlobal(1, 0),
        ByteCode::Move(2, 0),
        ByteCode::LoadInt(3, 100),
        ByteCode::LoadInt(4, 200),
        ByteCode::Call(2, 2),
        ByteCode::Call(1, 1),
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

    let mut vm = Lua::new();
    vm.run_program(&program).expect("Should work");

    assert!(matches!(vm.stack[0], Value::Closure(_)));
    assert!(matches!(vm.stack[1], Value::Function(_)));
    assert_eq!(vm.stack[2], Value::Integer(300));
}
