use alloc::string::String;

use crate::{bytecode::Bytecode, ext::Unescape, program::Local, Program};

#[test]
fn escape() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // print "tab:\thi" -- tab
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1),
            Bytecode::call(0, 2, 1),
            // print "\xE4\xBD\xA0\xE5\xA5\xBD" -- 你好
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 2),
            Bytecode::call(0, 2, 1),
            // print "\xE4\xBD" -- invalid UTF-8
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 3),
            Bytecode::call(0, 2, 1),
            // print "\72\101\108\108\111" -- Hello
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 4),
            Bytecode::call(0, 2, 1),
            // print "null: \0." -- '\0'
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 5),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
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
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}

#[test]
fn strings() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

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

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local s = "hello_world"
            Bytecode::load_constant(0, 0),
            // local m = "middle_string_middle_string"
            Bytecode::load_constant(1, 1),
            // local l = "long_string_long_string_long_string_long_string_long_string"
            Bytecode::load_constant(2, 2),
            // print(s)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::move_bytecode(4, 0),
            Bytecode::call(3, 2, 1),
            // print(m)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::move_bytecode(4, 1),
            Bytecode::call(3, 2, 1),
            // print(l)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::move_bytecode(4, 2),
            Bytecode::call(3, 2, 1),
            // hello_world = 12
            Bytecode::set_uptable(0, 0, 4, 1),
            // middle_string_middle_string = 345
            Bytecode::set_uptable(0, 1, 5, 1),
            // long_string_long_string_long_string_long_string_long_string = 6789
            Bytecode::get_upvalue(3, 0),
            Bytecode::load_constant(4, 2),
            Bytecode::set_table(3, 4, 6, 1),
            // print(hello_world)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::call(3, 2, 1),
            // print(middle_string_middle_string)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::get_uptable(4, 0, 1),
            Bytecode::call(3, 2, 1),
            // print(long_string_long_string_long_string_long_string_long_string)
            Bytecode::get_uptable(3, 0, 3),
            Bytecode::get_upvalue(4, 0),
            Bytecode::load_constant(5, 2),
            Bytecode::get_table(4, 4, 5),
            Bytecode::call(3, 2, 1),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
        ],
        &[
            "hello_world".into(),
            "middle_string_middle_string".into(),
            "long_string_long_string_long_string_long_string_long_string".into(),
            "print".into(),
            12i64.into(),
            345i64.into(),
            6789i64.into(),
        ],
        &[
            Local::new("s".into(), 3, 31),
            Local::new("m".into(), 4, 31),
            Local::new("l".into(), 5, 31),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}
