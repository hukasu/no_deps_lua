use core::{cell::RefCell, cmp::Ordering, ops::Deref};

use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    stack_frame::FunctionIndex,
    table::Table,
    value::{Value, ValueKey},
    Function, Lua, Program,
};

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// `extra`: Extra number of `nil`s to load
    LoadNil(u8, u8),
    /// `GETUPVAL`  
    /// Gets a upvalue and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `uptable`: UpTable to collect the field from
    GetUpValue(u8, u8),
    /// `GETTABUP`  
    /// Gets a upvalue and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `upvalue`: UpTable to collect the field from  
    /// `name`: Location on `constants` where the name of the
    /// global resides
    GetUpTable(u8, u8, u8),
    /// `GETTABLE`  
    /// Loads a table field to the stack using a stack value
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `src`: Location of the name on the stack  
    GetTable(u8, u8, u8),
    /// `GETI`  
    /// Loads a value from the table into the stack using integer index
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `index`: Index of the item to load
    GetInt(u8, u8, u8),
    /// `GETFIELD`  
    /// Loads a table field to the stack using a name
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    GetField(u8, u8, u8),
    /// `SETTABUP`  
    /// Sets a value a global with a value from the stack
    ///
    /// `upvalue`: UpTable to store the value at  
    /// `name`: Location on `constants` where the name of the
    /// global resides  
    /// `src`: Location of the value on stack to set the global
    SetUpTable(u8, u8, u8),
    /// `SETTABUPk`  
    /// Sets a value a global with a value from the stack
    ///
    /// `upvalue`: UpTable to store the value at  
    /// `key`: Id of constant key  
    /// `value`: Id of constant value
    SetUpTableConstant(u8, u8, u8),
    /// `SETTABLE`  
    /// Sets a table field to a value using a value
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location on the stack of the value that will be used
    /// as key  
    /// `value`: Location of the value on the stack
    SetTable(u8, u8, u8),
    /// `SETTABLEk`  
    /// Sets a table field to a value using a value
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location on the stack of the value that will be used
    /// as key  
    /// `constant`: Id of `constant`
    SetTableConstant(u8, u8, u8),
    /// `SETFIELD`  
    /// Sets a table field to a value using a name
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    /// `value`: Location of the value on the stack
    SetField(u8, u8, u8),
    /// `SETFIELDk`  
    /// Sets a table field to a value using a name
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    /// `constant`: Id of `constant`
    SetFieldConstant(u8, u8, u8),
    /// `NEWTABLE`  
    /// Creates a new table value
    ///
    /// `dst`: Location on the stack to store the table  
    /// `array_len`: Amount of items to allocate on the list  
    /// `table_len`: Amount of items to allocate for the map
    NewTable(u8, u8, u8),
    /// `SELF`  
    /// Get a method and pass self as the first argument  
    ///
    /// `dst`: Destination of the closure  
    /// `table`: Location of the table on the `stack`     
    /// `key`: Location of the key on `constants`  
    TableSelf(u8, u8, u8),
    /// `ADDI`  
    /// Performs arithmetic addition with an integer.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `integer`: Integer value to add
    AddInteger(u8, u8, i8),
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
    ShiftLeft(u8, u8, u8),
    /// `SHR`  
    /// Performs bitwise shift right.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    ShiftRight(u8, u8, u8),
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
    /// `first`: Location on stack of first string, the result is
    /// stored here  
    /// `string_count`: Number of strings to concat
    Concat(u8, u8),
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
    /// `EQI`
    /// Peforms equal comparison (==) between the register and i8.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant  
    /// `test`: If it should test for `true` or `false`
    EqualInteger(u8, i8, u8),
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
    /// `in`: Number of items on the stack going into the function, the function
    /// itself counts as one item, `0` means a variable number of items   
    /// `out`: How many items coming out of the function should be moved into
    /// the caller stack frame, `0` means all  
    Call(u8, u8, u8),
    /// `TAILCALL`  
    /// Calls a function
    ///
    /// `func`: Location on the stack where the function was loaded  
    /// `args`: Count of arguments  
    /// `variadics`: Number of variadic arguments
    TailCall(u8, u8, u8),
    /// `RETURN0`  
    /// Returns from function
    ///
    /// `register`: First item on stack to return  
    /// `count`: Number of items to return, the actual value is subtracting
    /// it by 1, if zero, return all items on stack  
    /// `var_args`: If `var_args` > 0, the function is has variadic arguments
    /// and the number of fixed arguments is `var_args - 1`
    Return(u8, u8, u8),
    /// `RETURN0`  
    /// Returns from function with 0 out values
    ZeroReturn,
    /// `RETURN1`  
    /// Returns from function with 1 out values
    ///
    /// `return`: Location on stack of the returned value
    OneReturn(u8),
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
    /// `c`: ?
    SetList(u8, u8, u8),
    /// `CLOSURE`
    /// Puts reference to a local function into the stack
    ///
    /// `dst`: Stack location to store function reference  
    /// `func_id`: Id of function
    Closure(u8, u8),
    /// `VARARG`
    /// Collect variable arguments
    ///
    /// `register`: First destination of variable arguments  
    /// `count`: Count of variable arguments, `0` means use all, other values
    /// are subtracted by `2`
    VariadicArguments(u8, u8),
    /// `VARARGPREP`
    /// Prepares the variadic arguments of the closure.
    ///
    /// `fixed`: Number of fixed arguments
    VariadicArgumentPrepare(u8),
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

    pub fn load_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadConstant(dst, key));

        vm.set_stack(
            *dst,
            vm.get_running_closure(main_program).constants[*key as usize].clone(),
        )
    }

    pub fn load_false(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadFalse(dst));

        vm.set_stack(*dst, Value::Boolean(false))
    }

    pub fn load_false_skip(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadFalseSkip(dst));

        vm.jump(1)?;
        vm.set_stack(*dst, Value::Boolean(false))
    }

    pub fn load_true(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadTrue(dst));

        vm.set_stack(*dst, Value::Boolean(true))
    }

    pub fn load_nil(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LoadNil(dst, extras));

        // If `extra` is 0, runs once
        for dst in *dst..=(dst + extras) {
            vm.set_stack(dst, Value::Nil)?;
        }

        Ok(())
    }

    pub fn get_up_value(&self, vm: &mut Lua, _main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetUpValue(dst, uptable));

        let upvalue = vm
            .get_up_table(usize::from(*uptable))
            .cloned()
            .ok_or(Error::UpvalueDoesNotExist)?;

        vm.set_stack(*dst, upvalue)
    }

    pub fn get_up_table(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetUpTable(dst, uptable, name));

        let Value::Table(uptable) = vm
            .get_up_table(usize::from(*uptable))
            .ok_or(Error::UpvalueDoesNotExist)?
        else {
            return Err(Error::ExpectedTable);
        };

        let key = &vm.get_running_closure(main_program).constants[*name as usize];
        let value = uptable.deref().borrow().get(ValueKey(key.clone())).clone();

        vm.set_stack(*dst, value)
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
                (*table)
                    .borrow()
                    .array
                    .get(usize::from(*index) - 1)
                    .cloned()
                    .unwrap_or(Value::Nil)
            };
            vm.set_stack(*dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn get_field(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GetField(dst, table, key));

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(
                vm.get_running_closure(main_program).constants[usize::from(*key)].clone(),
            );
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

    pub fn set_up_table(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetUpTable(uptable, name, src));

        let key = vm.get_running_closure(main_program).constants[*name as usize].clone();
        let value = vm.get_stack(*src)?.clone();

        match vm.get_up_table(usize::from(*uptable)) {
            Some(Value::Table(uptable)) => uptable.borrow_mut().set(ValueKey(key), value),
            Some(_) => Err(Error::ExpectedTable),
            None => Err(Error::UpvalueDoesNotExist),
        }
    }

    pub fn set_up_table_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetUpTableConstant(uptable, name, constant));

        let running_program = vm.get_running_closure(main_program);
        let key = running_program.constants[*name as usize].clone();
        let value = running_program.constants[usize::from(*constant)].clone();

        match vm.get_up_table(usize::from(*uptable)) {
            Some(Value::Table(uptable)) => uptable.borrow_mut().set(ValueKey(key), value),
            Some(_) => Err(Error::ExpectedTable),
            None => Err(Error::UpvalueDoesNotExist),
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

    pub fn set_table_constant(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetTableConstant(table, key, constant));

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(vm.get_stack(*key)?.clone());
            let value = program.constants[usize::from(*constant)].clone();

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

    pub fn set_field(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetField(table, key, value));

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(
                vm.get_running_closure(main_program).constants[usize::from(*key)].clone(),
            );
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

    pub fn set_field_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetFieldConstant(table, key, constant));

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = ValueKey::from(
                vm.get_running_closure(main_program).constants[usize::from(*key)].clone(),
            );
            let value =
                vm.get_running_closure(main_program).constants[usize::from(*constant)].clone();

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

    pub fn table_self(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::TableSelf(dst, table, key));

        if let Value::Table(table) = vm.get_stack(*table).cloned()? {
            vm.set_stack(dst + 1, Value::Table(table.clone()))?;

            let key = ValueKey::from(
                vm.get_running_closure(main_program).constants[usize::from(*key)].clone(),
            );
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

    pub fn add_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::AddConstant(dst, lhs, constant));

        let res = match (
            &vm.get_stack(*lhs)?,
            &vm.get_running_closure(main_program).constants[*constant as usize],
        ) {
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

    pub fn mul_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::MulConstant(dst, lhs, constant));

        let res = match (
            &vm.get_stack(*lhs)?,
            &vm.get_running_closure(main_program).constants[*constant as usize],
        ) {
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
        validate_bytecode!(self, ByteCode::ShiftLeft(dst, lhs, rhs));

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
        validate_bytecode!(self, ByteCode::ShiftRight(dst, lhs, rhs));

        let res = match (
            &vm.get_stack(*lhs)?.clone().try_int(),
            &vm.get_stack(*rhs)?.clone().try_int(),
        ) {
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
        validate_bytecode!(self, ByteCode::Concat(first, count));

        let mut strings = Vec::with_capacity(usize::from(*count));

        for src in *first..(first + count) {
            match &vm.get_stack(src)? {
                Value::Nil => return Err(Error::NilConcat),
                Value::Boolean(_) => return Err(Error::BoolConcat),
                Value::Table(_) => return Err(Error::TableConcat),
                Value::NativeFunction(_) => return Err(Error::FunctionConcat),
                Value::Function(_) => return Err(Error::FunctionConcat),
                Value::Integer(lhs) => strings.push(lhs.to_string()),
                Value::Float(lhs) => strings.push(lhs.to_string()),
                Value::ShortString(lhs) => strings.push(lhs.to_string()),
                Value::String(lhs) => strings.push(lhs.to_string()),
            };
        }

        let concatenated = strings.into_iter().collect::<String>();
        vm.set_stack(*first, concatenated.as_str().into())
    }

    pub fn jmp(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Jmp(jump));

        vm.jump(isize::from(*jump))
    }

    pub fn less_than(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::LessThan(lhs, rhs, test));

        let lhs = vm.get_stack(*lhs)?;
        let rhs = vm.get_stack(*rhs)?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Less, *test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
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
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    pub fn equal_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::EqualConstant(register, constant, test));

        let lhs = vm.get_stack(*register)?;
        let rhs = &vm.get_running_closure(main_program).constants[*constant as usize];

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Equal, *test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    pub fn equal_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::EqualInteger(register, integer, test));

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering == Ordering::Equal,
            *test == 1,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
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
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    pub fn greater_equal_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::GreaterEqualInteger(register, integer, test));

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(lhs, &rhs, |ordering| ordering != Ordering::Less, *test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    pub fn test(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Test(src, test));

        let cond = vm.get_stack(*src)?;
        match (cond, test) {
            (Value::Nil | Value::Boolean(false), 0) => (),
            (Value::Nil | Value::Boolean(false), 1) => vm.jump(1)?,
            (_, 1) => (),
            _ => vm.jump(1)?,
        };

        Ok(())
    }

    pub fn call(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Call(func_index, in_items, out));
        let func_index_usize = usize::from(*func_index);
        let in_items = usize::from(*in_items);
        let out_params = usize::from(*out);

        let func = &vm.get_stack(*func_index)?;
        if let Value::NativeFunction(func) = func {
            Self::run_native_function(vm, func_index_usize, in_items, out_params, *func)
        } else if let Value::Function(func) = func {
            let func = func.clone();
            Self::setup_closure(vm, func_index_usize, in_items, out_params, func.as_ref())
        } else {
            Err(Error::InvalidFunction((*func).clone()))
        }

        // TODO deal with c
    }

    pub fn tail_call(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::TailCall(func_index, args, out_params));
        let func_index_usize = usize::from(*func_index);
        let args = usize::from(*args);
        let out_params = usize::from(*out_params);

        let top_stack = vm.get_stack_frame();
        let tail_start = top_stack.stack_frame + func_index_usize;
        let FunctionIndex::Closure(prev_func_index) = top_stack.function_index else {
            unreachable!("Tailcall should never be called on main.");
        };
        vm.drop_stack_frame(func_index_usize, vm.stack.len() - tail_start);

        let func = &vm.get_stack(u8::try_from(prev_func_index)?)?;
        if let Value::NativeFunction(func) = func {
            Self::run_native_function(vm, prev_func_index, args, out_params, *func)
        } else if let Value::Function(func) = func {
            let func = func.clone();
            Self::setup_closure(vm, prev_func_index, args, out_params, func.as_ref())
        } else {
            Err(Error::InvalidFunction((*func).clone()))
        }
    }

    pub fn return_bytecode(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Return(return_start, count, _));

        vm.drop_stack_frame(usize::from(*return_start), usize::from(count - 1));

        Ok(())
    }

    pub fn zero_return(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ZeroReturn);

        vm.drop_stack_frame(0, 0);

        Ok(())
    }

    pub fn one_return(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::OneReturn(return_loc));

        vm.drop_stack_frame(usize::from(*return_loc), 1);

        Ok(())
    }

    pub fn for_loop(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::ForLoop(for_stack, jmp));

        if let Value::Integer(counter) = &mut vm.get_stack_mut(for_stack + 1)? {
            if counter != &0 {
                *counter -= 1;
                ByteCode::Add(for_stack + 3, for_stack + 3, for_stack + 2).add(vm, program)?;
                vm.jump(-isize::try_from(*jmp)?)?;
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
                vm.jump(isize::try_from(*jmp)? + 1)?;
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
                vm.jump(isize::try_from(*jmp)? + 1)?;
            }
            Ok(())
        }
    }

    pub fn set_list(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::SetList(table, count, _c));

        let top_stack = vm.get_stack_frame();

        let table_items_start =
            top_stack.stack_frame + top_stack.variadic_arguments + usize::from(*table) + 1;
        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let values = if *count == 0 {
                let true_count = vm.stack.len() - table_items_start;
                vm.stack
                    .drain(table_items_start..(table_items_start + true_count))
            } else {
                vm.stack
                    .drain(table_items_start..(table_items_start + usize::from(*count)))
            };

            table.borrow_mut().array.extend(values);
            Ok(())
        } else {
            Err(Error::ExpectedTable)
        }
    }

    pub fn closure(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::Closure(dst, func_id));

        let func = program.functions[usize::from(*func_id)].clone();
        vm.set_stack(*dst, func)
    }

    pub fn variadic_arguments(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        validate_bytecode!(self, ByteCode::VariadicArguments(register, count));

        let top_stack = vm.get_stack_frame();

        let variadics = top_stack.variadic_arguments;

        if *count == 0 {
            let start = top_stack.stack_frame;
            let end = start + variadics;

            let move_vals = vm.stack[start..end].to_vec();

            vm.stack
                .truncate(start + variadics + usize::from(*register));
            vm.stack.extend(move_vals);
        } else {
            let true_count = usize::from(count - 1);
            let start = top_stack.stack_frame;
            let end = start + true_count.min(variadics);

            let move_vals = vm.stack[start..end].to_vec();

            vm.stack
                .truncate(start + variadics + usize::from(*register));
            vm.stack.extend(move_vals);

            if true_count > variadics {
                let remaining = true_count - variadics;
                vm.stack.resize(vm.stack.len() + remaining, Value::Nil);
            }
        }

        Ok(())
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

    fn run_native_function(
        vm: &mut Lua,
        func_index: usize,
        args: usize,
        out_params: usize,
        func: fn(&mut Lua) -> i32,
    ) -> Result<(), Error> {
        log::trace!("Calling native function");

        let top_stack = vm.get_stack_frame();

        let args = if args == 0 {
            vm.stack.len() - (top_stack.stack_frame + top_stack.variadic_arguments + func_index) - 1
        } else {
            args - 1
        };

        vm.prepare_new_function_stack(func_index, args, out_params, 0);

        let returns = usize::try_from(func(vm))?;

        vm.drop_stack_frame(0, returns);

        Ok(())
    }

    fn setup_closure(
        vm: &mut Lua,
        func_index: usize,
        args: usize,
        out_params: usize,
        func: &Function,
    ) -> Result<(), Error> {
        log::trace!("Calling closure");

        let top_stack = vm.get_stack_frame();

        let locals_and_temps_on_function_stack = vm.stack.len()
            - (top_stack.stack_frame + top_stack.variadic_arguments + func_index)
            - 1;

        let (args, var_args) = if args == 0 {
            (
                locals_and_temps_on_function_stack - top_stack.variadic_arguments,
                top_stack.variadic_arguments,
            )
        } else if func.variadic_args() {
            (
                func.arg_count(),
                locals_and_temps_on_function_stack.saturating_sub(func.arg_count()),
            )
        } else {
            (func.arg_count(), 0)
        };

        if args > 0 && var_args > 0 {
            let variadics = vm
                .stack
                .drain((vm.stack.len() - var_args)..)
                .collect::<Vec<_>>();
            let fixed = vm
                .stack
                .drain((vm.stack.len() - args)..)
                .collect::<Vec<_>>();

            vm.stack.extend(variadics);
            vm.stack.extend(fixed);
        }

        vm.prepare_new_function_stack(func_index, args, out_params, var_args);

        Ok(())
    }
}
