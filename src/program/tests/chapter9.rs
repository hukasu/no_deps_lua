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
            Bytecode::variadic_arguments_prepare(0),
            // g1, g2 = 1, 2
            Bytecode::load_integer(0, 1),
            Bytecode::set_uptable(0, 1, 2, 1),
            Bytecode::set_uptable(0, 0, 0, 0),
            // local up1, up2, up3, up4 = 11, 12, 13, 14
            Bytecode::load_integer(0, 11),
            Bytecode::load_integer(1, 12),
            Bytecode::load_integer(2, 13),
            Bytecode::load_integer(3, 14),
            // local print = print
            Bytecode::get_uptable(4, 0, 3),
            // local function foo()
            Bytecode::closure(5, 0),
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
            Bytecode::load_integer(0, 101),
            Bytecode::load_integer(1, 102),
            //     l1, g1 = g2, l2
            Bytecode::get_uptable(2, 0, 1),
            Bytecode::set_uptable(0, 0, 1, 0),
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
            Bytecode::set_upvalue(2, 4),
            Bytecode::set_uptable(0, 0, 3, 0),
            Bytecode::move_bytecode(0, 2),
            //     print(l1, g1, up1)
            Bytecode::get_upvalue(2, 1),
            Bytecode::move_bytecode(3, 0),
            Bytecode::get_uptable(4, 0, 0),
            Bytecode::get_upvalue(5, 2),
            Bytecode::call(2, 4, 1),
            //     local inner = function()
            Bytecode::closure(2, 0),
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
            Bytecode::load_integer(0, 101),
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
            Bytecode::closure(0, 0),
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
            Bytecode::load_integer(0, 0),
            //     return function()
            Bytecode::closure(1, 0),
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
            Bytecode::set_upvalue(1, 0),
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
