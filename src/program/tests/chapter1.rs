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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print "hello, world!"
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print "hello, again!"
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 2u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "hello, world!".into(),
            "hello, again!".into(),
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}
