use core::{cell::RefCell, cmp::Ordering};

use alloc::{format, rc::Rc};

use crate::{
    table::Table,
    value::{Value, ValueKey},
    Lua, Program,
};

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
    /// `LFALSESKIP`  
    /// Loads a `false` value into the stack and skips next instruction
    ///
    /// `dst`: Location on the stack to place boolean  
    LoadFalseSkip(u8),
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
    /// Performs arithmetic addition with an integer.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `int`: Integer value to add
    AddInteger(u8, u8, u8),
    /// `ADDK`  
    /// Performs arithmetic addition with a constant.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    AddConstant(u8, u8, u8),
    /// `MULK`  
    /// Performs arithmetic multiplication with a constant.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    MulConstant(u8, u8, u8),
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
    /// `LT`  
    /// Performs less than (<) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    LessThan(u8, u8, u8),
    /// `LE`  
    /// Performs less than or equal (<=) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    LessEqual(u8, u8, u8),
    /// `EQK`
    /// Peforms equal comparison (==) between the register and constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `constant`: Id of constant  
    /// `test`: If it should test for `true` or `false`
    EqualConstant(u8, u8, u8),
    /// `GTI`
    /// Peforms a greater than (>) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    GreaterThanInteger(u8, i8, u8),
    /// `GEI`
    /// Peforms a greater or equal (>=) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    GreaterEqualInteger(u8, i8, u8),
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
    /// `RETURN0`  
    /// Returns from function
    Return,
    /// `RETURN0`  
    /// Returns from function with 0 out values
    ZeroReturn,
    /// `RETURN1`  
    /// Returns from function with 1 out values
    OneReturn,
    /// `FORLOOP`  
    /// Increment counter and jumps back to start of loop
    ///
    /// `for`: Location on the stack counter information is stored  
    /// `jump`: Number of byte codes to jump to reach start of for block
    ForLoop(u8, u16),
    /// `FORPREP`  
    /// Prepares for loop counter
    ///
    /// `for`: Location on the stack counter information is stored  
    /// `jump`: Number of byte codes to jump to reach end of for loop
    ForPrepare(u8, u16),
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
    /// `CLOSURE`
    /// Puts reference to a local function into the stack
    ///
    /// `dst`: Stack location to store function reference  
    /// `func_id`: Id of function
    Closure(u8, u8),
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

        let value = vm.get_stack(*src)?.clone();
        vm.set_stack(*dst, value)
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

    pub fn load_false_skip(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadFalseSkip(dst));

        vm.program_counter += 1;
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
        let value = vm.get_stack(*src)?.clone();
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

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(vm.get_stack(*src)?.clone());
            let bin_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);

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

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let value = if index == &0 {
                let bin_search = (*table)
                    .borrow()
                    .table
                    .binary_search_by_key(&&ValueKey::from(Value::Integer(0)), |a| &a.0);
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

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(program.constants[usize::from(*key)].clone());
            let bin_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);

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

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(vm.get_stack(*key)?.clone());
            let value = vm.get_stack(*value)?.clone();

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

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(program.constants[usize::from(*key)].clone());
            let value = vm.get_stack(*value)?.clone();

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

        let res = match &vm.get_stack(*lhs)? {
            Value::Integer(l) => Value::Integer(l + i64::from(*int)),
            Value::Float(l) => Value::Float(l + *int as f64),
            lhs => {
                return Err(Error::ArithmeticOperand(
                    "add",
                    lhs.static_type_name(),
                    "integer",
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn add_constant(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::AddConstant(dst, lhs, constant));

        let res = match (&vm.get_stack(*lhs)?, &program.constants[*constant as usize]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 + r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l + *r as f64),
            (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "add",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn mul_constant(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::MulConstant(dst, lhs, constant));

        let res = match (&vm.get_stack(*lhs)?, &program.constants[*constant as usize]) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 * r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l * *r as f64),
            (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "mul",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn add(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Add(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 + r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l + *r as f64),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "add",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn sub(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Sub(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 - r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l - *r as f64),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "sub",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn mul(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Mul(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 * r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l * *r as f64),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "mul",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn mod_bytecode(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Mod(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l % r),
            (Value::Float(l), Value::Float(r)) => Value::Float(l % r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 % r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l % *r as f64),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "mod",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn pow(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Pow(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Float((*l as f64).powf(*r as f64)),
            (Value::Float(l), Value::Float(r)) => Value::Float(l.powf(*r)),
            (Value::Integer(l), Value::Float(r)) => Value::Float((*l as f64).powf(*r)),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l.powf(*r as f64)),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "pow",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn div(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Div(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Float(*l as f64 / *r as f64),
            (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
            (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 / r),
            (Value::Float(l), Value::Integer(r)) => Value::Float(l / *r as f64),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "div",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn idiv(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Idiv(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l / r),
            (Value::Float(l), Value::Float(r)) => Value::Float((l / r).trunc()),
            (Value::Integer(l), Value::Float(r)) => Value::Float((*l as f64 / r).trunc()),
            (Value::Float(l), Value::Integer(r)) => Value::Float((l / *r as f64).trunc()),
            (lhs, rhs) => {
                return Err(Error::ArithmeticOperand(
                    "idiv",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn bit_and(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitAnd(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l & r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "and",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn bit_or(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitOr(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l | r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "or",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn bit_xor(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitXor(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l ^ r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "xor",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn shiftl(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ShiftL(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l << r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "shift left",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn shiftr(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ShiftR(dst, lhs, rhs));

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l >> r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "shift right",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(*dst, res)
    }

    pub fn neg(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Neg(dst, rhs));

        let value = match vm.get_stack(*rhs)? {
            Value::Integer(integer) => Value::Integer(-integer),
            Value::Float(float) => Value::Float(-float),
            _ => return Err(Error::InvalidNegOperand),
        };
        vm.set_stack(*dst, value)
    }

    pub fn bit_not(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::BitNot(dst, rhs));

        let value = match vm.get_stack(*rhs)? {
            Value::Integer(integer) => Value::Integer(!integer),
            _ => return Err(Error::InvalidBitNotOperand),
        };
        vm.set_stack(*dst, value)
    }

    pub fn not(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Not(dst, rhs));

        let value = match &vm.get_stack(*rhs)? {
            Value::Boolean(false) | Value::Nil => Value::Boolean(true),
            _ => Value::Boolean(false),
        };
        vm.set_stack(*dst, value)
    }

    pub fn len(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Len(dst, rhs));

        let value = match &vm.get_stack(*rhs)? {
            Value::String(string) => Value::Integer(i64::try_from(string.len())?),
            Value::ShortString(string) => Value::Integer(i64::try_from(string.len())?),
            _ => return Err(Error::InvalidLenOperand),
        };
        vm.set_stack(*dst, value)
    }

    pub fn concat(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Concat(dst, lhs, rhs));

        let value = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
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

    pub fn less_than(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LessThan(lhs, rhs, test));

        let lhs = vm.get_stack(*lhs)?;
        let rhs = vm.get_stack(*rhs)?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Less, *test == 1)
            .map(|should_advance_pc| {
                if should_advance_pc {
                    vm.program_counter += 1;
                }
            })
    }

    pub fn less_equal(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LessEqual(lhs, rhs, test));

        let lhs = &vm.get_stack(*lhs)?;
        let rhs = &vm.get_stack(*rhs)?;

        Self::relational_comparison(
            lhs,
            rhs,
            |ordering| ordering != Ordering::Greater,
            *test == 1,
        )
        .map(|should_advance_pc| {
            if should_advance_pc {
                vm.program_counter += 1;
            }
        })
    }

    pub fn equal_constant(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::EqualConstant(register, constant, test));

        let lhs = vm.get_stack(*register)?;
        let rhs = &program.constants[*constant as usize];

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Equal, *test == 1)
            .map(|should_advance_pc| {
                if should_advance_pc {
                    vm.program_counter += 1;
                }
            })
    }

    pub fn greater_than_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GreaterThanInteger(register, integer, test));

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering == Ordering::Greater,
            *test == 1,
        )
        .map(|should_advance_pc| {
            if should_advance_pc {
                vm.program_counter += 1;
            }
        })
    }

    pub fn greater_equal_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GreaterEqualInteger(register, integer, test));

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(lhs, &rhs, |ordering| ordering != Ordering::Less, *test == 1)
            .map(|should_advance_pc| {
                if should_advance_pc {
                    vm.program_counter += 1;
                }
            })
    }

    pub fn test(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Test(src, test));

        let cond = vm.get_stack(*src)?;
        match (cond, test) {
            (Value::Nil | Value::Boolean(false), 0) => (),
            (Value::Nil | Value::Boolean(false), 1) => vm.program_counter += 1,
            (_, 1) => (),
            _ => vm.program_counter += 1,
        };

        Ok(())
    }

    pub fn call(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Call(func_index, _args));

        vm.func_index = *func_index as usize;
        let func = vm.get_stack(*func_index)?;
        if let Value::Function(func) = func {
            log::trace!("Calling native function");
            func(vm);
            Ok(())
        } else if let Value::Closure(func) = func {
            log::trace!("Calling closure");
            let f = func.clone();

            let cache_program_counter = core::mem::take(&mut vm.program_counter);
            let stack_size = vm.stack.len();
            let return_stack = usize::from(*func_index) + f.arg_count();

            vm.stack.resize(return_stack + 1, Value::Nil);
            vm.return_stack.push(return_stack - 1);

            vm.run_program(f.program())?;

            vm.return_stack.pop();
            vm.stack.truncate(stack_size - f.arg_count());
            vm.program_counter = cache_program_counter;

            Ok(())
        } else {
            Err(Error::InvalidFunction(func.clone()))
        }
    }

    pub fn zero_return(&self, _vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ZeroReturn);

        Ok(())
    }

    pub fn for_loop(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ForLoop(for_stack, jmp));

        if let Value::Integer(counter) = &mut vm.get_stack_mut(for_stack + 1)? {
            if counter != &0 {
                *counter -= 1;
                ByteCode::Add(for_stack + 3, for_stack + 3, for_stack + 2).add(vm, program)?;
                vm.program_counter -= usize::from(*jmp);
            }
            Ok(())
        } else {
            log::error!("For loop counter should be an Integer.");
            Err(Error::ForZeroStep)
        }
    }

    pub fn for_prepare(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ForPrepare(for_stack, jmp));

        let init = vm.get_stack(*for_stack)?;
        let limit = vm.get_stack(for_stack + 1)?;
        let step = vm.get_stack(for_stack + 2)?;

        if let (&Value::Integer(init), &Value::Integer(step)) = (init, step) {
            if step == 0 {
                return Err(Error::ForZeroStep);
            }

            let count = match limit {
                Value::Integer(i) => (i - init) / step,
                Value::Float(i) => (*i as i64 - init) / step,
                _ => {
                    log::error!("For loop limit can't be converted to Float.");
                    return Err(Error::TryFloatConversion);
                }
            };

            vm.set_stack(for_stack + 1, Value::Integer(count))?;
            vm.set_stack(for_stack + 3, Value::Integer(init))?;
            if count <= 0 {
                vm.program_counter += usize::from(*jmp) + 1;
            }
            Ok(())
        } else {
            let Some(Value::Float(init)) = init.try_float() else {
                log::error!("For loop init can't be converted to Float.");
                return Err(Error::TryFloatConversion);
            };
            let Some(Value::Float(limit)) = limit.try_float() else {
                log::error!("For loop limit can't be converted to Float.");
                return Err(Error::TryFloatConversion);
            };
            let Some(Value::Float(step)) = step.try_float() else {
                log::error!("For loop step can't be converted to Float.");
                return Err(Error::TryFloatConversion);
            };

            let count = ((limit - init) / step).trunc();

            vm.set_stack(*for_stack, Value::Float(init))?;
            vm.set_stack(for_stack + 1, Value::Integer(count as i64))?;
            vm.set_stack(for_stack + 2, Value::Float(step))?;
            vm.set_stack(for_stack + 3, Value::Float(init))?;
            if count <= 0.0 {
                vm.program_counter += usize::from(*jmp) + 1;
            }
            Ok(())
        }
    }

    pub fn set_list(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetList(table, count));

        let table_items_start = usize::from(*table) + 1;
        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
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

    pub fn closure(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Closure(dst, func_id));

        let func = program.functions[usize::from(*func_id)].clone();
        vm.set_stack(*dst, func)
    }

    fn relational_comparison(
        lhs: &Value,
        rhs: &Value,
        ordering_test: fn(Ordering) -> bool,
        test: bool,
    ) -> Result<bool, Error> {
        if let Some(ordering) = lhs.partial_cmp(rhs) {
            Ok(ordering_test(ordering) != test)
        } else {
            Err(Error::RelationalOperand(
                lhs.static_type_name(),
                rhs.static_type_name(),
            ))
        }
    }
}
