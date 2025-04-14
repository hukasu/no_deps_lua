use crate::{Program, bytecode::Bytecode, program::Local};

#[test]
fn upvalues() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
g1, g2 = 1, 2
local up1, up2, up3, up4 = 11, 12, 13, 14
local print = print
local function foo()
    local l1, l2 = 101, 102
    l1, g1 = g2, l2
    print(l1, g1)

    -- assign to upvalues
    up1, up2, up3 = l1, g1, up4
    print(up1, up2, up3)

    -- assign by upvalues
    l1, g1, up1 = up2, up3, up4
    print(l1, g1, up1)

    local inner = function()
        -- assign to upvalues
        up1, up2, up3 = 101, g2, up4
        print(up1, up2, up3)
    end
    inner()
end

foo()
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // g1, g2 = 1, 2
            Bytecode::load_integer(0.into(), 1i8.into()),
            Bytecode::set_uptable(0.into(), 1.into(), 2.into(), true.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 0.into(), false.into()),
            // local up1, up2, up3, up4 = 11, 12, 13, 14
            Bytecode::load_integer(0.into(), 11i8.into()),
            Bytecode::load_integer(1.into(), 12i8.into()),
            Bytecode::load_integer(2.into(), 13i8.into()),
            Bytecode::load_integer(3.into(), 14i8.into()),
            // local print = print
            Bytecode::get_uptable(4.into(), 0.into(), 3.into()),
            // local function foo()
            Bytecode::closure(5.into(), 0u8.into()),
            // foo()
            Bytecode::move_bytecode(6.into(), 5.into()),
            Bytecode::call(6.into(), 1.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(6.into(), 1.into(), 1.into()),
        ],
        &["g1".into(), "g2".into(), 2i64.into(), "print".into()],
        &[
            Local::new("up1".into(), 9, 14),
            Local::new("up2".into(), 9, 14),
            Local::new("up3".into(), 9, 14),
            Local::new("up4".into(), 9, 14),
            Local::new("print".into(), 10, 14),
            Local::new("foo".into(), 11, 14),
        ],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function foo()
            //     local l1, l2 = 101, 102
            Bytecode::load_integer(0.into(), 101i8.into()),
            Bytecode::load_integer(1.into(), 102i8.into()),
            //     l1, g1 = g2, l2
            Bytecode::get_uptable(2.into(), 0.into(), 1.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 1.into(), false.into()),
            Bytecode::move_bytecode(0.into(), 2.into()),
            //     print(l1, g1)
            Bytecode::get_upvalue(2.into(), 1.into()),
            Bytecode::move_bytecode(3.into(), 0.into()),
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::call(2.into(), 3.into(), 1.into()),
            //     up1, up2, up3 = l1, g1, up4
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::get_uptable(3.into(), 0.into(), 0.into()),
            Bytecode::get_upvalue(4.into(), 5.into()),
            Bytecode::set_upvalue(4.into(), 4.into()),
            Bytecode::set_upvalue(3.into(), 3.into()),
            Bytecode::set_upvalue(2.into(), 2.into()),
            //     print(up1, up2, up3)
            Bytecode::get_upvalue(2.into(), 1.into()),
            Bytecode::get_upvalue(3.into(), 2.into()),
            Bytecode::get_upvalue(4.into(), 3.into()),
            Bytecode::get_upvalue(5.into(), 4.into()),
            Bytecode::call(2.into(), 4.into(), 1.into()),
            //     l1, g1, up1 = up2, up3, up4
            Bytecode::get_upvalue(2.into(), 3.into()),
            Bytecode::get_upvalue(3.into(), 4.into()),
            Bytecode::get_upvalue(4.into(), 5.into()),
            Bytecode::set_upvalue(2.into(), 4.into()),
            Bytecode::set_uptable(0.into(), 0.into(), 3.into(), false.into()),
            Bytecode::move_bytecode(0.into(), 2.into()),
            //     print(l1, g1, up1)
            Bytecode::get_upvalue(2.into(), 1.into()),
            Bytecode::move_bytecode(3.into(), 0.into()),
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::get_upvalue(5.into(), 2.into()),
            Bytecode::call(2.into(), 4.into(), 1.into()),
            //     local inner = function()
            Bytecode::closure(2.into(), 0u8.into()),
            //     inner()
            Bytecode::move_bytecode(3.into(), 2.into()),
            Bytecode::call(3.into(), 1.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["g1".into(), "g2".into()],
        &[
            Local::new("l1".into(), 3, 36),
            Local::new("l2".into(), 3, 36),
            Local::new("inner".into(), 33, 36),
        ],
        &[
            "_ENV".into(),
            "print".into(),
            "up1".into(),
            "up2".into(),
            "up3".into(),
            "up4".into(),
        ],
        1,
    );

    let closure = super::get_closure_program(closure, 0);
    super::compare_program(
        closure,
        &[
            // local inner = function()
            //     up1, up2, up3 = 101, g2, up4
            Bytecode::load_integer(0.into(), 101i8.into()),
            Bytecode::get_uptable(1.into(), 3.into(), 0.into()),
            Bytecode::get_upvalue(2.into(), 4.into()),
            Bytecode::set_upvalue(2.into(), 2.into()),
            Bytecode::set_upvalue(1.into(), 1.into()),
            Bytecode::set_upvalue(0.into(), 0.into()),
            //     print(up1, up2, up3)
            Bytecode::get_upvalue(0.into(), 5.into()),
            Bytecode::get_upvalue(1.into(), 0.into()),
            Bytecode::get_upvalue(2.into(), 1.into()),
            Bytecode::get_upvalue(3.into(), 2.into()),
            Bytecode::call(0.into(), 4.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["g2".into()],
        &[],
        &[
            "up1".into(),
            "up2".into(),
            "up3".into(),
            "_ENV".into(),
            "up4".into(),
            "print".into(),
        ],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn broker() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function factory()
    local i = 0
    return function()
        print(i)
	    i = i + 1
    end
end

local f1 = factory()
f1()
f1()
local f2 = factory()
f2()
f1()
f2()
f1()
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function factory()
            Bytecode::closure(0.into(), 0u8.into()),
            // local f1 = factory()
            Bytecode::move_bytecode(1.into(), 0.into()),
            Bytecode::call(1.into(), 1.into(), 2.into()),
            // f1()
            Bytecode::move_bytecode(2.into(), 1.into()),
            Bytecode::call(2.into(), 1.into(), 1.into()),
            // f1()
            Bytecode::move_bytecode(2.into(), 1.into()),
            Bytecode::call(2.into(), 1.into(), 1.into()),
            // local f2 = factory()
            Bytecode::move_bytecode(2.into(), 0.into()),
            Bytecode::call(2.into(), 1.into(), 2.into()),
            // f2()
            Bytecode::move_bytecode(3.into(), 2.into()),
            Bytecode::call(3.into(), 1.into(), 1.into()),
            // f1()
            Bytecode::move_bytecode(3.into(), 1.into()),
            Bytecode::call(3.into(), 1.into(), 1.into()),
            // f2()
            Bytecode::move_bytecode(3.into(), 2.into()),
            Bytecode::call(3.into(), 1.into(), 1.into()),
            // f1()
            Bytecode::move_bytecode(3.into(), 1.into()),
            Bytecode::call(3.into(), 1.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(3.into(), 1.into(), 1.into()),
        ],
        &[],
        &[
            Local::new("factory".into(), 3, 20),
            Local::new("f1".into(), 5, 20),
            Local::new("f2".into(), 11, 20),
        ],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function factory()
            //     local i = 0
            Bytecode::load_integer(0.into(), 0i8.into()),
            //     return function()
            Bytecode::closure(1.into(), 0u8.into()),
            Bytecode::return_bytecode(1.into(), 2.into(), 0.into()),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[Local::new("i".into(), 2, 5)],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(closure, 0);
    super::compare_program(
        closure,
        &[
            // function()
            //     print(i)
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::get_upvalue(1.into(), 1.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            //     i = i + 1
            Bytecode::get_upvalue(0.into(), 1.into()),
            Bytecode::add_integer(0.into(), 0.into(), 1.into()),
            // MMBINI
            Bytecode::set_upvalue(1.into(), 0.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        &[],
        &["_ENV".into(), "i".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn env() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local function my_print(a)
    print("test _ENV:", a)
end

-- _ENV as local variable
local function test_local_env()
    local _ENV = { print = my_print }
    print "hello, world!" -- this `print` is my_print
end

test_local_env()

-- _ENV as upvalue
local _ENV = { print = my_print }
local function test_upvalue_env()
    print "hello, upvalue!" -- this `print` is my_print
end

test_upvalue_env()
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local function my_print(a)
            Bytecode::closure(0.into(), 0u8.into()),
            // local function test_local_env()
            Bytecode::closure(1.into(), 1u8.into()),
            // test_local_env()
            Bytecode::move_bytecode(2.into(), 1.into()),
            Bytecode::call(2.into(), 1.into(), 1.into()),
            // local _ENV = { print = my_print }
            Bytecode::new_table(2.into(), 1.into(), 0.into()),
            // EXTRAARG
            Bytecode::set_field(2.into(), 0.into(), 0.into(), false.into()),
            // local function test_upvalue_env()
            Bytecode::closure(3.into(), 2u8.into()),
            // test_upvalue_env()
            Bytecode::move_bytecode(4.into(), 3.into()),
            Bytecode::call(4.into(), 1.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(4.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[
            Local::new("my_print".into(), 3, 12),
            Local::new("test_local_env".into(), 4, 12),
            Local::new("_ENV".into(), 8, 12),
            Local::new("test_upvalue_env".into(), 9, 12),
        ],
        &["_ENV".into()],
        3,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function my_print(a)
            //     print("test _ENV:", a)
            Bytecode::get_uptable(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 1u8.into()),
            Bytecode::move_bytecode(3.into(), 0.into()),
            Bytecode::call(1.into(), 3.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "test _ENV:".into()],
        &[Local::new("a".into(), 1, 6)],
        &["_ENV".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // local function test_local_env()
            //     local _ENV = { print = my_print }
            Bytecode::new_table(0.into(), 1.into(), 0.into()),
            // EXTRAARG
            Bytecode::get_upvalue(1.into(), 0.into()),
            Bytecode::set_field(0.into(), 0.into(), 1.into(), false.into()),
            //     print "hello, world!" -- this `print` is my_print
            Bytecode::get_field(1.into(), 0.into(), 0.into()),
            Bytecode::load_constant(2.into(), 1u8.into()),
            Bytecode::call(1.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello, world!".into()],
        &[Local::new("_ENV".into(), 4, 8)],
        &["my_print".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 2);
    super::compare_program(
        closure,
        &[
            // local function test_upvalue_env()
            //     print "hello, upvalue!" -- this `print` is my_print
            Bytecode::get_uptable(0.into(), 0.into(), 0.into()),
            Bytecode::load_constant(1.into(), 1u8.into()),
            Bytecode::call(0.into(), 2.into(), 1.into()),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into(), "hello, upvalue!".into()],
        &[],
        &["_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn block() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local f, g, h
local up1 = 1
do
    local up2 = 2
    do
        local up3 = 3

        -- closure with local variable in block
        f = function()
            up3 = up3 + 1
            print(up3)
        end

        -- closure with local variable out of block
        g = function()
            up2 = up2 + 1
            print(up2)
        end

        -- closure with local variable out of block 2 levels
        h = function()
            up1 = up1 + 1
            print(up1)
        end

        -- call these closures in block
        f()
        g()
        h()
        print(up1, up2, up3)
    end

    -- call these closures out of block
    f()
    g()
    h()
    print(up1, up2)

end

-- call these closures out of block
f()
g()
h()
print(up1)
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0.into()),
            // local f, g, h
            Bytecode::load_nil(0.into(), 2.into()),
            // local up1 = 1
            Bytecode::load_integer(3.into(), 1i8.into()),
            // do
            //     local up2 = 2
            Bytecode::load_integer(4.into(), 2i8.into()),
            //     do
            //         local up3 = 3
            Bytecode::load_integer(5.into(), 3i8.into()),
            //         f = function()
            Bytecode::closure(0.into(), 0u8.into()), // This is optimized compared to luac
            //         g = function()
            Bytecode::closure(1.into(), 1u8.into()), // This is optimized compared to luac
            //         h = function()
            Bytecode::closure(2.into(), 2u8.into()), // This is optimized compared to luac
            //         f()
            Bytecode::move_bytecode(6.into(), 0.into()),
            Bytecode::call(6.into(), 1.into(), 1.into()),
            //         g()
            Bytecode::move_bytecode(6.into(), 1.into()),
            Bytecode::call(6.into(), 1.into(), 1.into()),
            //         h()
            Bytecode::move_bytecode(6.into(), 2.into()),
            Bytecode::call(6.into(), 1.into(), 1.into()),
            //         print(up1, up2, up3)
            Bytecode::get_uptable(6.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(7.into(), 3.into()),
            Bytecode::move_bytecode(8.into(), 4.into()),
            Bytecode::move_bytecode(9.into(), 5.into()),
            Bytecode::call(6.into(), 4.into(), 1.into()),
            //     end
            Bytecode::close(5.into()),
            //     f()
            Bytecode::move_bytecode(5.into(), 0.into()),
            Bytecode::call(5.into(), 1.into(), 1.into()),
            //     g()
            Bytecode::move_bytecode(5.into(), 1.into()),
            Bytecode::call(5.into(), 1.into(), 1.into()),
            //     h()
            Bytecode::move_bytecode(5.into(), 2.into()),
            Bytecode::call(5.into(), 1.into(), 1.into()),
            //     print(up1, up2)
            Bytecode::get_uptable(5.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(6.into(), 3.into()),
            Bytecode::move_bytecode(7.into(), 4.into()),
            Bytecode::call(5.into(), 3.into(), 1.into()),
            // end
            Bytecode::close(4.into()),
            // f()
            Bytecode::move_bytecode(4.into(), 0.into()),
            Bytecode::call(4.into(), 1.into(), 1.into()),
            // g()
            Bytecode::move_bytecode(4.into(), 1.into()),
            Bytecode::call(4.into(), 1.into(), 1.into()),
            // h()
            Bytecode::move_bytecode(4.into(), 2.into()),
            Bytecode::call(4.into(), 1.into(), 1.into()),
            // print(up1)
            Bytecode::get_uptable(4.into(), 0.into(), 0.into()),
            Bytecode::move_bytecode(5.into(), 3.into()),
            Bytecode::call(4.into(), 2.into(), 1.into()),
            // EOF
            Bytecode::return_bytecode(4.into(), 1.into(), 1.into()),
        ],
        &["print".into()],
        &[
            Local::new("f".into(), 3, 42),
            Local::new("g".into(), 3, 42),
            Local::new("h".into(), 3, 42),
            Local::new("up1".into(), 4, 42),
            Local::new("up2".into(), 5, 31),
            Local::new("up3".into(), 6, 20),
        ],
        &["_ENV".into()],
        3,
    );

    crate::Lua::run_program(program).expect("Should run");
}
