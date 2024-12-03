use core::cell::RefCell;

use alloc::{format, rc::Rc};

use crate::{ext::FloatExt, table::Table, value::Value, Lua, Program};

use super::Error;

#[derive(Debug, PartialEq)]
pub enum ByteCode {
    /// `MOVE`  
    /// Moves a value from one location on the stack to another
    ///
    /// `dst`: Location on the stack to store the value  
    /// `src`: Location on the stack to load the value
    Move(u8, u8),
    /// `LOADI`  
    /// Loads a `integer` into the stack
    ///
    /// `dst`: Location on the stack to place integer  
    /// `value`: Integer value to load into stack, this is limited
    /// to a i16
    LoadInt(u8, i16),
    /// `LOADF`  
    /// Loads a `float` into the stack
    ///
    /// `dst`: Location on the stack to place integer  
    /// `value`: Float value to load into stack, this is limited
    /// to a whole floats that can be expressed as a i16
    LoadFloat(u8, i16),
    /// `LOADK`  
    /// Loads the value of a constant into the stack
    ///
    /// `dst`: Location on the stack to place constant  
    /// `src`: Location of the value on `constants` to load into stack
    LoadConstant(u8, u8),
    /// `LOADFALSE`  
    /// Loads a `false` value into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    LoadFalse(u8),
    /// `LOADTRUE`  
    /// Loads a `false` value into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    LoadTrue(u8),
    /// `LOADNIL`  
    /// Loads a `nil` into the stack
    ///
    /// `dst`: Location on the stack to place nil  
    LoadNil(u8),
    /// `GETTABUP`?  
    /// Gets a value from `globals` and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `name`: Location on `constants` where the name of the
    /// global resides
    GetGlobal(u8, u8),
    /// `SETTABUP`?  
    /// Sets a value a global with a value from the stack
    ///
    /// `name`: Location on `constants` where the name of the
    /// global resides  
    /// `src`: Location of the value on stack to set the global
    SetGlobal(u8, u8),
    /// `GETTABLE`?  
    /// Loads a table field to the stack using a stack value
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `src`: Location of the name on the stack  
    GetTable(u8, u8, u8),
    /// `GETI`?  
    /// Loads a value from the table into the stack using integer index
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `index`: Index of the item to load
    GetInt(u8, u8, u8),
    /// `GETFIELD`?  
    /// Loads a table field to the stack using a name
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    GetField(u8, u8, u8),
    /// `SETTABLE`?  
    /// Sets a table field to a value using a value
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location on the stack of the value that will be used
    /// as key  
    /// `value`: Location of the value on the stack
    SetTable(u8, u8, u8),
    /// `SETFIELD`?  
    /// Sets a table field to a value using a name
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    /// `value`: Location of the value on the stack
    SetField(u8, u8, u8),
    /// `NEWTABLE`  
    /// Creates a new table value
    ///
    /// `dst`: Location on the stack to store the table  
    /// `array_len`: Amount of items to allocate on the list  
    /// `table_len`: Amount of items to allocate for the map
    NewTable(u8, u8, u8),
    /// `ADDI`  
    /// Performs arithmetic addition.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `int`: Integer value to add
    AddInteger(u8, u8, u8),
    /// `ADDK`  
    /// Performs arithmetic addition.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    AddConstant(u8, u8, u8),
    /// `ADD`  
    /// Performs arithmetic addition.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Add(u8, u8, u8),
    /// `SUB`  
    /// Performs arithmetic subtraction.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Sub(u8, u8, u8),
    /// `MUL`  
    /// Performs arithmetic multiplication.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Mul(u8, u8, u8),
    /// `MOD`  
    /// Performs arithmetic modulus.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Mod(u8, u8, u8),
    /// `POW`  
    /// Performs arithmetic power.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Pow(u8, u8, u8),
    /// `DIV`  
    /// Performs arithmetic division.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Div(u8, u8, u8),
    /// `IDIV`  
    /// Performs arithmetic whole division.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Idiv(u8, u8, u8),
    /// `BAND`  
    /// Performs bitwise `and`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    BitAnd(u8, u8, u8),
    /// `BOR`  
    /// Performs bitwise `or`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    BitOr(u8, u8, u8),
    /// `BXOR`  
    /// Performs bitwise `xor`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    BitXor(u8, u8, u8),
    /// `SHL`  
    /// Performs bitwise shift left.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    ShiftL(u8, u8, u8),
    /// `SHR`  
    /// Performs bitwise shift right.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    ShiftR(u8, u8, u8),
    /// `UNM`  
    /// Performs negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    Neg(u8, u8),
    /// `BNOT`
    /// Performs bit-wise not.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    BitNot(u8, u8),
    /// `NOT`  
    /// Performs logical negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    Not(u8, u8),
    /// `LEN`  
    /// Performs length calculation on String.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    Len(u8, u8),
    /// `CONCAT`  
    /// Performs concatenation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    Concat(u8, u8, u8),
    /// `JMP`  
    /// Performs jump.
    ///
    /// `jump`: Number of intructions to jump
    Jmp(i16),
    /// `TEST`  
    /// Performs test.
    ///
    /// `src`: Location on stack of the value that if going to be tested  
    /// `test`: Test to perform  
    Test(u8, u8),
    /// `CALL`  
    /// Calls a function
    ///
    /// `func`: Location on the stack where the function was loaded  
    /// `args`: Count of arguments
    Call(u8, u8),
    /// `SETLIST`  
    /// Stores multiple values from the stack into the table
    ///
    /// `table`: Location of the table on the stack  
    /// `array_len`: Number of items on the stack to store
    SetList(u8, u8),
    /// Sets a value from a global with a constant value
    ///
    /// `name`: Location on `constants` where the name of the
    /// global resides  
    /// `src`: Location of the value on `constants` to set the global
    SetGlobalConstant(u8, u8),
    /// Sets a value from a global with a integer
    ///
    /// `name`: Location on `constants` where the name of the
    /// global resides  
    /// `value`: Integer to store on global
    SetGlobalInteger(u8, i16),
    /// Sets a value of a global with a value from another global
    ///
    /// `dst_name`: Location on `constants` where the name of the
    /// destination global resides  
    /// `src_name`: Location on `constants` where the name of the
    /// source global resides
    SetGlobalGlobal(u8, u8),
}

macro_rules! validate_bytecode {
    ($got:expr, $expected:pat) => {
        let $expected = $got else {
            unreachable!(
                "This should never be called with a ByteCode other than {}, but got {:?}.",
                stringify!($expected),
                $got
            )
        };
    };
}

impl ByteCode {
    pub fn move_bytecode(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Move(dst, src));

        vm.set_stack(*dst, vm.stack[*src as usize].clone())
    }

    pub fn load_int(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadInt(dst, value));

        vm.set_stack(*dst, Value::Integer(i64::from(*value)))
    }

    pub fn load_float(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadFloat(dst, value));

        vm.set_stack(*dst, Value::Float(*value as f64))
    }

    pub fn load_constant(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadConstant(dst, key));

        vm.set_stack(*dst, program.constants[*key as usize].clone())
    }

    pub fn load_false(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadFalse(dst));

        vm.set_stack(*dst, Value::Boolean(false))
    }

    pub fn load_true(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadTrue(dst));

        vm.set_stack(*dst, Value::Boolean(true))
    }

    pub fn load_nil(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadNil(dst));

        vm.set_stack(*dst, Value::Nil)
    }

    pub fn get_global(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetGlobal(dst, name));

        let key = &program.constants[*name as usize];
        if let Some(index) = vm.globals.iter().position(|global| global.0.eq(key)) {
            vm.set_stack(*dst, vm.globals[index].1.clone())
        } else {
            vm.set_stack(*dst, Value::Nil)
        }
    }

    pub fn set_global(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetGlobal(name, src));

        let key = &program.constants[*name as usize];
        let value = vm.stack[*src as usize].clone();
        if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
            global.1 = value;
            Ok(())
        } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
            vm.globals.push((key.clone(), value));
            Ok(())
        } else {
            Err(Error::ExpectedName)
        }
    }

    pub fn get_table(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetTable(dst, table, src));

        if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
            let key = &vm.stack[usize::from(*src)];
            let bin_search = (*table).borrow().table.binary_search_by_key(&key, |a| &a.0);

            let value = match bin_search {
                Ok(i) => (*table).borrow().table[i].1.clone(),
                Err(_) => Value::Nil,
            };
            vm.set_stack(*dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn get_int(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetInt(dst, table, index));

        if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
            let value = if index == &0 {
                let bin_search = (*table)
                    .borrow()
                    .table
                    .binary_search_by_key(&&Value::Integer(0), |a| &a.0);
                match bin_search {
                    Ok(i) => (*table).borrow().table[i].1.clone(),
                    Err(_) => Value::Nil,
                }
            } else {
                (*table).borrow().array[usize::from(*index) - 1].clone()
            };
            vm.set_stack(*dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn get_field(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetField(dst, table, key));

        if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
            let key = &program.constants[usize::from(*key)];
            let bin_search = (*table).borrow().table.binary_search_by_key(&key, |a| &a.0);

            let value = match bin_search {
                Ok(i) => (*table).borrow().table[i].1.clone(),
                Err(_) => Value::Nil,
            };
            vm.set_stack(*dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn set_table(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetTable(table, key, value));

        if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
            let key = vm.stack[usize::from(*key)].clone();
            let value = vm.stack[usize::from(*value)].clone();

            let binary_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);
            match binary_search {
                Ok(i) => {
                    let mut table_borrow = table.borrow_mut();
                    let Some(table_value) = table_borrow.table.get_mut(i) else {
                        unreachable!("Already tested existence of table value");
                    };
                    table_value.1 = value;
                }
                Err(i) => table.borrow_mut().table.insert(i, (key, value)),
            }
            Ok(())
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn set_field(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetField(table, key, value));

        if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
            let key = program.constants[usize::from(*key)].clone();
            let value = vm.stack[usize::from(*value)].clone();

            let binary_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);
            match binary_search {
                Ok(i) => {
                    let mut table_borrow = table.borrow_mut();
                    let Some(table_value) = table_borrow.table.get_mut(i) else {
                        unreachable!("Already tested existence of table value");
                    };
                    table_value.1 = value;
                }
                Err(i) => table.borrow_mut().table.insert(i, (key, value)),
            }
            Ok(())
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn new_table(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(
            self,
            ByteCode::NewTable(dst, array_initial_size, table_initial_size)
        );

        vm.set_stack(
            *dst,
            Value::Table(Rc::new(RefCell::new(Table::new(
                usize::from(*array_initial_size),
                usize::from(*table_initial_size),
            )))),
        )
    }

    pub fn add_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::AddInteger(dst, lhs, int));

        todo!("AddInteger")
    }

    pub fn add_constant(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::AddConstant(dst, lhs, constant));

        todo!("AddInteger")
    }

    pub fn add(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Add(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 + r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l + *r as f64),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn sub(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Sub(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 - r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l - *r as f64),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn mul(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Mul(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 * r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l * *r as f64),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn mod_bytecode(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Mod(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l % r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l % r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 % r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l % *r as f64),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn pow(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Pow(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Float((*l as f64).powf(*r as f64)),
            (Value::Float(l), Value::Float(r)) => Value::Float(l.powf(*r)),
            (Value::Integer(l), Value::Float(r)) => Value::Float((*l as f64).powf(*r)),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l.powf(*r as f64)),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn div(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Div(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Float(*l as f64 / *r as f64),
            (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 / r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l / *r as f64),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn idiv(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Idiv(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l / r),
            (Value::Float(l), Value::Float(r)) => Value::Float((l / r).trunc()),
            (Value::Integer(l), Value::Float(r)) => Value::Float((*l as f64 / r).trunc()),
            (Value::Float(l), Value::Integer(r)) => Value::Float((l / *r as f64).trunc()),
            (Value::Nil, _) => return Err(Error::NilArithmetic),
            (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringArithmetic),
            (Value::Table(_), _) => return Err(Error::TableArithmetic),
            (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
            (_, Value::Nil) => return Err(Error::NilArithmetic),
            (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringArithmetic),
            (_, Value::Table(_)) => return Err(Error::TableArithmetic),
            (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
        };
        vm.set_stack(*dst, res)
    }

    pub fn bit_and(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitAnd(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l & r),
            (Value::Float(_), _) => return Err(Error::FloatBitwise),
            (Value::Nil, _) => return Err(Error::NilBitwise),
            (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringBitwise),
            (Value::Table(_), _) => return Err(Error::TableBitwise),
            (Value::Function(_), _) => return Err(Error::FunctionBitwise),
            (_, Value::Float(_)) => return Err(Error::FloatBitwise),
            (_, Value::Nil) => return Err(Error::NilBitwise),
            (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringBitwise),
            (_, Value::Table(_)) => return Err(Error::TableBitwise),
            (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
        };
        vm.set_stack(*dst, res)
    }

    pub fn bit_or(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitOr(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l | r),
            (Value::Float(_), _) => return Err(Error::FloatBitwise),
            (Value::Nil, _) => return Err(Error::NilBitwise),
            (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringBitwise),
            (Value::Table(_), _) => return Err(Error::TableBitwise),
            (Value::Function(_), _) => return Err(Error::FunctionBitwise),
            (_, Value::Float(_)) => return Err(Error::FloatBitwise),
            (_, Value::Nil) => return Err(Error::NilBitwise),
            (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringBitwise),
            (_, Value::Table(_)) => return Err(Error::TableBitwise),
            (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
        };
        vm.set_stack(*dst, res)
    }

    pub fn bit_xor(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitXor(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l ^ r),
            (Value::Float(_), _) => return Err(Error::FloatBitwise),
            (Value::Nil, _) => return Err(Error::NilBitwise),
            (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringBitwise),
            (Value::Table(_), _) => return Err(Error::TableBitwise),
            (Value::Function(_), _) => return Err(Error::FunctionBitwise),
            (_, Value::Float(_)) => return Err(Error::FloatBitwise),
            (_, Value::Nil) => return Err(Error::NilBitwise),
            (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringBitwise),
            (_, Value::Table(_)) => return Err(Error::TableBitwise),
            (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
        };
        vm.set_stack(*dst, res)
    }

    pub fn shiftl(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ShiftL(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l << r),
            (Value::Float(_), _) => return Err(Error::FloatBitwise),
            (Value::Nil, _) => return Err(Error::NilBitwise),
            (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringBitwise),
            (Value::Table(_), _) => return Err(Error::TableBitwise),
            (Value::Function(_), _) => return Err(Error::FunctionBitwise),
            (_, Value::Float(_)) => return Err(Error::FloatBitwise),
            (_, Value::Nil) => return Err(Error::NilBitwise),
            (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringBitwise),
            (_, Value::Table(_)) => return Err(Error::TableBitwise),
            (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
        };
        vm.set_stack(*dst, res)
    }

    pub fn shiftr(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ShiftR(dst, lhs, rhs));

        let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l >> r),
            (Value::Float(l), Value::Float(r)) => {
                if l.zero_frac() && r.zero_frac() {
                    Value::Integer((*l as i64) >> (*r as i64))
                } else {
                    return Err(Error::FloatBitwise);
                }
            }
            (Value::Float(l), Value::Integer(r)) => {
                if l.zero_frac() {
                    Value::Integer((*l as i64) >> r)
                } else {
                    return Err(Error::FloatBitwise);
                }
            }
            (Value::Integer(l), Value::Float(r)) => {
                if r.zero_frac() {
                    Value::Integer(l >> (*r as i64))
                } else {
                    return Err(Error::FloatBitwise);
                }
            }
            (Value::Nil, _) => return Err(Error::NilBitwise),
            (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
            (Value::String(_) | Value::ShortString(_), _) => return Err(Error::StringBitwise),
            (Value::Table(_), _) => return Err(Error::TableBitwise),
            (Value::Function(_), _) => return Err(Error::FunctionBitwise),
            (_, Value::Float(_)) => return Err(Error::FloatBitwise),
            (_, Value::Nil) => return Err(Error::NilBitwise),
            (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
            (_, Value::String(_) | Value::ShortString(_)) => return Err(Error::StringBitwise),
            (_, Value::Table(_)) => return Err(Error::TableBitwise),
            (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
        };
        vm.set_stack(*dst, res)
    }

    pub fn neg(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Neg(dst, rhs));

        let value = match vm.stack[usize::from(*rhs)] {
            Value::Integer(integer) => Value::Integer(-integer),
            Value::Float(float) => Value::Float(-float),
            _ => return Err(Error::InvalidNegOperand),
        };
        vm.set_stack(*dst, value)
    }

    pub fn bit_not(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitNot(dst, rhs));

        let value = match vm.stack[usize::from(*rhs)] {
            Value::Integer(integer) => Value::Integer(!integer),
            _ => return Err(Error::InvalidBitNotOperand),
        };
        vm.set_stack(*dst, value)
    }

    pub fn not(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Not(dst, rhs));

        let value = match &vm.stack[usize::from(*rhs)] {
            Value::Boolean(false) | Value::Nil => Value::Boolean(true),
            _ => Value::Boolean(false),
        };
        vm.set_stack(*dst, value)
    }

    pub fn len(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Len(dst, rhs));

        let value = match &vm.stack[usize::from(*rhs)] {
            Value::String(string) => Value::Integer(i64::try_from(string.len())?),
            Value::ShortString(string) => Value::Integer(i64::try_from(string.len())?),
            _ => return Err(Error::InvalidLenOperand),
        };
        vm.set_stack(*dst, value)
    }

    pub fn concat(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Concat(dst, lhs, rhs));

        let value = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
            (Value::Nil, _) => return Err(Error::NilConcat),
            (Value::Boolean(_), _) => return Err(Error::BoolConcat),
            (Value::Table(_), _) => return Err(Error::TableConcat),
            (Value::Function(_), _) => return Err(Error::FunctionConcat),
            (_, Value::Nil) => return Err(Error::NilConcat),
            (_, Value::Boolean(_)) => return Err(Error::BoolConcat),
            (_, Value::Table(_)) => return Err(Error::TableConcat),
            (_, Value::Function(_)) => return Err(Error::FunctionConcat),
            (Value::Float(lhs), Value::Float(rhs)) => format!("{:?}{:?}", lhs, rhs).as_str().into(),
            (Value::Float(lhs), rhs) => format!("{:?}{}", lhs, rhs).as_str().into(),
            (lhs, Value::Float(rhs)) => format!("{}{:?}", lhs, rhs).as_str().into(),
            (lhs, rhs) => format!("{}{}", lhs, rhs).as_str().into(),
        };
        vm.set_stack(*dst, value)
    }

    pub fn jmp(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Jmp(jump));

        if jump.is_negative() {
            vm.program_counter -= isize::from(*jump).unsigned_abs();
        } else {
            vm.program_counter += usize::try_from(*jump)?;
        }

        Ok(())
    }

    pub fn test(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Test(src, test));

        let cond = &vm.stack[*src as usize];
        match (cond, test) {
            (Value::Nil | Value::Boolean(false), 0) => (),
            (Value::Nil | Value::Boolean(false), 1) => vm.program_counter += 1,
            (_, 1) => (),
            _ => vm.program_counter += 1,
        };

        Ok(())
    }

    pub fn call(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Call(func, _args));

        vm.func_index = *func as usize;
        let func = &vm.stack[vm.func_index];
        if let Value::Function(f) = func {
            f(vm);
            Ok(())
        } else {
            Err(Error::InvalidFunction(func.clone()))
        }
    }

    pub fn set_list(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetList(table, count));

        let table_items_start = usize::from(*table) + 1;
        if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
            let values = vm
                .stack
                .drain(table_items_start..(table_items_start + usize::from(*count)));
            table.borrow_mut().array.extend(values);
            Ok(())
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn set_global_constant(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetGlobalConstant(name, src));

        let key = &program.constants[*name as usize];
        let value = program.constants[*src as usize].clone();
        if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
            global.1 = value;
            Ok(())
        } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
            vm.globals.push((key.clone(), value));
            Ok(())
        } else {
            Err(Error::ExpectedName)
        }
    }

    pub fn set_global_integer(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetGlobalInteger(name, value));

        let key = &program.constants[*name as usize];
        let value = (*value).into();
        if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
            global.1 = value;
            Ok(())
        } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
            vm.globals.push((key.clone(), value));
            Ok(())
        } else {
            Err(Error::ExpectedName)
        }
    }

    pub fn set_global_global(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetGlobalGlobal(dst_name, src_name));

        let dst_key = &program.constants[*dst_name as usize];
        let src_key = &program.constants[*src_name as usize];
        let value = vm
            .globals
            .iter()
            .find(|global| global.0.eq(src_key))
            .map_or(Value::Nil, |global| global.1.clone());
        if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(dst_key)) {
            global.1 = value;
            Ok(())
        } else if matches!(dst_key, Value::String(_) | Value::ShortString(_)) {
            vm.globals.push((dst_key.clone(), value));
            Ok(())
        } else {
            Err(Error::ExpectedName)
        }
    }
}
