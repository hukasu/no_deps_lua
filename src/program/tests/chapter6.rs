use crate::{Program, bytecode::Bytecode, program::Local};

#[test]
fn if_statement() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
if a then
    print "skip this"
end
if print then
    local a = "I am true"
    print(a)
end

print (a) -- should be nil
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // if a then
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            // print "skip this"
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::load_constant(1.into(), 2u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            // if print then
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(4i8.into()),
            // local a = "I am true"
            Bytecode::load_constant(0.into(), 3u8.into()),
            // print(a)
            Bytecode::get_uptable(1.into(), 0.into(), 1.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // end
            // print (a) -- should be nil
            Bytecode::get_uptable(0.into(), 0.into(), 1.into()),
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &[
            "a".into(),
            "print".into(),
            "skip this".into(),
            "I am true".into(),
        ],
        &[Local::new("a".into(), 12, 15)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn if_else() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local a,b = 123
if b then
  print "not here"
elseif g then
  print "not here"
elseif a then
  print "yes, here"
else
  print "not here"
end

if b then
  print "not here"
else
  print "yes, here"
end

if b then
  print "not here"
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a,b = 123
            Bytecode::load_integer(0.into(), 123i8.into()),
            Bytecode::load_nil(1.into(), 0.into()),
            // if b then
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(4i8.into()),
            //   print "not here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 1u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            Bytecode::jump(16i8.into()),
            // elseif g then
            Bytecode::get_uptable(2.into(), 0.into(), 2.into()),
            Bytecode::test(2.into(), false.into()),
            Bytecode::jump(4i8.into()),
            //   print "not here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 1u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            Bytecode::jump(9i8.into()),
            // elseif a then
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(4i8.into()),
            //   print "yes, here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 3u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            Bytecode::jump(3i8.into()),
            // else
            //   print "not here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 1u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end
            // if b then
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(4i8.into()),
            //   print "not here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 1u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            Bytecode::jump(3i8.into()),
            // else
            //   print "yes, here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 3u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end
            // if b then
            Bytecode::test(1.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //   print "yes, here"
            Bytecode::get_uptable(2.into(), 0.into(), 0.into()),
            Bytecode::load_constant(3.into(), 1u8.into()),
            Bytecode::call(2.into(), 2.into(), 1.into()),
            // end
            // EOF
            Bytecode::return_bytecode(2.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "not here".into(),
            "g".into(),
            "yes, here".into(),
        ],
        &[Local::new("a".into(), 4, 41), Local::new("b".into(), 4, 41)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn while_statement() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local a = 123
while a do
  print(a)
  a = not a
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a = 123
            Bytecode::load_integer(0.into(), 123i8.into()),
            // while a do
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(5i8.into()),
            //   print(a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            //   a = not a
            Bytecode::not(0.into(), 0.into()),
            // end
            Bytecode::jump((-7i8).into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[Local::new("a".into(), 3, 11)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn break_statement() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local z = 1
while z do
  while z do
    print "break inner"
    break
    print "unreachable inner"
  end

  print "break outer"
  break
  print "unreachable outer"
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local z = 1
            Bytecode::load_integer(0.into(), 1i8.into()),
            // while z do
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(18i8.into()),
            //   while z do
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump(8i8.into()),
            //     print "break inner"
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 1u8.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            //     break
            Bytecode::jump(4i8.into()),
            //     print "unreachable inner"
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 2u8.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            //   end
            Bytecode::jump((-10i8).into()),
            //   print "break outer"
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 3u8.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            //   break
            Bytecode::jump(4i8.into()),
            //   print "unreachable outer"
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 4u8.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // end
            Bytecode::jump((-20i8).into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "break inner".into(),
            "unreachable inner".into(),
            "break outer".into(),
            "unreachable outer".into(),
        ],
        &[Local::new("z".into(), 3, 24)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn repeat() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local a = false
repeat
  print(a)
  a = not a
until a
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a = false
            Bytecode::load_false(0.into()),
            // repeat
            //   print(a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            //   a = not a
            Bytecode::not(0.into(), 0.into()),
            // until a
            Bytecode::test(0.into(), false.into()),
            Bytecode::jump((-6i8).into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[Local::new("a".into(), 3, 10)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn for_statement() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
-- 1~3
for i = 1, 3, 1 do
    print(i)
end

-- negetive step, 1~-2
for i = 1, -2, -1 do
    print(i)
end

-- float limit, 1~3
for i = 1, 3.2 do
    print(i)
end

-- float i, 1.0~3.0
for i = 1.0, 3 do
    print(i)
end

-- special case, should not run
local max = 9223372036854775807
for i = max, max*10.0, -1 do
    print (i)
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // for i = 1, 3, 1 do
            Bytecode::load_integer(0.into(), 1i8.into()),
            Bytecode::load_integer(1.into(), 3i8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::for_prepare(0.into(), 3u8.into()),
            //     print(i)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 3.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            // end
            Bytecode::for_loop(0.into(), 4u8.into()),
            // for i = 1, -2, -1 do
            Bytecode::load_integer(0.into(), 1i8.into()),
            Bytecode::load_integer(1.into(), (-2i8).into()),
            Bytecode::load_integer(2.into(), (-1i8).into()),
            Bytecode::for_prepare(0.into(), 3u8.into()),
            //     print(i)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 3.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            // end
            Bytecode::for_loop(0.into(), 4u8.into()),
            // for i = 1, 3.2 do
            Bytecode::load_integer(0.into(), 1i8.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::for_prepare(0.into(), 3u8.into()),
            //     print(i)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 3.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            // end
            Bytecode::for_loop(0.into(), 4u8.into()),
            // for i = 1.0, 3 do
            Bytecode::load_float(0.into(), 1i8.into()),
            Bytecode::load_integer(1.into(), 3i8.into()),
            Bytecode::load_integer(2.into(), 1i8.into()),
            Bytecode::for_prepare(0.into(), 3u8.into()),
            //     print(i)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 3.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            // end
            Bytecode::for_loop(0.into(), 4u8.into()),
            // local max = 9223372036854775807
            Bytecode::load_constant(0.into(), 2u8.into()),
            // for i = max, max*10.0, -1 do
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::mul_constant(2.into(), 0.into(), 3.into()),
            // TODO MMBINK
            Bytecode::load_integer(3.into(), (-1i8).into()),
            Bytecode::for_prepare(1.into(), 3u8.into()),
            //     print (i)
            Bytecode::get_uptable(5.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(6.into(), 4.into()),
            Bytecode::call(5.into(), 2.into(), 1.into()),
            // end
            Bytecode::for_loop(1.into(), 4u8.into()),
            // EOF
            Bytecode::return_bytecode(1.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            3.2f64.into(),
            9223372036854775807i64.into(),
            10.0f64.into(),
        ],
        &[
            // TODO update when implementing MMBINK
            Local::new("?for_start".into(), 5, 10),
            Local::new("?for_end".into(), 5, 10),
            Local::new("?for_step".into(), 5, 10),
            Local::new("i".into(), 6, 9),
            Local::new("?for_start".into(), 13, 18),
            Local::new("?for_end".into(), 13, 18),
            Local::new("?for_step".into(), 13, 18),
            Local::new("i".into(), 14, 17),
            Local::new("?for_start".into(), 21, 26),
            Local::new("?for_end".into(), 21, 26),
            Local::new("?for_step".into(), 21, 26),
            Local::new("i".into(), 22, 25),
            Local::new("?for_start".into(), 29, 34),
            Local::new("?for_end".into(), 29, 34),
            Local::new("?for_step".into(), 29, 34),
            Local::new("i".into(), 30, 33),
            Local::new("max".into(), 35, 44),
            Local::new("?for_start".into(), 38, 43),
            Local::new("?for_end".into(), 38, 43),
            Local::new("?for_step".into(), 38, 43),
            Local::new("i".into(), 39, 42),
        ],
        &["_ENV".into()],
        0,
    );
    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn goto() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
::label1::
print("block: 1")
goto label2
::label3::
print("block: 3")
goto label4
::label2::
do
  print("block: 2")
  goto label3 -- goto outer block
end
::label4::
print("block: 4")

do
  goto label
  local a = 'local var'
  ::label:: -- skip the local var but it's OK
  ::another:: -- void statment
  ;; -- void statment
  ::another2:: -- void statment
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // print("block: 1")
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // goto label2
            Bytecode::jump(4i8.into()),
            // print("block: 3")
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 2u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // goto label4
            Bytecode::jump(4i8.into()),
            // do
            //   print("block: 2")
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 3u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            //   goto label3 -- goto outer block
            Bytecode::jump((-8i8).into()),
            // end
            // print("block: 4")
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 4u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // do
            //   goto label
            Bytecode::jump(1i8.into()),
            //   local a = 'local var'
            Bytecode::load_constant(0.into(), 5u8.into()),
            // end
            // EOF
            Bytecode::return_bytecode(0.into(), 1.into(), 1.into()),
        ],
        &[
            "print".into(),
            "block: 1".into(),
            "block: 3".into(),
            "block: 2".into(),
            "block: 4".into(),
            "local var".into(),
        ],
        &[Local::new("a".into(), 19, 19)],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn local_in_blocks() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local a = 5
if a > 10 then
    local b = 100
elseif a < 0 then
    local b = 101
else
    local b = 102
end
do
    local b = 103
end
for i = 1,2 do
    local b = 104
end
repeat
    local b = 105
until a > 0
while a > 0 do
    local b = 106
    break
end
local c = 10

"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local a = 5
            Bytecode::load_integer(0.into(), 5i8.into()),
            // if a > 10 then
            Bytecode::greater_than_integer(0.into(), 10.into(), false.into()),
            Bytecode::jump(2i8.into()),
            //     local b = 100
            Bytecode::load_integer(1.into(), 100i8.into()),
            Bytecode::jump(5i8.into()),
            // elseif a < 0 then
            Bytecode::less_than_integer(0.into(), 0.into(), false.into()),
            Bytecode::jump(2i8.into()),
            //     local b = 101
            Bytecode::load_integer(1.into(), 101i8.into()),
            Bytecode::jump(1i8.into()),
            // else
            //     local b = 102
            Bytecode::load_integer(1.into(), 102i8.into()),
            // end
            // do
            //     local b = 103
            Bytecode::load_integer(1.into(), 103i8.into()),
            // end
            // for i = 1,2 do
            Bytecode::load_integer(1.into(), 1i8.into()),
            Bytecode::load_integer(2.into(), 2i8.into()),
            Bytecode::load_integer(3.into(), 1i8.into()),
            Bytecode::for_prepare(1.into(), 1u8.into()),
            //     local b = 104
            Bytecode::load_integer(5.into(), 104i8.into()),
            // end
            Bytecode::for_loop(1.into(), 2u8.into()),
            // repeat
            //     local b = 105
            Bytecode::load_integer(1.into(), 105i8.into()),
            // until a > 0
            Bytecode::greater_than_integer(0.into(), 0.into(), false.into()),
            Bytecode::jump((-3i8).into()),
            // while a > 0 do
            Bytecode::greater_than_integer(0.into(), 0.into(), false.into()),
            Bytecode::jump(3i8.into()),
            //     local b = 106
            Bytecode::load_integer(1.into(), 106i8.into()),
            //     break
            Bytecode::jump(1i8.into()),
            // end
            Bytecode::jump((-5i8).into()),
            // local c = 10
            Bytecode::load_integer(1.into(), 10i8.into()),
            // EOF
            Bytecode::return_bytecode(2.into(), 1.into(), 1.into()),
        ],
        &[],
        &[
            Local::new("a".into(), 3, 29),
            Local::new("b".into(), 6, 6),
            Local::new("b".into(), 10, 10),
            Local::new("b".into(), 12, 12),
            Local::new("b".into(), 13, 13),
            Local::new("?for_start".into(), 16, 19),
            Local::new("?for_end".into(), 16, 19),
            Local::new("?for_step".into(), 16, 19),
            Local::new("i".into(), 17, 18),
            Local::new("b".into(), 18, 18),
            Local::new("b".into(), 20, 22),
            Local::new("b".into(), 25, 26),
            Local::new("c".into(), 28, 29),
        ],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}
