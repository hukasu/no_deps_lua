use crate::{Program, bytecode::Bytecode, program::Local};

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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local key = "key"
            Bytecode::load_constant(0.into(), 0u8.into()),
            // local t = {...}
            Bytecode::new_table(1.into(), 3.into(), 3.into()),
            // TODO EXTRAARG
            // 100, 200, 300;
            Bytecode::load_integer(2.into(), 100i16.into()),
            Bytecode::load_integer(3.into(), 200i16.into()),
            Bytecode::load_integer(4.into(), 300i16.into()),
            // x="hello", y="world";
            Bytecode::set_field(1.into(), 1.into(), 2.into(), true.into()),
            Bytecode::set_field(1.into(), 3.into(), 4.into(), true.into()),
            // [key]="val";
            Bytecode::set_table(1.into(), 0.into(), 5.into(), true.into()),
            // {...}
            Bytecode::set_list(1.into(), 3.into(), 0.into()),
            // print(t[1])
            Bytecode::get_uptable(2.into(), 0.into(), 6.into()),
            Bytecode::get_index(3.into(), 1.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(t['x'])
            Bytecode::get_uptable(2.into(), 0.into(), 6.into()),
            Bytecode::get_field(3.into(), 1.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(t.key)
            Bytecode::get_uptable(2.into(), 0.into(), 6.into()),
            Bytecode::get_field(3.into(), 1.into(), 0.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print(t)
            Bytecode::get_uptable(2.into(), 0.into(), 6.into()),
            Bytecode::move_bytecode(3.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(2.into(), 1.into(), 1.into()),
        ],
        &[
            "key".into(),
            "x".into(),
            "hello".into(),
            "y".into(),
            "world".into(),
            "val".into(),
            "print".into(),
        ],
        &[
            // TODO update when implementing `EXTRAARG` bytecode
            Local::new("k".into(), 3, 24),
            Local::new("t".into(), 11, 24),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
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
    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a,b = 100,200
            Bytecode::load_integer(0.into(), 100i16.into()),
            Bytecode::load_integer(1.into(), 200i16.into()),
            // t = {...}
            Bytecode::new_table(2.into(), 2.into(), 3.into()),
            // TODO EXTRAARG
            // k=300
            Bytecode::set_field(2.into(), 1.into(), 2.into(), true.into()),
            // z=a
            Bytecode::set_field(2.into(), 3.into(), 0.into(), false.into()),
            // 10,20,30
            Bytecode::load_integer(3.into(), 10i8.into()),
            Bytecode::load_integer(4.into(), 20i8.into()),
            Bytecode::load_integer(5.into(), 30i8.into()),
            Bytecode::set_list(2.into(), 3.into(), 0.into()),
            // t = {...}
            Bytecode::set_uptable(0.into(), 0.into(), 2.into(), false.into()),
            // t.k = 400 -- set
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::set_field(2.into(), 1.into(), 4.into(), true.into()),
            // t.x = t.z -- new
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_field(3.into(), 3.into(), 3.into()),
            Bytecode::set_field(2.into(), 5.into(), 3.into(), false.into()),
            // t.f = print -- new
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 7.into()),
            Bytecode::set_field(2.into(), 6.into(), 3.into(), false.into()),
            // t.f(t.k)
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::get_field(2.into(), 2.into(), 6.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_field(3.into(), 3.into(), 1.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // t.f(t.x)
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::get_field(2.into(), 2.into(), 6.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_field(3.into(), 3.into(), 5.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // t.f(t[2])
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::get_field(2.into(), 2.into(), 6.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_index(3.into(), 3.into(), 2.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // t.f(t[1000])
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::get_field(2.into(), 2.into(), 6.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::load_integer(4.into(), 1000i16.into()),
            Bytecode::get_table(3.into(), 3.into(), 4.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(2.into(), 1.into(), 1.into()),
        ],
        &[
            "t".into(),
            "k".into(),
            300i64.into(),
            "z".into(),
            400i64.into(),
            "x".into(),
            "f".into(),
            "print".into(),
        ],
        &[
            // TODO update when implementing EXTRAARG
            Local::new("a".into(), 4, 43),
            Local::new("b".into(), 4, 43),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}
