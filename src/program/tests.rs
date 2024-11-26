use alloc::string::String;

use crate::ext::Unescape;

use super::*;

#[test]
fn chapter1() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
print "hello world"
print "hello again..."
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "hello world".into(),
            "hello again...".into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter1b() {
    let err = Program::parse(
        r#"
print "hello world"
print "hello again...
"#,
    )
    .expect_err("This program should fail");
    assert_eq!(err, Error::Parse);
}

#[test]
fn chapter2_1() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
print(nil)
print(false)
print(123)
print(123456)
print(123456.0)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &["print".into(), 123456i64.into(), 123456.0.into(),]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print(nil)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadNil(1),
            ByteCode::Call(0, 1),
            // print(false)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadBool(1, false),
            ByteCode::Call(0, 1),
            // print(123)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadInt(1, 123),
            ByteCode::Call(0, 1),
            // print(123456)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            // print(123456.0)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter2_2() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a = "hello, local!" -- define a local by string
local b = a -- define a local by another local
print(b) -- print local variable
print(print) -- print global variable
local print = print --define a local by global variable with same name
print "I'm local-print!" -- call local function
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "hello, local!".into(),
            "print".into(),
            "I'm local-print!".into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = "hello, local!"
            ByteCode::LoadConstant(0, 0),
            // local b = a
            ByteCode::Move(1, 0),
            // print(b)
            ByteCode::GetGlobal(2, 1),
            ByteCode::Move(3, 1),
            ByteCode::Call(2, 1),
            // print(print)
            ByteCode::GetGlobal(2, 1),
            ByteCode::GetGlobal(3, 1),
            ByteCode::Call(2, 1),
            // local print = print
            ByteCode::GetGlobal(2, 1),
            // print "I'm local-print!"
            ByteCode::Move(3, 2),
            ByteCode::LoadConstant(4, 2),
            ByteCode::Call(3, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter2_3() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a = 456
a = 123
print(a)
a = a
print(a)
a = g
print(a)
g = 123
print(g)
g = a
print(g)
g = g2
print(g)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &["print".into(), "g".into(), "g2".into(),]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = 456
            ByteCode::LoadInt(0, 456),
            // a = 123
            ByteCode::LoadInt(0, 123),
            // print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // a = a
            // print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // a = g
            ByteCode::GetGlobal(0, 1),
            // print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // g = 123
            ByteCode::SetGlobalInteger(1, 123),
            // print(g)
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
            // g = a
            ByteCode::SetGlobal(1, 0),
            // print(g)
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
            // g = g2
            ByteCode::SetGlobalGlobal(1, 2),
            // print(g)
            ByteCode::GetGlobal(1, 0),
            ByteCode::GetGlobal(2, 1),
            ByteCode::Call(1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter3_4() {
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
fn chapter4_2() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local key = "key"
print {
    100, 200, 300; -- list style
    x="hello", y="world"; -- record style
    [key]="val"; -- general style
}
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "key".into(),
            "print".into(),
            "hello".into(),
            "x".into(),
            "world".into(),
            "y".into(),
            "val".into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local key = "key"
            ByteCode::LoadConstant(0, 0),
            // print {...}
            ByteCode::GetGlobal(1, 1),
            // {...}
            ByteCode::NewTable(2, 3, 3),
            // 100, 200, 300;
            ByteCode::LoadInt(3, 100),
            ByteCode::LoadInt(4, 200),
            ByteCode::LoadInt(5, 300),
            // x="hello", y="world";
            ByteCode::LoadConstant(6, 2),
            ByteCode::SetField(2, 3, 6),
            ByteCode::LoadConstant(6, 4),
            ByteCode::SetField(2, 5, 6),
            // [key]="val";
            ByteCode::Move(6, 0),
            ByteCode::LoadConstant(7, 6),
            ByteCode::SetTable(2, 6, 7),
            // {...}
            ByteCode::SetList(2, 3),
            // print {...}
            ByteCode::Call(1, 1)
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter4_table() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local key = "key"
local t = {
    100, 200, 300;  -- list style
    x="hello", y="world";  -- record style
    [key]="val";  -- general style
}
print(t[1])
print(t['x'])
print(t.key)
print(t)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "key".into(),
            "hello".into(),
            "x".into(),
            "world".into(),
            "y".into(),
            "val".into(),
            "print".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local key = "key"
            ByteCode::LoadConstant(0, 0),
            // local t = {...}
            ByteCode::NewTable(1, 3, 3),
            // 100, 200, 300;
            ByteCode::LoadInt(2, 100),
            ByteCode::LoadInt(3, 200),
            ByteCode::LoadInt(4, 300),
            // x="hello", y="world";
            ByteCode::LoadConstant(5, 1),
            ByteCode::SetField(1, 2, 5),
            ByteCode::LoadConstant(5, 3),
            ByteCode::SetField(1, 4, 5),
            // [key]="val";
            ByteCode::Move(5, 0),
            ByteCode::LoadConstant(6, 5),
            ByteCode::SetTable(1, 5, 6),
            // {...}
            ByteCode::SetList(1, 3),
            // print(t[1])
            ByteCode::GetGlobal(2, 6),
            ByteCode::GetInt(3, 1, 1),
            ByteCode::Call(2, 1),
            // print(t['x'])
            ByteCode::GetGlobal(2, 6),
            ByteCode::GetField(3, 1, 2),
            ByteCode::Call(2, 1),
            // print(t.key)
            ByteCode::GetGlobal(2, 6),
            ByteCode::LoadConstant(4, 0),
            ByteCode::GetTable(3, 1, 4),
            ByteCode::Call(2, 1),
            // print(t)
            ByteCode::GetGlobal(2, 6),
            ByteCode::Move(3, 1),
            ByteCode::Call(2, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter4_prefixexp() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a,b = 100,200
t = {k=300, z=a, 10,20,30}
t.k = 400 -- set
t.x = t.z -- new
t.f = print -- new
t.f(t.k)
t.f(t.x)
t.f(t[2])
t.f(t[1000])
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "t".into(),
            "k".into(),
            "z".into(),
            "f".into(),
            "print".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local key = "key"
            ByteCode::LoadConstant(0, 0),
            // print {...}
            ByteCode::GetGlobal(1, 1),
            // {...}
            ByteCode::NewTable(2, 3, 3),
            // 100, 200, 300;
            ByteCode::LoadInt(3, 100),
            ByteCode::LoadInt(4, 200),
            ByteCode::LoadInt(5, 300),
            // x="hello", y="world";
            ByteCode::LoadConstant(6, 2),
            ByteCode::SetField(2, 3, 6),
            ByteCode::LoadConstant(6, 4),
            ByteCode::SetField(2, 5, 6),
            // [key]="val";
            ByteCode::Move(6, 0),
            ByteCode::LoadConstant(7, 6),
            ByteCode::SetTable(2, 6, 7),
            // {...}
            ByteCode::SetList(2, 3),
            // print {...}
            ByteCode::Call(1, 1)
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
