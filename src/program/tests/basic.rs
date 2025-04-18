use crate::bytecode::Bytecode;

#[test]
fn print_and_warn() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = crate::Program::parse(
        r#"
print "hello, world!"
warn "hello, warn!"
warn "@on"
warn "hello, warn!"
warn "@off"
warn "hello, warn!"
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
            // warn "hello, warn!"
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // warn "@on"
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::load_constant(1.into(), 4u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // warn "hello, warn!"
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // warn "@off"
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::load_constant(1.into(), 5u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // warn "hello, warn!"
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "hello, world!".into(),
            "warn".into(),
            "hello, warn!".into(),
            "@on".into(),
            "@off".into(),
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}
