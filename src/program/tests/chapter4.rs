use crate::{byte_code::ByteCode, Program};

#[test]
fn table() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
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
fn prefixexp() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
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
