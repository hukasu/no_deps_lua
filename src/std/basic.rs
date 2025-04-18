use alloc::{format, string::ToString, vec::Vec};

use crate::{Lua, value::Value};

pub fn lib_print(vm: &mut Lua) -> i32 {
    let top_stack = vm.get_stack_frame();
    let args_start = top_stack.stack_frame;
    let print_string = vm.stack[args_start..]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join("\t");

    log::info!(target: "no_deps_lua::vm", "{}", print_string);
    0
}

pub fn lib_type(vm: &mut crate::Lua) -> i32 {
    let type_name = vm.get_stack(0).unwrap();
    vm.set_stack(0, type_name.static_type_name().into())
        .unwrap();

    1
}

pub fn lib_warn(vm: &mut Lua) -> i32 {
    let switch = match vm.get_upvalue(0) {
        Ok(Value::Boolean(switch)) => switch,
        Ok(other) => {
            log::error!(
                "`lib_warn`'s upvalue should be a boolean, but was {}.",
                other
            );
            return -1;
        }
        Err(_) => {
            log::error!("`lib_warn` did not have a upvalue.");
            return -1;
        }
    };
    let top_stack = vm.get_stack_frame();
    let args_start = top_stack.stack_frame;
    let args = vm.stack[args_start..]
        .iter()
        .map(|val| val.to_string())
        .collect::<Vec<_>>();
    match (args.as_slice(), switch) {
        ([single], false) if single == "@on" => {
            if let Err(err) = vm.set_upvalue(0, true) {
                log::error!("Failed to update `lib_warn`'s upvalue due to `{:?}`.", err);
                return -1;
            }
            log::trace!("Warn logging enabled.");
        }
        ([single], true) if single == "@off" => {
            if let Err(err) = vm.set_upvalue(0, false) {
                log::error!("Failed to update `lib_warn`'s upvalue due to `{:?}`.", err);
                return -1;
            }
            log::trace!("Warn logging disabled.");
        }
        (args, true) => {
            let print_string = args.join("\t");
            log::warn!(target: "no_deps_lua::vm", "{}", print_string);
        }
        (_, false) => (),
    }
    0
}
