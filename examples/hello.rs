use no_deps_lua::{Lua, Program};
use simplelog::{Config, SimpleLogger};

const PROGRAM: &str = r#"
print "hello, world!"
print "hello, again..."
"#;

fn main() {
    SimpleLogger::init(log::LevelFilter::Info, Config::default()).unwrap();

    let program = Program::parse(PROGRAM.as_bytes()).unwrap();
    let mut vm = Lua::new();
    vm.execute(&program).unwrap();
}
