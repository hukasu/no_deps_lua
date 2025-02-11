use crate::{byte_code::ByteCode, Program};

#[test]
fn chapter6_if() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
    assert_eq!(
        &program.constants,
        &[
            "a".into(),
            "print".into(),
            "skip this".into(),
            "I am true".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // if a then
            ByteCode::GetGlobal(0, 0),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(3),
            // print "skip this"
            ByteCode::GetGlobal(0, 1),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
            // end
            // if print then
            ByteCode::GetGlobal(0, 1),
            ByteCode::Test(0, 0),
            ByteCode::Jmp(4),
            // local a = "I am true"
            ByteCode::LoadConstant(0, 3),
            // print(a)
            ByteCode::GetGlobal(1, 1),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            // end
            // print (a) -- should be nil
            ByteCode::GetGlobal(0, 1),
            ByteCode::GetGlobal(1, 0),
            ByteCode::Call(0, 1),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_if_else() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
  print "yes, here"
end
"#,
    )
    .unwrap();
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "not here".into(),
            "g".into(),
            "yes, here".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local a,b = 123
            ByteCode::LoadInt(0, 123),
            ByteCode::LoadNil(1),
            // if b then
            ByteCode::Test(1, 0),
            ByteCode::Jmp(4),
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(16),
            // elseif g then
            ByteCode::GetGlobal(2, 2),
            ByteCode::Test(2, 0),
            ByteCode::Jmp(4),
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(9),
            // elseif a then
            ByteCode::Test(0, 0),
            ByteCode::Jmp(4),
            //   print "yes, here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 3),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(3),
            // else
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            // end
            // if b then
            ByteCode::Test(1, 0),
            ByteCode::Jmp(4),
            //   print "not here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 1),
            ByteCode::Call(2, 1),
            ByteCode::Jmp(3),
            // else
            //   print "yes, here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 3),
            ByteCode::Call(2, 1),
            // end
            // if b then
            ByteCode::Test(1, 0),
            ByteCode::Jmp(3),
            //   print "yes, here"
            ByteCode::GetGlobal(2, 0),
            ByteCode::LoadConstant(3, 3),
            ByteCode::Call(2, 1),
            // end
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_while() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
    assert_eq!(&program.constants, &["print".into(),]);
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = 123
            ByteCode::LoadInt(0, 123),
            // while a do
            ByteCode::Test(0, 0),
            ByteCode::Jmp(5),
            //   print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            //   a = not a
            ByteCode::Not(0, 0),
            // end
            ByteCode::Jmp(-7),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_break() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "break inner".into(),
            "unreachable inner".into(),
            "break outer".into(),
            "unreachable outer".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // local z = 1
            ByteCode::LoadInt(0, 1),
            // while z do
            ByteCode::Test(0, 0),
            ByteCode::Jmp(18),
            //   while z do
            ByteCode::Test(0, 0),
            ByteCode::Jmp(8),
            //     print "break inner"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 1),
            ByteCode::Call(1, 1),
            //     break
            ByteCode::Jmp(4),
            //     print "unreachable inner"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 2),
            ByteCode::Call(1, 1),
            //   end
            ByteCode::Jmp(-10),
            //   print "break outer"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 3),
            ByteCode::Call(1, 1),
            //   break
            ByteCode::Jmp(4),
            //   print "unreachable outer"
            ByteCode::GetGlobal(1, 0),
            ByteCode::LoadConstant(2, 4),
            ByteCode::Call(1, 1),
            // end
            ByteCode::Jmp(-20),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_repeat() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
    assert_eq!(&program.constants, &["print".into(),]);
    assert_eq!(
        &program.byte_codes,
        &[
            // local a = false
            ByteCode::LoadFalse(0),
            // repeat
            //   print(a)
            ByteCode::GetGlobal(1, 0),
            ByteCode::Move(2, 0),
            ByteCode::Call(1, 1),
            //   a = not a
            ByteCode::Not(0, 0),
            // until a
            ByteCode::Test(0, 0),
            ByteCode::Jmp(-6),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_for() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            3.2f64.into(),
            9223372036854775807i64.into(),
            10.0f64.into()
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // for i = 1, 3, 1 do
            ByteCode::LoadInt(0, 1),
            ByteCode::LoadInt(1, 3),
            ByteCode::LoadInt(2, 1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // for i = 1, -2, -1 do
            ByteCode::LoadInt(0, 1),
            ByteCode::LoadInt(1, -2),
            ByteCode::LoadInt(2, -1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // for i = 1, 3.2 do
            ByteCode::LoadInt(0, 1),
            ByteCode::LoadConstant(1, 1),
            ByteCode::LoadInt(2, 1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // for i = 1.0, 3 do
            ByteCode::LoadFloat(0, 1),
            ByteCode::LoadInt(1, 3),
            ByteCode::LoadInt(2, 1),
            ByteCode::ForPrepare(0, 3),
            //     print(i)
            ByteCode::GetGlobal(4, 0),
            ByteCode::Move(5, 3),
            ByteCode::Call(4, 1),
            // end
            ByteCode::ForLoop(0, 4),
            // local max = 9223372036854775807
            ByteCode::LoadConstant(0, 2),
            // for i = max, max*10.0, -1 do
            ByteCode::Move(1, 0),
            ByteCode::MulConstant(2, 0, 3),
            ByteCode::LoadInt(3, -1),
            ByteCode::ForPrepare(1, 3),
            //     print (i)
            ByteCode::GetGlobal(5, 0),
            ByteCode::Move(6, 4),
            ByteCode::Call(5, 1),
            // end
            ByteCode::ForLoop(1, 4),
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}

#[test]
fn chapter6_goto() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default());
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
    assert_eq!(
        &program.constants,
        &[
            "print".into(),
            "block: 1".into(),
            "block: 3".into(),
            "block: 2".into(),
            "block: 4".into(),
            "local var".into(),
        ]
    );
    assert_eq!(
        &program.byte_codes,
        &[
            // print("block: 1")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 1),
            ByteCode::Call(0, 1),
            // goto label2
            ByteCode::Jmp(4),
            // print("block: 3")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 2),
            ByteCode::Call(0, 1),
            // goto label4
            ByteCode::Jmp(4),
            // do
            //   print("block: 2")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 3),
            ByteCode::Call(0, 1),
            //   goto label3 -- goto outer block
            ByteCode::Jmp(-8),
            // end
            // print("block: 4")
            ByteCode::GetGlobal(0, 0),
            ByteCode::LoadConstant(1, 4),
            ByteCode::Call(0, 1),
            // do
            //   goto label
            ByteCode::Jmp(1),
            //   local a = 'local var'
            ByteCode::LoadConstant(0, 5),
            // end
        ]
    );
    crate::Lua::execute(&program).expect("Should run");
}
