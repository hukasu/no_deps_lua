use crate::{bytecode::Bytecode, program::Local, Program};

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
            Bytecode::variadic_arguments_prepare(0),
            // local key = "key"
            Bytecode::load_constant(0, 0),
            // local t = {...}
            Bytecode::new_table(1, 3, 3),
            // TODO EXTRAARG
            // 100, 200, 300;
            Bytecode::load_integer(2, 100),
            Bytecode::load_integer(3, 200),
            Bytecode::load_integer(4, 300),
            // x="hello", y="world";
            Bytecode::set_field(1, 1, 2, 1),
            Bytecode::set_field(1, 3, 4, 1),
            // [key]="val";
            Bytecode::set_table(1, 0, 5, 1),
            // {...}
            Bytecode::set_list(1, 3, 0),
            // print(t[1])
            Bytecode::get_uptable(2, 0, 6),
            Bytecode::get_index(3, 1, 1),
            Bytecode::call(2, 2, 1),
            // print(t['x'])
            Bytecode::get_uptable(2, 0, 6),
            Bytecode::get_field(3, 1, 1),
            Bytecode::call(2, 2, 1),
            // print(t.key)
            Bytecode::get_uptable(2, 0, 6),
            Bytecode::get_field(3, 1, 0),
            Bytecode::call(2, 2, 1),
            // print(t)
            Bytecode::get_uptable(2, 0, 6),
            Bytecode::move_bytecode(3, 1),
            Bytecode::call(2, 2, 1),
            // EOF
            Bytecode::return_bytecode(2, 1, 1),
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
            Bytecode::variadic_arguments_prepare(0),
            // local a,b = 100,200
            Bytecode::load_integer(0, 100),
            Bytecode::load_integer(1, 200),
            // t = {...}
            Bytecode::new_table(2, 2, 3),
            // TODO EXTRAARG
            // k=300
            Bytecode::set_field(2, 1, 2, 1),
            // z=a
            Bytecode::set_field(2, 3, 0, 0),
            // 10,20,30
            Bytecode::load_integer(3, 10),
            Bytecode::load_integer(4, 20),
            Bytecode::load_integer(5, 30),
            Bytecode::set_list(2, 3, 0),
            // t = {...}
            Bytecode::set_uptable(0, 0, 2, 0),
            // t.k = 400 -- set
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::set_field(2, 1, 4, 1),
            // t.x = t.z -- new
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_field(3, 3, 3),
            Bytecode::set_field(2, 5, 3, 0),
            // t.f = print -- new
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_uptable(3, 0, 7),
            Bytecode::set_field(2, 6, 3, 0),
            // t.f(t.k)
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_field(2, 2, 6),
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_field(3, 3, 1),
            Bytecode::call(2, 2, 1),
            // t.f(t.x)
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_field(2, 2, 6),
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_field(3, 3, 5),
            Bytecode::call(2, 2, 1),
            // t.f(t[2])
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_field(2, 2, 6),
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_index(3, 3, 2),
            Bytecode::call(2, 2, 1),
            // t.f(t[1000])
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_field(2, 2, 6),
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::load_integer(4, 1000),
            Bytecode::get_table(3, 3, 4),
            Bytecode::call(2, 2, 1),
            // EOF
            Bytecode::return_bytecode(2, 1, 1),
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
