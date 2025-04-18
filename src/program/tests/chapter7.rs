use crate::{Error, Program, bytecode::Bytecode, program::Local};

#[test]
fn and_or() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
g1 = 1
g2 = 2

if g1 or g2 and g3 then
    print "test only once"
end

if g3 or g1 and g2 then
    print "test 3 times"
end

if (g3 or g1) and (g2 or g4) then
    print "test 3 times"
end

if (g3 or g1) and (g2 and g4) then
    print "test 4 times and fail"
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // g1 = 1
            Bytecode::set_uptable(0, 0, 1, true),
            // g2 = 2
            Bytecode::set_uptable(0, 2, 3, true),
            // if g1 or g2 and g3 then
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::test(0, true),
            Bytecode::jump(6i8),
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::test(0, false),
            Bytecode::jump(6i8),
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::test(0, false),
            Bytecode::jump(3i8),
            //     print "test only once"
            Bytecode::get_uptable(0, 0, 5),
            Bytecode::load_constant(1, 6u8),
            Bytecode::call(0, 2, 1),
            // end

            // if g3 or g1 and g2 then
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::test(0, true),
            Bytecode::jump(6i8),
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::test(0, false),
            Bytecode::jump(6i8),
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::test(0, false),
            Bytecode::jump(3i8),
            //     print "test 3 times"
            Bytecode::get_uptable(0, 0, 5),
            Bytecode::load_constant(1, 7u8),
            Bytecode::call(0, 2, 1),
            // end

            // if (g3 or g1) and (g2 or g4) then
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::test(0, true),
            Bytecode::jump(3i8),
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::test(0, false),
            Bytecode::jump(9i8),
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::test(0, true),
            Bytecode::jump(3i8),
            Bytecode::get_uptable(0, 0, 8),
            Bytecode::test(0, false),
            Bytecode::jump(3i8),
            //     print "test 3 times"
            Bytecode::get_uptable(0, 0, 5),
            Bytecode::load_constant(1, 7u8),
            Bytecode::call(0, 2, 1),
            // end

            // if (g3 or g1) and (g2 and g4) then
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::test(0, true),
            Bytecode::jump(3i8),
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::test(0, false),
            Bytecode::jump(9i8),
            Bytecode::get_uptable(0, 0, 2),
            Bytecode::test(0, false),
            Bytecode::jump(6i8),
            Bytecode::get_uptable(0, 0, 8),
            Bytecode::test(0, false),
            Bytecode::jump(3i8),
            //     print "test 4 times and fail"
            Bytecode::get_uptable(0, 0, 5),
            Bytecode::load_constant(1, 9u8),
            Bytecode::call(0, 2, 1),
            // end
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &[
            "g1".into(),
            1i64.into(),
            "g2".into(),
            2i64.into(),
            "g3".into(),
            "print".into(),
            "test only once".into(),
            "test 3 times".into(),
            "g4".into(),
            "test 4 times and fail".into(),
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn test_set() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
g1 = 1
g2 = 2

print( g1 or g2 and g3)
print( g3 or g1 and g2)
print( (g3 or g1) and (g2 or g4))
print( (g3 or g1) and (g2 and g4))
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // g1 = 1
            Bytecode::set_uptable(0, 0, 1, true),
            // g2 = 2
            Bytecode::set_uptable(0, 2, 3, true),
            // print( g1 or g2 and g3)
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::test(1, true),
            Bytecode::jump(4i8),
            Bytecode::get_uptable(1, 0, 2),
            Bytecode::test(1, false),
            Bytecode::jump(1i8),
            Bytecode::get_uptable(1, 0, 5),
            Bytecode::call(0, 2, 1),
            // print( g3 or g1 and g2)
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::get_uptable(1, 0, 5),
            Bytecode::test(1, true),
            Bytecode::jump(4i8),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::test(1, false),
            Bytecode::jump(1i8),
            Bytecode::get_uptable(1, 0, 2),
            Bytecode::call(0, 2, 1),
            // print( (g3 or g1) and (g2 or g4))
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::get_uptable(1, 0, 5),
            Bytecode::test(1, true),
            Bytecode::jump(3i8),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::test(1, false),
            Bytecode::jump(4i8),
            Bytecode::get_uptable(1, 0, 2),
            Bytecode::test(1, true),
            Bytecode::jump(1i8),
            Bytecode::get_uptable(1, 0, 6),
            Bytecode::call(0, 2, 1),
            // print( (g3 or g1) and (g2 and g4))
            Bytecode::get_uptable(0, 0, 4),
            Bytecode::get_uptable(1, 0, 5),
            Bytecode::test(1, true),
            Bytecode::jump(3i8),
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::test(1, false),
            Bytecode::jump(4i8),
            Bytecode::get_uptable(1, 0, 2),
            Bytecode::test(1, false),
            Bytecode::jump(1i8),
            Bytecode::get_uptable(1, 0, 6),
            Bytecode::call(0, 2, 1),
            // EOF
            Bytecode::return_bytecode(0, 1, 1),
        ],
        &[
            "g1".into(),
            1i64.into(),
            "g2".into(),
            2i64.into(),
            "print".into(),
            "g3".into(),
            "g4".into(),
        ],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn compare() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let code = r#"
local a, b = 123, "hello"
if a >= 123 and b == "hello" then
    print "yes"
end

if b <= "world" then
    print (a>100)
end

print (a == 1000 and b == "hello")
print (a<b)
print (a>b)
print (a<=b)
print (a>=b)
"#;

    let program = Program::parse(code).unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local a, b = 123, "hello"
            Bytecode::load_integer(0, 123i8),
            Bytecode::load_constant(1, 0u8),
            // if a >= 123 and b == "hello" then
            Bytecode::greater_equal_integer(0, 123, false),
            Bytecode::jump(5i8),
            Bytecode::equal_constant(1, 0, false),
            Bytecode::jump(3i8),
            //     print "yes"
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::load_constant(3, 2u8),
            Bytecode::call(2, 2, 1),
            // end

            // if b <= "world" then
            Bytecode::load_constant(2, 3u8),
            Bytecode::less_equal(1, 2, false),
            Bytecode::jump(6i8),
            //     print (a>100)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::greater_than_integer(0, 100, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // end

            // print (a == 1000 and b == "hello")
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::equal_constant(0, 4, false),
            Bytecode::jump(2i8),
            Bytecode::equal_constant(1, 0, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // print (a<b)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::less_than(0, 1, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // print (a>b)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::less_than(1, 0, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // print (a<=b)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::less_equal(0, 1, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // print (a>=b)
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::less_equal(1, 0, true),
            Bytecode::jump(1i8),
            Bytecode::load_false_skip(3),
            Bytecode::load_true(3),
            Bytecode::call(2, 2, 1),
            // EOF
            Bytecode::return_bytecode(2, 1, 1),
        ],
        &[
            "hello".into(),
            "print".into(),
            "yes".into(),
            "world".into(),
            1000i64.into(),
        ],
        &[Local::new("a".into(), 4, 53), Local::new("b".into(), 4, 53)],
        &["_ENV".into()],
        0,
    );

    match crate::Lua::run_program(program) {
        Err(err @ Error::RelationalOperand(_, _)) => log::error!("{}", err),
        Err(err) => panic!("Expected `RelationalOperand` error, but got {:?}.", err),
        Ok(_) => panic!("Last print should fail"),
    }
}
