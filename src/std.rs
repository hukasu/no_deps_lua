use crate::Lua;

pub fn lib_print(vm: &mut Lua) -> i32 {
    log::info!(target: "no_deps_lua::vm", "{:?}", vm.stack[vm.func_index + 1]);
    0
}
