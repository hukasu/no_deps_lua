use crate::byte_code::ByteCode;

#[test]
fn hello_world() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    let program = crate::Program::parse(
        r#"
print "hello, world!"
print "hello, again!"
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "hello, world!".into(),
            "hello, again!".into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print "hello, world!"
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 2, 1),
            // print "hello, again!"
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 2, 1),
            // EOF
            ByteCode::Return(0, 1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
