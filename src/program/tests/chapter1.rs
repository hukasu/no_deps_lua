use crate::bytecode::Bytecode;

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
            Bytecode::variadic_arguments_prepare(0),
            // print "hello, world!"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // print "hello, again!"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 2),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ]
    );
    crate::Lua::execute(&program).unwrap();
}
