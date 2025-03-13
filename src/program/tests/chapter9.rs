use crate::{bytecode::Bytecode, value::Value, Program};

#[test]
fn upvalues() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());

    let program = Program::parse(
        r#"
g1, g2 = 1, 2
local up1, up2, up3, up4 = 11, 12, 13, 14
local print = print
local function foo()
    local l1, l2 = 101, 102
    l1, g1 = g2, l2
    print(l1, g1)

    -- assign to upvalues
    up1, up2, up3 = l1, g1, up4
    print(up1, up2, up3)

    -- assign by upvalues
    l1, g1, up1 = up2, up3, up4
    print(l1, g1, up1)

    local inner = function()
        -- assign to upvalues
        up1, up2, up3 = 101, g2, up4
        print(up1, up2, up3)
    end
    inner()
end

foo()
"#,
    )
    .unwrap();
    let expected_bytecodes = &[
        Bytecode::variadic_arguments_prepare(0),
        // g1, g2 = 1, 2
        Bytecode::load_integer(0, 1),
        Bytecode::set_uptable(0, 1, 2, 1),
        Bytecode::set_uptable(0, 0, 0, 0),
        // local up1, up2, up3, up4 = 11, 12, 13, 14
        Bytecode::load_integer(0, 11),
        Bytecode::load_integer(1, 12),
        Bytecode::load_integer(2, 13),
        Bytecode::load_integer(3, 14),
        // local print = print
        Bytecode::get_uptable(4, 0, 3),
        // local function foo()
        Bytecode::closure(5, 0),
        // foo()
        Bytecode::move_bytecode(6, 5),
        Bytecode::call(6, 1, 1),
        // EOF
        Bytecode::return_bytecode(4, 1, 1),
    ];
    let expected_constants: &[Value] = &["g1".into(), "g2".into(), 2i64.into(), "print".into()];
    assert_eq!(program.constants, expected_constants);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    assert_eq!(program.functions.len(), 1);

    let Value::Function(func) = &program.functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "test _ENV:".into()];
    let expected_bytecodes = &[
        // local function my_print(a)
        //     print("test _ENV:", a)
        Bytecode::get_uptable(1, 0, 0),
        Bytecode::load_constant(2, 1),
        Bytecode::move_bytecode(3, 0),
        Bytecode::call(1, 3, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert_eq!(func.program().functions.len(), 1);

    let Value::Function(func) = &func.program().functions[0] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["print".into(), "hello, world!".into()];
    let expected_bytecodes = &[
        // local function test_local_env()
        //     local _ENV = { print = my_print }
        Bytecode::new_table(0, 1, 0),
        Bytecode::get_upvalue(1, 0),
        Bytecode::set_field(0, 0, 1, 0),
        //     print "hello, world!" -- this `print` is my_print
        Bytecode::get_field(1, 0, 0),
        Bytecode::load_constant(2, 1),
        Bytecode::call(1, 2, 1),
        // end
        Bytecode::zero_return(),
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    let Value::Function(func) = &program.functions[2] else {
        unreachable!("function must be a `Value::Closure`");
    };
    let expected_constants: &[Value] = &["g2".into()];
    let expected_bytecodes = &[
        // local inner = function()
        //     -- assign to upvalues
        //     up1, up2, up3 = 101, g2, up4
        Bytecode::load_integer(0, 101),
        Bytecode::get_uptable(1, 3, 0),
        Bytecode::get_upvalue(2, 4),
        Bytecode::set_upvalue(2, 2),
        //     print(up1, up2, up3)
        // end
    ];
    assert_eq!(func.program().constants, expected_constants);
    assert_eq!(func.program().byte_codes, expected_bytecodes);
    assert!(func.program().functions.is_empty());

    crate::Lua::execute(&program).expect("Should run");
}

// #[test]
// fn env() {
//     let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());

//     let program = Program::parse(
//         r#"
// local function my_print(a)
//     print("test _ENV:", a)
// end

// -- _ENV as local variable
// local function test_local_env()
//     local _ENV = { print = my_print }
//     print "hello, world!" -- this `print` is my_print
// end

// test_local_env()

// -- _ENV as upvalue
// local _ENV = { print = my_print }
// local function test_upvalue_env()
//     print "hello, upvalue!" -- this `print` is my_print
// end

// test_upvalue_env()
// "#,
//     )
//     .unwrap();

//     let expected_bytecodes = &[
//         Bytecode::variadic_arguments_prepare(0),
//         // local function my_print(a)
//         Bytecode::closure(0, 0),
//         // local function test_local_env()
//         Bytecode::closure(1, 1),
//         // test_local_env()
//         Bytecode::move_bytecode(2, 1),
//         Bytecode::call(2, 1, 1),
//         // local _ENV = { print = my_print }
//         Bytecode::new_table(2, 1, 0),
//         Bytecode::set_field(2, 0, 0, 0),
//         // local function test_upvalue_env()
//         Bytecode::closure(3, 2),
//         Bytecode::move_bytecode(4, 3),
//         // test_upvalue_env()
//         Bytecode::call(4, 1, 1),
//         // EOF
//         Bytecode::return_bytecode(4, 1, 1),
//     ];
//     assert_eq!(program.constants, &["print".into()]);
//     assert_eq!(&program.byte_codes, expected_bytecodes);
//     assert_eq!(program.functions.len(), 3);

//     let Value::Function(func) = &program.functions[0] else {
//         unreachable!("function must be a `Value::Closure`");
//     };
//     let expected_constants: &[Value] = &["print".into(), "test _ENV:".into()];
//     let expected_bytecodes = &[
//         // local function my_print(a)
//         //     print("test _ENV:", a)
//         Bytecode::get_uptable(1, 0, 0),
//         Bytecode::load_constant(2, 1),
//         Bytecode::move_bytecode(3, 0),
//         Bytecode::call(1, 3, 1),
//         // end
//         Bytecode::zero_return(),
//     ];
//     assert_eq!(func.program().constants, expected_constants);
//     assert_eq!(func.program().byte_codes, expected_bytecodes);
//     assert!(func.program().functions.is_empty());

//     let Value::Function(func) = &program.functions[1] else {
//         unreachable!("function must be a `Value::Closure`");
//     };
//     let expected_constants: &[Value] = &["print".into(), "hello, world!".into()];
//     let expected_bytecodes = &[
//         // local function test_local_env()
//         //     local _ENV = { print = my_print }
//         Bytecode::new_table(0, 1, 0),
//         Bytecode::get_upvalue(1, 0),
//         Bytecode::set_field(0, 0, 1, 0),
//         //     print "hello, world!" -- this `print` is my_print
//         Bytecode::get_field(1, 0, 0),
//         Bytecode::load_constant(2, 1),
//         Bytecode::call(1, 2, 1),
//         // end
//         Bytecode::zero_return(),
//     ];
//     assert_eq!(func.program().constants, expected_constants);
//     assert_eq!(func.program().byte_codes, expected_bytecodes);
//     assert!(func.program().functions.is_empty());

//     let Value::Function(func) = &program.functions[2] else {
//         unreachable!("function must be a `Value::Closure`");
//     };
//     let expected_constants: &[Value] = &["print".into(), "hello, world!".into()];
//     let expected_bytecodes = &[
//         // local function test_upvalue_env()
//         //     print "hello, upvalue!" -- this `print` is my_print
//         Bytecode::get_uptable(0, 0, 0),
//         Bytecode::load_constant(1, 1),
//         Bytecode::call(0, 2, 1),
//         // end
//         Bytecode::zero_return(),
//     ];
//     assert_eq!(func.program().constants, expected_constants);
//     assert_eq!(func.program().byte_codes, expected_bytecodes);
//     assert!(func.program().functions.is_empty());

//     crate::Lua::execute(&program).expect("Should run");
// }
