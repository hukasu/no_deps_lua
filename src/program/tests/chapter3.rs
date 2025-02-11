use alloc::string::String;

use crate::{byte_code::ByteCode, ext::Unescape, Program};

#[test]
fn chapter3_escape() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
print "tab:\thi" -- tab
print "\xE4\xBD\xA0\xE5\xA5\xBD" -- 你好
print "\xE4\xBD" -- invalid UTF-8
print "\72\101\108\108\111" -- Hello
print "null: \0." -- '\0'
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "tab:\thi".into(),
            String::from_utf8_lossy(b"\xE4\xBD\xA0\xE5\xA5\xBD")
                .as_ref()
                .unescape()
                .unwrap()
                .as_str()
                .into(),
            String::from_utf8_lossy(b"\xE4\xBD")
                .as_ref()
                .unescape()
                .unwrap()
                .as_str()
                .into(),
            "Hello".into(),
            "null: \0.".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print "tab:\thi" -- tab
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            // print "\xE4\xBD\xA0\xE5\xA5\xBD" -- 你好
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
            // print "\xE4\xBD" -- invalid UTF-8
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 3),
            ByteCode::Call(0, 1),
            // print "\72\101\108\108\111" -- Hello
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 4),
            ByteCode::Call(0, 1),
            // print "null: \0." -- '\0'
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 5),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter3_strings() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local s = "hello_world"
local m = "middle_string_middle_string"
local l = "long_string_long_string_long_string_long_string_long_string"
print(s)
print(m)
print(l)

hello_world = 12
middle_string_middle_string = 345
long_string_long_string_long_string_long_string_long_string = 6789
print(hello_world)
print(middle_string_middle_string)
print(long_string_long_string_long_string_long_string_long_string)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "hello_world".into(),
            "middle_string_middle_string".into(),
            "long_string_long_string_long_string_long_string_long_string".into(),
            "print".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local s = "hello_world"
            ByteCode::LoadConstant(0, 0),
            // local m = "middle_string_middle_string"
            ByteCode::LoadConstant(1, 1),
            // local l = "long_string_long_string_long_string_long_string_long_string"
            ByteCode::LoadConstant(2, 2),
            // print(s)
            ByteCode::GetGlobal(3, 3),
            ByteCode::Move(4, 0),
            ByteCode::Call(3, 1),
            // print(m)
            ByteCode::GetGlobal(3, 3),
            ByteCode::Move(4, 1),
            ByteCode::Call(3, 1),
            // print(l)
            ByteCode::GetGlobal(3, 3),
            ByteCode::Move(4, 2),
            ByteCode::Call(3, 1),
            // hello_world = 12
            ByteCode::SetGlobalInteger(0, 12),
            // middle_string_middle_string = 345
            ByteCode::SetGlobalInteger(1, 345),
            // long_string_long_string_long_string_long_string_long_string = 6789
            ByteCode::SetGlobalInteger(2, 6789),
            // print(hello_world)
            ByteCode::GetGlobal(3, 3),
            ByteCode::GetGlobal(4, 0),
            ByteCode::Call(3, 1),
            // print(middle_string_middle_string)
            ByteCode::GetGlobal(3, 3),
            ByteCode::GetGlobal(4, 1),
            ByteCode::Call(3, 1),
            // print(long_string_long_string_long_string_long_string_long_string)
            ByteCode::GetGlobal(3, 3),
            ByteCode::GetGlobal(4, 2),
            ByteCode::Call(3, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
