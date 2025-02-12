use crate::{byte_code::ByteCode, value::Value, Program};

#[test]
fn and_or() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
g1 = 1
g2 = 2

if g1 or g2 and g3 then
    print "test only once"
end

if g3 or g1 and g2 then
    print "test 3 times"
end

if (g3 or g1) and (g2 or g4) then
    print "test 3 times"
end

if (g3 or g1) and (g2 and g4) then
    print "test 4 times and fail"
end
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "g1".into(),
            "g2".into(),
            "g3".into(),
            "print".into(),
            "test only once".into(),
            "test 3 times".into(),
            "g4".into(),
            "test 4 times and fail".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // g1 = 1
            ByteCode::SetGlobalInteger(0, 1),
            // g2 = 2
            ByteCode::SetGlobalInteger(1, 2),
            // if g1 or g2 and g3 then
            ByteCode::GetGlobal(0, 0),
            ByteCode::Test(0, 1),
            ByteCode::Jmp(6),
            ByteCode::GetGlobal(0, 1),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(6),
            ByteCode::GetGlobal(0, 2),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(3),
            //     print "test only once"
            ByteCode::GetGlobal(0, 3),
            ByteCode::LoadConstant(1, 4),
            ByteCode::Call(0, 1),
            // end

            // if g3 or g1 and g2 then
            ByteCode::GetGlobal(0, 2),
            ByteCode::Test(0, 1),
            ByteCode::Jmp(6),
            ByteCode::GetGlobal(0, 0),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(6),
            ByteCode::GetGlobal(0, 1),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(3),
            //     print "test 3 times"
            ByteCode::GetGlobal(0, 3),
            ByteCode::LoadConstant(1, 5),
            ByteCode::Call(0, 1),
            // end

            // if (g3 or g1) and (g2 or g4) then
            ByteCode::GetGlobal(0, 2),
            ByteCode::Test(0, 1),
            ByteCode::Jmp(3),
            ByteCode::GetGlobal(0, 0),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(9),
            ByteCode::GetGlobal(0, 1),
            ByteCode::Test(0, 1),
            ByteCode::Jmp(3),
            ByteCode::GetGlobal(0, 6),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(3),
            //     print "test 3 times"
            ByteCode::GetGlobal(0, 3),
            ByteCode::LoadConstant(1, 5),
            ByteCode::Call(0, 1),
            // end

            // if (g3 or g1) and (g2 and g4) then
            ByteCode::GetGlobal(0, 2),
            ByteCode::Test(0, 1),
            ByteCode::Jmp(3),
            ByteCode::GetGlobal(0, 0),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(9),
            ByteCode::GetGlobal(0, 1),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(6),
            ByteCode::GetGlobal(0, 6),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(3),
            //     print "test 4 times and fail"
            ByteCode::GetGlobal(0, 3),
            ByteCode::LoadConstant(1, 7),
            ByteCode::Call(0, 1),
            // end
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn test_set() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
g1 = 1
g2 = 2

print( g1 or g2 and g3)
print( g3 or g1 and g2)
print( (g3 or g1) and (g2 or g4))
print( (g3 or g1) and (g2 and g4))
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "g1".into(),
            "g2".into(),
            "print".into(),
            "g3".into(),
            "g4".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // g1 = 1
            ByteCode::SetGlobalInteger(0, 1),
            // g2 = 2
            ByteCode::SetGlobalInteger(1, 2),
            // print( g1 or g2 and g3)
            ByteCode::GetGlobal(0, 2),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Test(1, 1),
            ByteCode::Jmp(4),
            ByteCode::GetGlobal(1, 1),
            ByteCode::Test(1, 0),
            ByteCode::Jmp(1),
            ByteCode::GetGlobal(1, 3),
            ByteCode::Call(0, 1),
            // print( g3 or g1 and g2)
            ByteCode::GetGlobal(0, 2),
            ByteCode::GetGlobal(1, 3),
            ByteCode::Test(1, 1),
            ByteCode::Jmp(4),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Test(1, 0),
            ByteCode::Jmp(1),
            ByteCode::GetGlobal(1, 1),
            ByteCode::Call(0, 1),
            // print( (g3 or g1) and (g2 or g4))
            ByteCode::GetGlobal(0, 2),
            ByteCode::GetGlobal(1, 3),
            ByteCode::Test(1, 1),
            ByteCode::Jmp(3),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Test(1, 0),
            ByteCode::Jmp(4),
            ByteCode::GetGlobal(1, 1),
            ByteCode::Test(1, 1),
            ByteCode::Jmp(1),
            ByteCode::GetGlobal(1, 4),
            ByteCode::Call(0, 1),
            // print( (g3 or g1) and (g2 and g4))
            ByteCode::GetGlobal(0, 2),
            ByteCode::GetGlobal(1, 3),
            ByteCode::Test(1, 1),
            ByteCode::Jmp(3),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Test(1, 0),
            ByteCode::Jmp(4),
            ByteCode::GetGlobal(1, 1),
            ByteCode::Test(1, 0),
            ByteCode::Jmp(1),
            ByteCode::GetGlobal(1, 4),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn compare() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());

    let code = r#"
local a, b = 123, "hello"
if a >= 123 and b == "hello" then
    print "yes"
end

if b <= "world" then
    print (a>100)
end

print (a == 1000 and b == "hello")
print (a<b)
print (a>b)
print (a<=b)
print (a>=b)
"#;
    let expected_constants: &[Value] = &[
        "hello".into(),
        "print".into(),
        "yes".into(),
        "world".into(),
        1000i64.into(),
    ];
    let expected_bytecodes = &[
        // local a, b = 123, "hello"
        ByteCode::LoadInt(0, 123),
        ByteCode::LoadConstant(1, 0),
        // if a >= 123 and b == "hello" then
        ByteCode::GreaterEqualInteger(0, 123, 0),
        ByteCode::Jmp(5),
        ByteCode::EqualConstant(1, 0, 0),
        ByteCode::Jmp(3),
        //     print "yes"
        ByteCode::GetGlobal(2, 1),
        ByteCode::LoadConstant(3, 2),
        ByteCode::Call(2, 1),
        // end

        // if b <= "world" then
        ByteCode::LoadConstant(2, 3),
        ByteCode::LessEqual(1, 2, 0),
        ByteCode::Jmp(6),
        //     print (a>100)
        ByteCode::GetGlobal(2, 1),
        ByteCode::GreaterThanInteger(0, 100, 1),
        ByteCode::Jmp(1),
        ByteCode::LoadFalseSkip(3),
        ByteCode::LoadTrue(3),
        ByteCode::Call(2, 1),
        // end

        // print (a == 1000 and b == "hello")
        ByteCode::GetGlobal(2, 1),
        ByteCode::EqualConstant(0, 4, 0),
        ByteCode::Jmp(2),
        ByteCode::EqualConstant(1, 0, 1),
        ByteCode::Jmp(1),
        ByteCode::LoadFalseSkip(3),
        ByteCode::LoadTrue(3),
        ByteCode::Call(2, 1),
        // print (a<b)
        ByteCode::GetGlobal(2, 1),
        ByteCode::LessThan(0, 1, 1),
        ByteCode::Jmp(1),
        ByteCode::LoadFalseSkip(3),
        ByteCode::LoadTrue(3),
        ByteCode::Call(2, 1),
        // print (a>b)
        ByteCode::GetGlobal(2, 1),
        ByteCode::LessThan(1, 0, 1),
        ByteCode::Jmp(1),
        ByteCode::LoadFalseSkip(3),
        ByteCode::LoadTrue(3),
        ByteCode::Call(2, 1),
        // print (a<=b)
        ByteCode::GetGlobal(2, 1),
        ByteCode::LessEqual(0, 1, 1),
        ByteCode::Jmp(1),
        ByteCode::LoadFalseSkip(3),
        ByteCode::LoadTrue(3),
        ByteCode::Call(2, 1),
        // print (a>=b)
        ByteCode::GetGlobal(2, 1),
        ByteCode::LessEqual(1, 0, 1),
        ByteCode::Jmp(1),
        ByteCode::LoadFalseSkip(3),
        ByteCode::LoadTrue(3),
        ByteCode::Call(2, 1),
    ];

    let program = Program::parse(code).unwrap();
    assert_eq!(&program.constants, expected_constants);
    assert_eq!(&program.byte_codes, expected_bytecodes);
    crate::Lua::execute(&program).expect_err("Comparison between string and integer should fail");
}
