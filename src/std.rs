use crate::Lua;

pub fn lib_print(vm: &mut Lua) -> i32 {
    log::info!(target: "no_deps_lua::vm", "{:?}", vm.get_stack(u8::try_from(vm.func_index + 1).unwrap()));
    0
}
