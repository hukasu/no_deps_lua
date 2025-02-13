use alloc::{string::ToString, vec::Vec};

use crate::Lua;

pub fn lib_print(vm: &mut Lua) -> i32 {
    let args_start = vm.get_return_stack();
    let print_string = vm.stack[args_start..]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");

    log::info!(target: "no_deps_lua::vm", "{}", print_string);
    0
}

pub fn lib_type(vm: &mut Lua) -> i32 {
    let type_name = vm.get_stack(0).unwrap();
    vm.set_stack(0, type_name.static_type_name().into())
        .unwrap();

    1
}
