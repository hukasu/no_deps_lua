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
            Bytecode::variadic_arguments_prepare(0),
            // print "hello, world!"
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1u8),
            Bytecode::call(0, 2, 1),
            // warn "hello, warn!"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3u8),
            Bytecode::call(0, 2, 1),
            // warn "@on"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 4u8),
            Bytecode::call(0, 2, 1),
            // warn "hello, warn!"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3u8),
            Bytecode::call(0, 2, 1),
            // warn "@off"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 5u8),
            Bytecode::call(0, 2, 1),
            // warn "hello, warn!"
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::load_constant(1, 3u8),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
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
