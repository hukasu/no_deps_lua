use alloc::string::String;

use crate::{Program, bytecode::Bytecode, ext::Unescape, program::Local};

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
            Bytecode::variadic_arguments_prepare(0.into()),
            // print "tab:\thi" -- tab
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print "\xE4\xBD\xA0\xE5\xA5\xBD" -- 你好
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 2u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print "\xE4\xBD" -- invalid UTF-8
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print "\72\101\108\108\111" -- Hello
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 4u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print "null: \0." -- '\0'
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 5u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
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
            Bytecode::variadic_arguments_prepare(0.into()),
            // local s = "hello_world"
            Bytecode::load_constant(0.into(), 0u8.into()),
            // local m = "middle_string_middle_string"
            Bytecode::load_constant(1.into(), 1u8.into()),
            // local l = "long_string_long_string_long_string_long_string_long_string"
            Bytecode::load_constant(2.into(), 2u8.into()),
            // print(s)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::move_bytecode(4.into(), 0.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(m)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::move_bytecode(4.into(), 1.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(l)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::move_bytecode(4.into(), 2.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // hello_world = 12
            Bytecode::set_uptable(0.into(), 0.into(), 4.into(), true.into()),
            // middle_string_middle_string = 345
            Bytecode::set_uptable(0.into(), 1.into(), 5.into(), true.into()),
            // long_string_long_string_long_string_long_string_long_string = 6789
            Bytecode::get_upvalue(3.into(), 0.into()),
            Bytecode::load_constant(4.into(), 2u8.into()),
            Bytecode::set_table(3.into(), 4.into(), 6.into(), true.into()),
            // print(hello_world)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(middle_string_middle_string)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::get_uptable(4.into(), 0.into(), 1.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // print(long_string_long_string_long_string_long_string_long_string)
            Bytecode::get_uptable(3.into(), 0.into(), 3.into()),
            Bytecode::get_upvalue(4.into(), 0.into()),
            Bytecode::load_constant(5.into(), 2u8.into()),
            Bytecode::get_table(4.into(), 4.into(), 5.into()),
            Bytecode::call(3.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(3.into(), 1.into(), 1.into()),
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
