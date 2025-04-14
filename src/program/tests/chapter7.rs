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
            Bytecode::variadic_arguments_prepare(0.into()),
            // g1 = 1
            Bytecode::set_uptable(0.into(), 0.into(), 1.into(), true.into()),
            // g2 = 2
            Bytecode::set_uptable(0.into(), 2.into(), 3.into(), true.into()),
            // if g1 or g2 and g3 then
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::test(0.into(), true.into()),
            Bytecode::jump(6i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(6i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //     print "test only once"
            Bytecode::get_uptable(0.into(), 0.into(), 5.into()),
            Bytecode::load_constant(1.into(), 6u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end

            // if g3 or g1 and g2 then
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::test(0.into(), true.into()),
            Bytecode::jump(6i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(6i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //     print "test 3 times"
            Bytecode::get_uptable(0.into(), 0.into(), 5.into()),
            Bytecode::load_constant(1.into(), 7u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end

            // if (g3 or g1) and (g2 or g4) then
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::test(0.into(), true.into()),
            Bytecode::jump(3i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(9i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::test(0.into(), true.into()),
            Bytecode::jump(3i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 8.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //     print "test 3 times"
            Bytecode::get_uptable(0.into(), 0.into(), 5.into()),
            Bytecode::load_constant(1.into(), 7u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end

            // if (g3 or g1) and (g2 and g4) then
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::test(0.into(), true.into()),
            Bytecode::jump(3i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(9i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 2.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(6i8.into()),
            Bytecode::get_uptable(0.into(), 0.into(), 8.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //     print "test 4 times and fail"
            Bytecode::get_uptable(0.into(), 0.into(), 5.into()),
            Bytecode::load_constant(1.into(), 9u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
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
            Bytecode::variadic_arguments_prepare(0.into()),
            // g1 = 1
            Bytecode::set_uptable(0.into(), 0.into(), 1.into(), true.into()),
            // g2 = 2
            Bytecode::set_uptable(0.into(), 2.into(), 3.into(), true.into()),
            // print( g1 or g2 and g3)
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::test(1.into(), true.into()),
            Bytecode::jump(4i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 2.into()),
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 5.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print( g3 or g1 and g2)
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 5.into()),
            Bytecode::test(1.into(), true.into()),
            Bytecode::jump(4i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 2.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print( (g3 or g1) and (g2 or g4))
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 5.into()),
            Bytecode::test(1.into(), true.into()),
            Bytecode::jump(3i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(4i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 2.into()),
            Bytecode::test(1.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 6.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // print( (g3 or g1) and (g2 and g4))
            Bytecode::get_uptable(0.into(), 0.into(), 4.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 5.into()),
            Bytecode::test(1.into(), true.into()),
            Bytecode::jump(3i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(4i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 2.into()),
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 6.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
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
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a, b = 123, "hello"
            Bytecode::load_integer(0.into(), 123i8.into()),
            Bytecode::load_constant(1.into(), 0u8.into()),
            // if a >= 123 and b == "hello" then
            Bytecode::greater_equal_integer(0.into(), 123.into(), false.into()),
            Bytecode::jump(5i8.into()),
            Bytecode::equal_constant(1.into(), 0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //     print "yes"
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::load_constant(3.into(), 2u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end

            // if b <= "world" then
            Bytecode::load_constant(2.into(), 3u8.into()),
            Bytecode::less_equal(1.into(), 2.into(), false.into()),
            Bytecode::jump(6i8.into()),
            //     print (a>100)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::greater_than_integer(0.into(), 100.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::load_false_skip(3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end

            // print (a == 1000 and b == "hello")
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::equal_constant(0.into(), 4.into(), false.into()),
            Bytecode::jump(2i8.into()),
            Bytecode::equal_constant(1.into(), 0.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::load_false_skip(3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print (a<b)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::less_than(0.into(), 1.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::load_false_skip(3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print (a>b)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::less_than(1.into(), 0.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::load_false_skip(3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print (a<=b)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::less_equal(0.into(), 1.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::load_false_skip(3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // print (a>=b)
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::less_equal(1.into(), 0.into(), true.into()),
            Bytecode::jump(1i8.into()),
            Bytecode::load_false_skip(3.into()),
            Bytecode::load_true(3.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(2.into(), 1.into(), 1.into()),
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
