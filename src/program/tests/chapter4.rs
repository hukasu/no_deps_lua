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
            "x".into(),
            "hello".into(),
            "y".into(),
            "world".into(),
            "val".into(),
            "print".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            ByteCode::VariadicArgumentPrepare(0),
            // local key = "key"
            ByteCode::LoadConstant(0, 0),
            // local t = {...}
            ByteCode::NewTable(1, 3, 3),
            // 100, 200, 300;
            ByteCode::LoadInt(2, 100),
            ByteCode::LoadInt(3, 200),
            ByteCode::LoadInt(4, 300),
            // x="hello", y="world";
            ByteCode::SetFieldConstant(1, 1, 2),
            ByteCode::SetFieldConstant(1, 3, 4),
            // [key]="val";
            ByteCode::SetTableConstant(1, 0, 5),
            // {...}
            ByteCode::SetList(1, 3, 0),
            // print(t[1])
            ByteCode::GetUpTable(2, 0, 6),
            ByteCode::GetInt(3, 1, 1),
            ByteCode::Call(2, 2, 1),
            // print(t['x'])
            ByteCode::GetUpTable(2, 0, 6),
            ByteCode::GetField(3, 1, 1),
            ByteCode::Call(2, 2, 1),
            // print(t.key)
            ByteCode::GetUpTable(2, 0, 6),
            ByteCode::GetField(3, 1, 0),
            ByteCode::Call(2, 2, 1),
            // print(t)
            ByteCode::GetUpTable(2, 0, 6),
            ByteCode::Move(3, 1),
            ByteCode::Call(2, 2, 1),
            // EOF
            ByteCode::Return(2, 1, 1),
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
            300i64.into(),
            "z".into(),
            400i64.into(),
            "x".into(),
            "f".into(),
            "print".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            ByteCode::VariadicArgumentPrepare(0),
            // local a,b = 100,200
            ByteCode::LoadInt(0, 100),
            ByteCode::LoadInt(1, 200),
            // t = {...}
            ByteCode::NewTable(2, 2, 3),
            // k=300
            ByteCode::SetFieldConstant(2, 1, 2),
            // z=a
            ByteCode::SetField(2, 3, 0),
            // 10,20,30
            ByteCode::LoadInt(3, 10),
            ByteCode::LoadInt(4, 20),
            ByteCode::LoadInt(5, 30),
            ByteCode::SetList(2, 3, 0),
            // t = {...}
            ByteCode::SetUpTable(0, 0, 2),
            // t.k = 400 -- set
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::SetFieldConstant(2, 1, 4),
            // t.x = t.z -- new
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::GetUpTable(3, 0, 0),
            ByteCode::GetField(3, 3, 3),
            ByteCode::SetField(2, 5, 3),
            // t.f = print -- new
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::GetUpTable(3, 0, 7),
            ByteCode::SetField(2, 6, 3),
            // t.f(t.k)
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::GetField(2, 2, 6),
            ByteCode::GetUpTable(3, 0, 0),
            ByteCode::GetField(3, 3, 1),
            ByteCode::Call(2, 2, 1),
            // t.f(t.x)
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::GetField(2, 2, 6),
            ByteCode::GetUpTable(3, 0, 0),
            ByteCode::GetField(3, 3, 5),
            ByteCode::Call(2, 2, 1),
            // t.f(t[2])
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::GetField(2, 2, 6),
            ByteCode::GetUpTable(3, 0, 0),
            ByteCode::GetInt(3, 3, 2),
            ByteCode::Call(2, 2, 1),
            // t.f(t[1000])
            ByteCode::GetUpTable(2, 0, 0),
            ByteCode::GetField(2, 2, 6),
            ByteCode::GetUpTable(3, 0, 0),
            ByteCode::LoadInt(4, 1000),
            ByteCode::GetTable(3, 3, 4),
            ByteCode::Call(2, 2, 1),
            // EOF
            ByteCode::Return(2, 1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
