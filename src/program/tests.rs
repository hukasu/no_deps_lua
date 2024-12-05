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
fn chapter2_types() {
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
            ByteCode::LoadFalse(1),
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
fn chapter2_local2() {
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
fn chapter2_assign() {
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

#[test]
fn chapter4_table() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local k = "key"
local t = {
    100, 200, 300;  -- list style
    x="hello", y="world";  -- record style
    [k]="val";  -- general style
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
            ByteCode::GetField(3, 1, 0),
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
            "x".into(),
            "print".into(),
            "f".into(),
            1000i64.into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a,b = 100,200
            ByteCode::LoadInt(0, 100),
            ByteCode::LoadInt(1, 200),
            // t = {...}
            ByteCode::NewTable(2, 3, 2),
            // k=300
            ByteCode::LoadInt(3, 300),
            ByteCode::SetField(2, 1, 3),
            // z=a
            ByteCode::Move(3, 0),
            ByteCode::SetField(2, 2, 3),
            // 10,20,30
            ByteCode::LoadInt(3, 10),
            ByteCode::LoadInt(4, 20),
            ByteCode::LoadInt(5, 30),
            ByteCode::SetList(2, 3),
            // t = {...}
            ByteCode::SetGlobal(0, 2),
            // t.k = 400 -- set
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadInt(3, 400),
            ByteCode::SetField(2, 1, 3),
            // t.x = t.z -- new
            ByteCode::GetGlobal(2, 0),
            ByteCode::GetField(2, 2, 2),
            ByteCode::GetGlobal(3, 0),
            ByteCode::SetField(3, 3, 2),
            // t.f = print -- new
            ByteCode::GetGlobal(2, 4),
            ByteCode::GetGlobal(3, 0),
            ByteCode::SetField(3, 5, 2),
            // t.f(t.k)
            ByteCode::GetGlobal(2, 0),
            ByteCode::GetField(2, 2, 5),
            ByteCode::GetGlobal(3, 0),
            ByteCode::GetField(3, 3, 1),
            ByteCode::Call(2, 1),
            // t.f(t.x)
            ByteCode::GetGlobal(2, 0),
            ByteCode::GetField(2, 2, 5),
            ByteCode::GetGlobal(3, 0),
            ByteCode::GetField(3, 3, 3),
            ByteCode::Call(2, 1),
            // t.f(t[2])
            ByteCode::GetGlobal(2, 0),
            ByteCode::GetField(2, 2, 5),
            ByteCode::GetGlobal(3, 0),
            ByteCode::GetInt(3, 3, 2),
            ByteCode::Call(2, 1),
            // t.f(t[1000])
            ByteCode::GetGlobal(2, 0),
            ByteCode::GetField(2, 2, 5),
            ByteCode::GetGlobal(3, 0),
            ByteCode::GetField(3, 3, 6),
            ByteCode::Call(2, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter5_unops() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local i = 100
local f = 3.14
a = "iamastring"
print(~100)
print(~i)
print(-3.14)
print(-f)
print(#"iamastring")
print(#a)
print(not false)
print(not nil)
print(not not nil)
print(not print)
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "a".into(),
            "iamastring".into(),
            "print".into(),
            #[allow(clippy::approx_constant)]
            (-3.14f64).into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local i = 100
            ByteCode::LoadInt(0, 100),
            // local f = 3.14
            ByteCode::LoadConstant(1, 0),
            // a = "iamastring"
            ByteCode::SetGlobalConstant(1, 2),
            // print(~100)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadInt(3, !100),
            ByteCode::Call(2, 1),
            // print(~i)
            ByteCode::GetGlobal(2, 3),
            ByteCode::BitNot(3, 0),
            ByteCode::Call(2, 1),
            // print(-3.14)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadConstant(3, 4),
            ByteCode::Call(2, 1),
            // print(-f)
            ByteCode::GetGlobal(2, 3),
            ByteCode::Neg(3, 1),
            ByteCode::Call(2, 1),
            // print(#"iamastring")
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadInt(3, 10),
            ByteCode::Call(2, 1),
            // print(#a)
            ByteCode::GetGlobal(2, 3),
            ByteCode::GetGlobal(3, 1),
            ByteCode::Len(3, 3),
            ByteCode::Call(2, 1),
            // print(not false)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadTrue(3),
            ByteCode::Call(2, 1),
            // print(not nil)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadTrue(3),
            ByteCode::Call(2, 1),
            // print(not not nil)
            ByteCode::GetGlobal(2, 3),
            ByteCode::LoadFalse(3),
            ByteCode::Call(2, 1),
            // print(not print)
            ByteCode::GetGlobal(2, 3),
            ByteCode::GetGlobal(3, 3),
            ByteCode::Not(3, 3),
            ByteCode::Call(2, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}

#[test]
fn chapter5_binops() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
g = 10
local a,b,c = 1.1, 2.0, 100

print(100+g) -- commutative, AddInt
print(a-1)
print(100/c) -- result is float
print(100>>b) -- 2.0 will be convert to int 2
print(100>>a) -- panic
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &["g".into(), 1.1f64.into(), "print".into()]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // g = 10
            ByteCode::SetGlobalInteger(0, 10),
            // local a,b,c = 1.1, 2.0, 100
            ByteCode::LoadConstant(0, 1),
            ByteCode::LoadFloat(1, 2),
            ByteCode::LoadInt(2, 100),
            // print(100+g) -- commutative, AddInt
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::GetGlobal(5, 0),
            ByteCode::Add(4, 4, 5),
            ByteCode::Call(3, 1),
            // print(a-1)
            ByteCode::GetGlobal(3, 2),
            ByteCode::Move(4, 0),
            ByteCode::LoadInt(5, 1),
            ByteCode::Sub(4, 4, 5),
            ByteCode::Call(3, 1),
            // print(100/c) -- result is float
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::Move(5, 2),
            ByteCode::Div(4, 4, 5),
            ByteCode::Call(3, 1),
            // print(100>>b) -- 2.0 will be convert to int 2
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::Move(5, 1),
            ByteCode::ShiftR(4, 4, 5),
            ByteCode::Call(3, 1),
            // print(100>>a) -- panic
            ByteCode::GetGlobal(3, 2),
            ByteCode::LoadInt(4, 100),
            ByteCode::Move(5, 0),
            ByteCode::ShiftR(4, 4, 5),
            ByteCode::Call(3, 1),
        ]
    );
    crate::Lua::execute(&program)
        .inspect_err(|err| log::error!("{err}"))
        .expect_err("Last print should fail");
}

#[test]
fn chapter5_concat() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
print('hello, '..'world')
print('hello, ' .. 123)
print(3.14 .. 15926)
local a = true
print('hello' .. a) -- panic
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "hello, ".into(),
            "world".into(),
            #[allow(clippy::approx_constant)]
            3.14f64.into(),
            "hello".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print('hello, '..'world')
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadConstant(2, 2),
            ByteCode::Concat(1, 1, 2),
            ByteCode::Call(0, 1),
            // print('hello, ' .. 123)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadInt(2, 123),
            ByteCode::Concat(1, 1, 2),
            ByteCode::Call(0, 1),
            // print(3.14 .. 15926)
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 3),
            ByteCode::LoadInt(2, 15926),
            ByteCode::Concat(1, 1, 2),
            ByteCode::Call(0, 1),
            // print('hello' .. true) -- panic
            ByteCode::LoadTrue(0),
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 4),
            ByteCode::Move(3, 0),
            ByteCode::Concat(2, 2, 3),
            ByteCode::Call(1, 1),
        ]
    );
    crate::Lua::execute(&program)
        .inspect_err(|err| log::error!("{err}"))
        .expect_err("Last print should fail");
}

#[test]
fn chapter6_if() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
if a then
    print "skip this"
end
if print then
    local a = "I am true"
    print(a)
end

print (a) -- should be nil
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "a".into(),
            "print".into(),
            "skip this".into(),
            "I am true".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // if a then
            ByteCode::GetGlobal(0, 0),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(3),
            // print "skip this"
            ByteCode::GetGlobal(0, 1),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
            // end
            // if print then
            ByteCode::GetGlobal(0, 1),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(4),
            // local a = "I am true"
            ByteCode::LoadConstant(0, 3),
            // print(a)
            ByteCode::GetGlobal(1, 1),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // end
            // print (a) -- should be nil
            ByteCode::GetGlobal(0, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_if_else() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a,b = 123
if b then
  print "not here"
elseif g then
  print "not here"
elseif a then
  print "yes, here"
else
  print "not here"
end

if b then
  print "not here"
else
  print "yes, here"
end

if b then
  print "yes, here"
end
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "not here".into(),
            "g".into(),
            "yes, here".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a,b = 123
            ByteCode::LoadInt(0, 123),
            ByteCode::LoadNil(1),
            // if b then
            ByteCode::Test(1, 0),
            ByteCode::Jmp(4),
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(16),
            // elseif g then
            ByteCode::GetGlobal(2, 2),
            ByteCode::Test(2, 0),
            ByteCode::Jmp(4),
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(9),
            // elseif a then
            ByteCode::Test(0, 0),
            ByteCode::Jmp(4),
            //   print "yes, here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 3),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(3),
            // else
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            // end
            // if b then
            ByteCode::Test(1, 0),
            ByteCode::Jmp(4),
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(3),
            // else
            //   print "yes, here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 3),
            ByteCode::Call(2, 1),
            // end
            // if b then
            ByteCode::Test(1, 0),
            ByteCode::Jmp(3),
            //   print "yes, here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 3),
            ByteCode::Call(2, 1),
            // end
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_while() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a = 123
while a do
  print(a)
  a = not a
end
"#,
    )
    .unwrap();
    assert_eq!(&program.constants, &["print".into(),]);
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = 123
            ByteCode::LoadInt(0, 123),
            // while a do
            ByteCode::Test(0, 0),
            ByteCode::Jmp(5),
            //   print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            //   a = not a
            ByteCode::Not(0, 0),
            // end
            ByteCode::Jmp(-7),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_break() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local z = 1
while z do
  while z do
    print "break inner"
    break
    print "unreachable inner"
  end

  print "break outer"
  break
  print "unreachable outer"
end
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "break inner".into(),
            "unreachable inner".into(),
            "break outer".into(),
            "unreachable outer".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local z = 1
            ByteCode::LoadInt(0, 1),
            // while z do
            ByteCode::Test(0, 0),
            ByteCode::Jmp(18),
            //   while z do
            ByteCode::Test(0, 0),
            ByteCode::Jmp(8),
            //     print "break inner"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 1),
            ByteCode::Call(1, 1),
            //     break
            ByteCode::Jmp(4),
            //     print "unreachable inner"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 2),
            ByteCode::Call(1, 1),
            //   end
            ByteCode::Jmp(-10),
            //   print "break outer"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 3),
            ByteCode::Call(1, 1),
            //   break
            ByteCode::Jmp(4),
            //   print "unreachable outer"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 4),
            ByteCode::Call(1, 1),
            // end
            ByteCode::Jmp(-20),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_repeat() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
local a = false
repeat
  print(a)
  a = not a
until a
"#,
    )
    .unwrap();
    assert_eq!(&program.constants, &["print".into(),]);
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = false
            ByteCode::LoadFalse(0),
            // repeat
            //   print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            //   a = not a
            ByteCode::Not(0, 0),
            // until a
            ByteCode::Test(0, 0),
            ByteCode::Jmp(-6),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_for() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
-- 1~3
for i = 1, 3, 1 do
    print(i)
end

-- negetive step, 1~-2
for i = 1, -2, -1 do
    print(i)
end

-- float limit, 1~3
for i = 1, 3.2 do
    print(i)
end

-- float i, 1.0~3.0
for i = 1.0, 3 do
    print(i)
end

-- special case, should not run
local max = 9223372036854775807
for i = max, max*10.0, -1 do
    print (i)
end
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            3.2f64.into(),
            9223372036854775807i64.into(),
            10.0f64.into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // for i = 1, 3, 1 do
            ByteCode::LoadInt(0, 1),
            ByteCode::LoadInt(1, 3),
            ByteCode::LoadInt(2, 1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // for i = 1, -2, -1 do
            ByteCode::LoadInt(0, 1),
            ByteCode::LoadInt(1, -2),
            ByteCode::LoadInt(2, -1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // for i = 1, 3.2 do
            ByteCode::LoadInt(0, 1),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadInt(2, 1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // for i = 1.0, 3 do
            ByteCode::LoadFloat(0, 1),
            ByteCode::LoadInt(1, 3),
            ByteCode::LoadInt(2, 1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // local max = 9223372036854775807
            ByteCode::LoadConstant(0, 2),
            // for i = max, max*10.0, -1 do
            ByteCode::Move(1, 0),
            ByteCode::MulConstant(2, 0, 3),
            ByteCode::LoadInt(3, -1),
            ByteCode::ForPrepare(1, 3),
            //     print (i)
            ByteCode::GetGlobal(5, 0),
            ByteCode::Move(6, 4),
            ByteCode::Call(5, 1),
            // end
            ByteCode::ForLoop(1, 4),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_goto() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
    let program = Program::parse(
        r#"
::label1::
print("block: 1")
goto label2
::label3::
print("block: 3")
goto label4
::label2::
do
  print("block: 2")
  goto label3 -- goto outer block
end
::label4::
print("block: 4")


do
  goto label
  local a = 'local var'
  ::label:: -- skip the local var but it's OK
  ::another:: -- void statment
  ;; -- void statment
  ::another2:: -- void statment
end
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "block: 1".into(),
            "block: 3".into(),
            "block: 2".into(),
            "block: 4".into(),
            "local var".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print("block: 1")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            // goto label2
            ByteCode::Jmp(4),
            // print("block: 3")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
            // goto label4
            ByteCode::Jmp(4),
            // do
            //   print("block: 2")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 3),
            ByteCode::Call(0, 1),
            //   goto label3 -- goto outer block
            ByteCode::Jmp(-8),
            // end
            // print("block: 4")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 4),
            ByteCode::Call(0, 1),
            // do
            //   goto label
            ByteCode::Jmp(1),
            //   local a = 'local var'
            ByteCode::LoadConstant(0, 5),
            // end
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}
