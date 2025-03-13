use alloc::{string::ToString, vec::Vec};

use crate::Lua;

pub fn lib_print(vm: &mut Lua) -> i32 {
    let top_stack = vm.get_stack_frame();
    let args_start = top_stack.stack_frame;
    let print_string = vm.stack[args_start..]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");

    log::info!(target: "no_deps_lua::vm", "{}", print_string);
    0
}
