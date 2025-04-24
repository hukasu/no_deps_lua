use core::cell::RefCell;

use alloc::{rc::Rc, vec};

use crate::{
    Error, Lua, Program,
    bytecode::Bytecode,
    closure::{Closure, NativeClosureReturn, Upvalue},
    environment::Environment,
    program::Local,
    value::Value,
};

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
            Bytecode::variadic_arguments_prepare(0),
            // g1, g2 = 1, 2
            Bytecode::load_integer(0, 1i8),
            Bytecode::set_uptable(0, 1, 2, true),
            Bytecode::set_uptable(0, 0, 0, false),
            // local up1, up2, up3, up4 = 11, 12, 13, 14
            Bytecode::load_integer(0, 11i8),
            Bytecode::load_integer(1, 12i8),
            Bytecode::load_integer(2, 13i8),
            Bytecode::load_integer(3, 14i8),
            // local print = print
            Bytecode::get_uptable(4, 0, 3),
            // local function foo()
            Bytecode::closure(5, 0u8),
            // foo()
            Bytecode::move_bytecode(6, 5),
            Bytecode::call(6, 1, 1),
            // EOF
            Bytecode::return_bytecode(6, 1, 1),
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
            Bytecode::load_integer(0, 101i8),
            Bytecode::load_integer(1, 102i8),
            //     l1, g1 = g2, l2
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::set_uptable(0, 0, 1, false),
            Bytecode::move_bytecode(0, 2),
            //     print(l1, g1)
            Bytecode::get_upvalue(2, 1),
            Bytecode::move_bytecode(3, 0),
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::call(2, 3, 1),
            //     up1, up2, up3 = l1, g1, up4
            Bytecode::move_bytecode(2, 0),
            Bytecode::get_uptable(3, 0, 0),
            Bytecode::get_upvalue(4, 5),
            Bytecode::set_upvalue(4, 4),
            Bytecode::set_upvalue(3, 3),
            Bytecode::set_upvalue(2, 2),
            //     print(up1, up2, up3)
            Bytecode::get_upvalue(2, 1),
            Bytecode::get_upvalue(3, 2),
            Bytecode::get_upvalue(4, 3),
            Bytecode::get_upvalue(5, 4),
            Bytecode::call(2, 4, 1),
            //     l1, g1, up1 = up2, up3, up4
            Bytecode::get_upvalue(2, 3),
            Bytecode::get_upvalue(3, 4),
            Bytecode::get_upvalue(4, 5),
            Bytecode::set_upvalue(4, 2),
            Bytecode::set_uptable(0, 0, 3, false),
            Bytecode::move_bytecode(0, 2),
            //     print(l1, g1, up1)
            Bytecode::get_upvalue(2, 1),
            Bytecode::move_bytecode(3, 0),
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::get_upvalue(5, 2),
            Bytecode::call(2, 4, 1),
            //     local inner = function()
            Bytecode::closure(2, 0u8),
            //     inner()
            Bytecode::move_bytecode(3, 2),
            Bytecode::call(3, 1, 1),
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
            Bytecode::load_integer(0, 101i8),
            Bytecode::get_uptable(1, 3, 0),
            Bytecode::get_upvalue(2, 4),
            Bytecode::set_upvalue(2, 2),
            Bytecode::set_upvalue(1, 1),
            Bytecode::set_upvalue(0, 0),
            //     print(up1, up2, up3)
            Bytecode::get_upvalue(0, 5),
            Bytecode::get_upvalue(1, 0),
            Bytecode::get_upvalue(2, 1),
            Bytecode::get_upvalue(3, 2),
            Bytecode::call(0, 4, 1),
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
            Bytecode::variadic_arguments_prepare(0),
            // local function factory()
            Bytecode::closure(0, 0u8),
            // local f1 = factory()
            Bytecode::move_bytecode(1, 0),
            Bytecode::call(1, 1, 2),
            // f1()
            Bytecode::move_bytecode(2, 1),
            Bytecode::call(2, 1, 1),
            // f1()
            Bytecode::move_bytecode(2, 1),
            Bytecode::call(2, 1, 1),
            // local f2 = factory()
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(2, 1, 2),
            // f2()
            Bytecode::move_bytecode(3, 2),
            Bytecode::call(3, 1, 1),
            // f1()
            Bytecode::move_bytecode(3, 1),
            Bytecode::call(3, 1, 1),
            // f2()
            Bytecode::move_bytecode(3, 2),
            Bytecode::call(3, 1, 1),
            // f1()
            Bytecode::move_bytecode(3, 1),
            Bytecode::call(3, 1, 1),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
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
            Bytecode::load_integer(0, 0i8),
            //     return function()
            Bytecode::closure(1, 0u8),
            Bytecode::return_bytecode(1, 2, 0),
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
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::get_upvalue(1, 1),
            Bytecode::call(0, 2, 1),
            //     i = i + 1
            Bytecode::get_upvalue(0, 1),
            Bytecode::add_integer(0, 0, 1),
            // MMBINI
            Bytecode::set_upvalue(0, 1),
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
            Bytecode::variadic_arguments_prepare(0),
            // local function my_print(a)
            Bytecode::closure(0, 0u8),
            // local function test_local_env()
            Bytecode::closure(1, 1u8),
            // test_local_env()
            Bytecode::move_bytecode(2, 1),
            Bytecode::call(2, 1, 1),
            // local _ENV = { print = my_print }
            Bytecode::new_table(2, 1, 0),
            // EXTRAARG
            Bytecode::set_field(2, 0, 0, false),
            // local function test_upvalue_env()
            Bytecode::closure(3, 2u8),
            // test_upvalue_env()
            Bytecode::move_bytecode(4, 3),
            Bytecode::call(4, 1, 1),
            // EOF
            Bytecode::return_bytecode(4, 1, 1),
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
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::load_constant(2, 1u8),
            Bytecode::move_bytecode(3, 0),
            Bytecode::call(1, 3, 1),
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
            Bytecode::new_table(0, 1, 0),
            // EXTRAARG
            Bytecode::get_upvalue(1, 0),
            Bytecode::set_field(0, 0, 1, false),
            //     print "hello, world!" -- this `print` is my_print
            Bytecode::get_field(1, 0, 0),
            Bytecode::load_constant(2, 1u8),
            Bytecode::call(1, 2, 1),
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
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::load_constant(1, 1u8),
            Bytecode::call(0, 2, 1),
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
            Bytecode::variadic_arguments_prepare(0),
            // local f, g, h
            Bytecode::load_nil(0, 2),
            // local up1 = 1
            Bytecode::load_integer(3, 1i8),
            // do
            //     local up2 = 2
            Bytecode::load_integer(4, 2i8),
            //     do
            //         local up3 = 3
            Bytecode::load_integer(5, 3i8),
            //         f = function()
            Bytecode::closure(0, 0u8), // This is optimized compared to luac
            //         g = function()
            Bytecode::closure(1, 1u8), // This is optimized compared to luac
            //         h = function()
            Bytecode::closure(2, 2u8), // This is optimized compared to luac
            //         f()
            Bytecode::move_bytecode(6, 0),
            Bytecode::call(6, 1, 1),
            //         g()
            Bytecode::move_bytecode(6, 1),
            Bytecode::call(6, 1, 1),
            //         h()
            Bytecode::move_bytecode(6, 2),
            Bytecode::call(6, 1, 1),
            //         print(up1, up2, up3)
            Bytecode::get_uptable(6, 0, 0),
            Bytecode::move_bytecode(7, 3),
            Bytecode::move_bytecode(8, 4),
            Bytecode::move_bytecode(9, 5),
            Bytecode::call(6, 4, 1),
            //     end
            Bytecode::close(5),
            //     f()
            Bytecode::move_bytecode(5, 0),
            Bytecode::call(5, 1, 1),
            //     g()
            Bytecode::move_bytecode(5, 1),
            Bytecode::call(5, 1, 1),
            //     h()
            Bytecode::move_bytecode(5, 2),
            Bytecode::call(5, 1, 1),
            //     print(up1, up2)
            Bytecode::get_uptable(5, 0, 0),
            Bytecode::move_bytecode(6, 3),
            Bytecode::move_bytecode(7, 4),
            Bytecode::call(5, 3, 1),
            // end
            Bytecode::close(4),
            // f()
            Bytecode::move_bytecode(4, 0),
            Bytecode::call(4, 1, 1),
            // g()
            Bytecode::move_bytecode(4, 1),
            Bytecode::call(4, 1, 1),
            // h()
            Bytecode::move_bytecode(4, 2),
            Bytecode::call(4, 1, 1),
            // print(up1)
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::move_bytecode(5, 3),
            Bytecode::call(4, 2, 1),
            // EOF
            Bytecode::return_bytecode(4, 1, 1),
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

#[test]
fn block_loop() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = Program::parse(
        r#"
local foos = {}
local bars = {}
for i = 1,4 do
    local up = 0
    foos[i] = function()
        up = up + 1
	    return up
    end
    do
        bars[i] = function()
            i = i + 1
            return i
        end
    end
end

print(foos[1](), foos[1](), "|", foos[4](), foos[4]())
print(foos[1](), foos[1](), "|", foos[4](), foos[4]())
print()
print(bars[1](), bars[1](), "|", bars[4](), bars[4]())
print(bars[1](), bars[1](), "|", bars[4](), bars[4]())
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local foos = {}
            Bytecode::new_table(0, 0, 0),
            // TODO EXTRAARG
            // local bars = {}
            Bytecode::new_table(1, 0, 0),
            // TODO EXTRAARG
            // for i = 1,4 do
            Bytecode::load_integer(2, 1i8),
            Bytecode::load_integer(3, 4i8),
            Bytecode::load_integer(4, 1i8),
            Bytecode::for_prepare(2, 7u8),
            //     local up = 0
            Bytecode::load_integer(6, 0i8),
            //     foos[i] = function()
            Bytecode::closure(7, 0u8),
            Bytecode::set_table(0, 5, 7, false),
            //     do
            //         bars[i] = function()
            Bytecode::closure(7, 1u8),
            Bytecode::set_table(1, 5, 7, false),
            //     end
            Bytecode::close(6),
            // end
            Bytecode::close(5),
            Bytecode::for_loop(2, 8u8),
            // print(foos[1](), foos[1](), "|", foos[4](), foos[4]())
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_index(3, 0, 1),
            Bytecode::call(3, 1, 2),
            Bytecode::get_index(4, 0, 1),
            Bytecode::call(4, 1, 2),
            Bytecode::load_constant(5, 1u8),
            Bytecode::get_index(6, 0, 4),
            Bytecode::call(6, 1, 2),
            Bytecode::get_index(7, 0, 4),
            Bytecode::call(7, 1, 0),
            Bytecode::call(2, 0, 1),
            // print(foos[1](), foos[1](), "|", foos[4](), foos[4]())
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_index(3, 0, 1),
            Bytecode::call(3, 1, 2),
            Bytecode::get_index(4, 0, 1),
            Bytecode::call(4, 1, 2),
            Bytecode::load_constant(5, 1u8),
            Bytecode::get_index(6, 0, 4),
            Bytecode::call(6, 1, 2),
            Bytecode::get_index(7, 0, 4),
            Bytecode::call(7, 1, 0),
            Bytecode::call(2, 0, 1),
            // print()
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::call(2, 1, 1),
            // print(bars[1](), bars[1](), "|", bars[4](), bars[4]())
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_index(3, 1, 1),
            Bytecode::call(3, 1, 2),
            Bytecode::get_index(4, 1, 1),
            Bytecode::call(4, 1, 2),
            Bytecode::load_constant(5, 1u8),
            Bytecode::get_index(6, 1, 4),
            Bytecode::call(6, 1, 2),
            Bytecode::get_index(7, 1, 4),
            Bytecode::call(7, 1, 0),
            Bytecode::call(2, 0, 1),
            // print(bars[1](), bars[1](), "|", bars[4](), bars[4]())
            Bytecode::get_uptable(2, 0, 0),
            Bytecode::get_index(3, 1, 1),
            Bytecode::call(3, 1, 2),
            Bytecode::get_index(4, 1, 1),
            Bytecode::call(4, 1, 2),
            Bytecode::load_constant(5, 1u8),
            Bytecode::get_index(6, 1, 4),
            Bytecode::call(6, 1, 2),
            Bytecode::get_index(7, 1, 4),
            Bytecode::call(7, 1, 0),
            Bytecode::call(2, 0, 1),
            // EOF
            Bytecode::return_bytecode(2, 1, 1),
        ],
        &["print".into(), "|".into()],
        &[
            Local::new("foos".into(), 3, 63),
            Local::new("bars".into(), 4, 63),
            Local::new("?for_start".into(), 7, 16),
            Local::new("?for_end".into(), 7, 16),
            Local::new("?for_step".into(), 7, 16),
            Local::new("i".into(), 8, 14),
            Local::new("up".into(), 9, 13),
        ],
        &["_ENV".into()],
        2,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function()
            //     up = up + 1
            Bytecode::get_upvalue(0, 0),
            Bytecode::add_integer(0, 0, 1),
            // TODO MMBINI
            Bytecode::set_upvalue(0, 0),
            //     return up
            Bytecode::get_upvalue(0, 0),
            Bytecode::one_return(0),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[],
        &["up".into()],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // function()
            //     i = i + 1
            Bytecode::get_upvalue(0, 0),
            Bytecode::add_integer(0, 0, 1),
            // TODO MMBINI
            Bytecode::set_upvalue(0, 0),
            //     return i
            Bytecode::get_upvalue(0, 0),
            Bytecode::one_return(0),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[],
        &["i".into()],
        0,
    );

    crate::Lua::run_program(program).expect("Should run");
}

#[test]
fn rust_closure() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = crate::Program::parse(
        r#"
local c1 = new_counter()
local c2 = new_counter()
c1()
c2()
c2()
c1()
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local c1 = new_counter()
            Bytecode::get_uptable(0, 0, 0),
            Bytecode::call(0, 1, 2),
            // local c2 = new_counter()
            Bytecode::get_uptable(1, 0, 0),
            Bytecode::call(1, 1, 2),
            // c1()
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(2, 1, 1),
            // c2()
            Bytecode::move_bytecode(2, 1),
            Bytecode::call(2, 1, 1),
            // c2()
            Bytecode::move_bytecode(2, 1),
            Bytecode::call(2, 1, 1),
            // c1()
            Bytecode::move_bytecode(2, 0),
            Bytecode::call(2, 1, 1),
            // EOF
            Bytecode::return_bytecode(2, 1, 1),
        ],
        &["new_counter".into()],
        &[
            Local::new("c1".into(), 4, 15),
            Local::new("c2".into(), 6, 15),
        ],
        &["_ENV".into()],
        0,
    );

    fn new_counter(vm: &mut Lua) -> NativeClosureReturn {
        let Value::Integer(start_count) = vm.get_upvalue(0)? else {
            return Err(Error::ExpectedTable);
        };
        vm.set_stack(
            0,
            Value::Closure(Rc::new(Closure::new_native(
                count,
                vec![Rc::new(RefCell::new(Upvalue::Closed(Value::Integer(
                    start_count,
                ))))],
            ))),
        )?;
        vm.set_upvalue(0, start_count + 1000)?;
        Ok(1)
    }

    fn count(vm: &mut Lua) -> NativeClosureReturn {
        let Value::Integer(count) = vm.get_upvalue(0)? else {
            return Err(Error::ExpectedTable);
        };
        log::info!("Counted to {}", count);
        vm.set_upvalue(0, count + 1)?;
        Ok(0)
    }

    let mut env = Environment::default();
    env.push(
        "new_counter",
        Value::Closure(Rc::new(Closure::new_native(
            new_counter,
            vec![Rc::new(RefCell::new(Upvalue::Closed(Value::Integer(0))))],
        ))),
    )
    .unwrap();

    crate::Lua::run_program_with_env(program, env).unwrap();
}

#[test]
fn goto() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = crate::Program::parse(
        r#"
local f
local first = true

::again::

-- step.1
if f then -- set after goto
    -- step.4
    f('first')
    f('first')
    f('first')
    first = false
end

-- step.2 and step.5
local i = 0 -- this local variable should be closed on goto
f = function (prefix)
    i = i + 1
    print(prefix, i)
end

if first then
    -- step.3
    goto again -- close `local i`
end

-- step.6
-- `f` is a new closure, with a new `i`
-- so the following calls should print: 1,2,3
f('after')
f('after')
f('after')
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local f
            Bytecode::load_nil(0, 0),
            // local first = true
            Bytecode::load_true(1),
            // if f then
            Bytecode::test(0, false),
            Bytecode::jump(10i8),
            //     f('first')
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_constant(3, 0u8),
            Bytecode::call(2, 2, 1),
            //     f('first')
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_constant(3, 0u8),
            Bytecode::call(2, 2, 1),
            //     f('first')
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_constant(3, 0u8),
            Bytecode::call(2, 2, 1),
            //     first = false
            Bytecode::load_false(1),
            // end
            // local i = 0
            Bytecode::load_integer(2, 0i8),
            // f = function (prefix)
            Bytecode::closure(0, 0u8), // Optimized out one move
            // if first then
            Bytecode::test(1, false),
            Bytecode::jump(2i8),
            //     goto again
            Bytecode::close(2),
            Bytecode::jump(-18i8),
            // end
            // f('after')
            Bytecode::move_bytecode(3, 0),
            Bytecode::load_constant(4, 1u8),
            Bytecode::call(3, 2, 1),
            // f('after')
            Bytecode::move_bytecode(3, 0),
            Bytecode::load_constant(4, 1u8),
            Bytecode::call(3, 2, 1),
            // f('after')
            Bytecode::move_bytecode(3, 0),
            Bytecode::load_constant(4, 1u8),
            Bytecode::call(3, 2, 1),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
        ],
        &["first".into(), "after".into()],
        &[
            Local::new("f".into(), 3, 32),
            Local::new("first".into(), 4, 32),
            Local::new("i".into(), 17, 32),
        ],
        &["_ENV".into()],
        1,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // function (prefix)
            Bytecode::get_upvalue(1, 0),
            //     i = i + 1
            Bytecode::add_integer(1, 1, 1),
            // MMBINI
            Bytecode::set_upvalue(1, 0),
            //     print(prefix, i)
            Bytecode::get_uptable(1, 1, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::get_upvalue(3, 0),
            Bytecode::call(1, 3, 1),
            // end
            Bytecode::zero_return(),
        ],
        &["print".into()],
        // TODO Update when MMBINI is implementend
        &[Local::new("prefix".into(), 1, 9)],
        &["i".into(), "_ENV".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}

#[test]
fn generic_for() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    let program = crate::Program::parse(
        r#"
local function iter(t, i)
    i = i + 1
    local v = t[i]
    if v then
        return i, v
    end
end

local function my_ipairs(t)
    return iter, t, 0
end

local z = {'hello', 123, 456}
for i,v in my_ipairs(z) do
	print(i, v)
end

for i,v in iter,z,0 do
	print(i, v)
end

for i,v in my_ipairs(z) do
	print(i, v)
	i = i+1 -- update ctrl-var during loop
end
"#,
    )
    .unwrap();

    super::compare_program(
        &program,
        &[
            Bytecode::variadic_arguments_prepare(0),
            // local function iter(t, i)
            Bytecode::closure(0, 0u8),
            // local function my_ipairs(t)
            Bytecode::closure(1, 1u8),
            // local z = {'hello', 123, 456}
            Bytecode::new_table(2, 0, 3),
            // EXTRAARG
            Bytecode::load_constant(3, 0u8),
            Bytecode::load_integer(4, 123i16),
            Bytecode::load_integer(5, 456i16),
            Bytecode::set_list(2, 3, 0),
            // for i,v in my_ipairs(z) do
            Bytecode::move_bytecode(3, 1),
            Bytecode::move_bytecode(4, 2),
            Bytecode::call(3, 2, 5),
            Bytecode::generic_for_prepare(3, 4u8),
            // 	print(i, v)
            Bytecode::get_uptable(9, 0, 1),
            Bytecode::move_bytecode(10, 7),
            Bytecode::move_bytecode(11, 8),
            Bytecode::call(9, 3, 1),
            // end
            Bytecode::generic_for_call(3, 2),
            Bytecode::generic_for_loop(3, 6u8),
            Bytecode::close(3),
            // for i,v in iter,z,0 do
            Bytecode::move_bytecode(3, 0),
            Bytecode::move_bytecode(4, 2),
            Bytecode::load_integer(5, 0i8),
            Bytecode::load_nil(6, 0),
            Bytecode::generic_for_prepare(3, 4u8),
            // 	print(i, v)
            Bytecode::get_uptable(9, 0, 1),
            Bytecode::move_bytecode(10, 7),
            Bytecode::move_bytecode(11, 8),
            Bytecode::call(9, 3, 1),
            // end
            Bytecode::generic_for_call(3, 2),
            Bytecode::generic_for_loop(3, 6u8),
            Bytecode::close(3),
            // for i,v in my_ipairs(z) do
            Bytecode::move_bytecode(3, 1),
            Bytecode::move_bytecode(4, 2),
            Bytecode::call(3, 2, 5),
            Bytecode::generic_for_prepare(3, 5u8),
            // 	print(i, v)
            Bytecode::get_uptable(9, 0, 1),
            Bytecode::move_bytecode(10, 7),
            Bytecode::move_bytecode(11, 8),
            Bytecode::call(9, 3, 1),
            // 	i = i+1 -- update ctrl-var during loop
            Bytecode::add_integer(7, 7, 1),
            // MMBINI
            // end
            Bytecode::generic_for_call(3, 2),
            Bytecode::generic_for_loop(3, 7u8),
            Bytecode::close(3),
            // EOF
            Bytecode::return_bytecode(3, 1, 1),
        ],
        &["hello".into(), "print".into()],
        // TODO Update when MMBINI is implementend
        &[
            Local::new("iter".into(), 3, 45),
            Local::new("my_ipairs".into(), 4, 45),
            Local::new("z".into(), 9, 45),
            Local::new("?for_iterator".into(), 12, 19),
            Local::new("?for_state".into(), 12, 19),
            Local::new("?for_control".into(), 12, 19),
            Local::new("?for_closing_value".into(), 12, 19),
            Local::new("i".into(), 13, 17),
            Local::new("v".into(), 13, 17),
            Local::new("?for_iterator".into(), 24, 31),
            Local::new("?for_state".into(), 24, 31),
            Local::new("?for_control".into(), 24, 31),
            Local::new("?for_closing_value".into(), 24, 31),
            Local::new("i".into(), 25, 29),
            Local::new("v".into(), 25, 29),
            Local::new("?for_iterator".into(), 35, 43),
            Local::new("?for_state".into(), 35, 43),
            Local::new("?for_control".into(), 35, 43),
            Local::new("?for_closing_value".into(), 35, 43),
            Local::new("i".into(), 36, 41),
            Local::new("v".into(), 36, 41),
        ],
        &["_ENV".into()],
        2,
    );

    let closure = super::get_closure_program(&program, 0);
    super::compare_program(
        closure,
        &[
            // local function iter(t, i)
            //     i = i + 1
            Bytecode::add_integer(1, 1, 1),
            // MMBINI
            //     local v = t[i]
            Bytecode::get_table(2, 0, 1),
            //     if v then
            Bytecode::test(2, false),
            Bytecode::jump(3i8),
            //         return i, v
            Bytecode::move_bytecode(3, 1),
            Bytecode::move_bytecode(4, 2),
            Bytecode::return_bytecode(3, 3, 0),
            //     end
            // end
            Bytecode::zero_return(),
        ],
        &[],
        // TODO Update when MMBINI is implementend
        &[
            Local::new("t".into(), 1, 9),
            Local::new("i".into(), 1, 9),
            Local::new("v".into(), 3, 9),
        ],
        &[],
        0,
    );

    let closure = super::get_closure_program(&program, 1);
    super::compare_program(
        closure,
        &[
            // local function my_ipairs(t)
            //     return iter, t, 0
            Bytecode::get_upvalue(1, 0),
            Bytecode::move_bytecode(2, 0),
            Bytecode::load_integer(3, 0i8),
            Bytecode::return_bytecode(1, 4, 0),
            // end
            Bytecode::zero_return(),
        ],
        &[],
        &[Local::new("t".into(), 1, 6)],
        &["iter".into()],
        0,
    );

    crate::Lua::run_program(program).unwrap();
}
