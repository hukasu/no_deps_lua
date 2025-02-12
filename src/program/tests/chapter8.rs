use crate::{byte_code::ByteCode, value::Value, Program};

#[test]
fn base_function() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());

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
        ByteCode::Call(3, 1),
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
    assert_eq!(func.constants, expected_constants);
    assert_eq!(func.byte_codes, expected_bytecodes);
    assert!(func.functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
}
