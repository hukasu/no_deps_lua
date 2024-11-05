use log::info;

use crate::Lua;

pub fn lib_print(vm: &mut Lua) -> i32 {
    info!(target: "lua_vm", "{:?}", vm.stack[1]);
    0
}
