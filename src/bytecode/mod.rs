mod opcode;

use core::{
    cell::RefCell,
    cmp::Ordering,
    fmt::{Debug, Display},
    ops::Deref,
};

use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    function::Function,
    stack_frame::FunctionIndex,
    table::Table,
    value::{Value, ValueKey},
    Lua, Program,
};

use super::Error;

pub use self::opcode::OpCode;

#[derive(Clone, Copy, PartialEq)]
pub struct Bytecode {
    bytecode: u32,
    function: BytecodeFunction,
}

type BytecodeFunction =
    fn(bytecode: &Bytecode, vm: &mut Lua, program: &Program) -> Result<(), Error>;

const A_MASK: u32 = 0x7f80;
const A_SHIFT: u32 = 7;
const B_MASK: u32 = 0xff0000;
const B_SHIFT: u32 = 16;
const C_MASK: u32 = 0xff000000;
const C_SHIFT: u32 = 24;
const K_MASK: u32 = 0x8000;
const K_SHIFT: u32 = 15;

const BX_MAX: u32 = (1 << 17) - 1;
const BX_MASK: u32 = 0xffff8000;
const BX_SHIFT: u32 = 15;

const J_MAX: u32 = (1 << 25) - 1;
const J_SHIFT: u32 = A_SHIFT;

const I8_OFFSET: u8 = u8::MAX >> 1;
const I17_OFFSET: u32 = BX_MAX >> 1;
const I25_OFFSET: u32 = J_MAX >> 1;

impl Bytecode {
    pub fn execute(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        (self.function)(self, vm, program)
    }

    /// `MOVE`  
    /// Moves a value from one location on the stack to another
    ///
    /// `dst`: Location on the stack to store the value  
    /// `src`: Location on the stack to load the value
    pub const fn move_bytecode(dst: u8, src: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Move, dst, src, 0, 0),
            function: Self::execute_move,
        }
    }

    /// `LOADI`
    /// Loads a `integer` into the stack
    ///
    /// `dst`: Location on the stack to place integer
    /// `integer`: Integer value to load into stack, this is limited
    /// 17 bits
    pub const fn load_integer(dst: u8, integer: i32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbx(OpCode::LoadInteger, dst, integer),
            function: Self::execute_load_integer,
        }
    }

    /// `LOADF`  
    /// Loads a `float` into the stack
    ///
    /// `dst`: Location on the stack to place integer  
    /// `value`: Float value to load into stack, this is limited
    /// to a whole floats that can be expressed in 17 bits
    pub const fn load_float(dst: u8, value: i32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbx(OpCode::LoadFloat, dst, value),
            function: Self::execute_load_float,
        }
    }

    /// `LOADK`  
    /// Loads the value of a constant into the stack
    ///
    /// `dst`: Location on the stack to place constant  
    /// `constant`: Id of `constant`
    pub const fn load_constant(dst: u8, constant: u32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::LoadConstant, dst, constant),
            function: Self::execute_load_constant,
        }
    }

    /// `LOADFALSE`  
    /// Loads a `false` value into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    pub const fn load_false(dst: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LoadFalse, dst, 0, 0, 0),
            function: Self::execute_load_false,
        }
    }

    /// `LFALSESKIP`  
    /// Loads a `false` value into the stack and skips next instruction
    ///
    /// `dst`: Location on the stack to place boolean
    pub const fn load_false_skip(dst: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LoadFalseSkip, dst, 0, 0, 0),
            function: Self::execute_load_false_skip,
        }
    }

    /// `LOADTRUE`  
    /// Loads a `false` value into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    pub const fn load_true(dst: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LoadTrue, dst, 0, 0, 0),
            function: Self::execute_load_true,
        }
    }

    /// `LOADNIL`  
    /// Loads a `nil` into the stack
    ///
    /// `dst`: Location on the stack to place nil  
    /// `extras`: Extra number of `nil`s to load
    pub const fn load_nil(dst: u8, extras: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LoadNil, dst, extras, 0, 0),
            function: Self::execute_load_nil,
        }
    }

    /// `GETUPVAL`  
    /// Gets a upvalue and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `upvalue`: Upvalue to load
    pub const fn get_upvalue(dst: u8, upvalue: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::GetUpValue, dst, upvalue, 0, 0),
            function: Self::execute_get_upvalue,
        }
    }

    /// `GETTABUP`  
    /// Get key into a upvalue and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `upvalue`: Upvalue to collect the field from  
    /// `key`: Location on `constants` where the key into the table resides
    pub const fn get_uptable(dst: u8, upvalue: u8, key: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::GetUpTable, dst, upvalue, key, 0),
            function: Self::execute_get_uptable,
        }
    }

    /// `GETTABLE`  
    /// Loads a table field to the stack using a stack value
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `src`: Location of the name on the stack  
    pub const fn get_table(dst: u8, table: u8, src: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::GetTable, dst, table, src, 0),
            function: Self::execute_get_table,
        }
    }

    /// `GETI`  
    /// Loads a value from the table into the stack using integer index
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `index`: Index of the item to load
    pub const fn get_index(dst: u8, table: u8, index: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::GetIndex, dst, table, index, 0),
            function: Self::execute_get_index,
        }
    }

    /// `GETFIELD`  
    /// Loads a table field to the stack using a key
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the key on `constants`  
    pub const fn get_field(dst: u8, table: u8, index: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::GetField, dst, table, index, 0),
            function: Self::execute_get_field,
        }
    }

    /// `SETTABUP`  
    /// Sets a value a global with a value from the stack
    ///
    /// `uptable`: UpTable to store the value at  
    /// `key`: Location on `constants` where the key into the table reside  
    /// `src`: Location of the value  
    /// `constant`: Whether `src` is a value on `stack` or on `constants`
    pub const fn set_uptable(uptable: u8, key: u8, src: u8, constant: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::SetUpTable, uptable, key, src, constant),
            function: Self::execute_set_uptable,
        }
    }

    /// `SETTABLE`  
    /// Sets a table field to a value using a value
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location on the stack of the value that will be used
    /// as key  
    /// `src`: Location of the value  
    /// `constant`: Whether `src` is a value on `stack` or on `constants`
    pub const fn set_table(table: u8, key: u8, src: u8, constant: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::SetTable, table, key, src, constant),
            function: Self::execute_set_table,
        }
    }

    /// `SETFIELD`  
    /// Sets a table field to a value using a name
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    /// `src`: Location of the value  
    /// `constant`: Whether `src` is a value on `stack` or on `constants`
    pub const fn set_field(table: u8, key: u8, src: u8, constant: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::SetField, table, key, src, constant),
            function: Self::execute_set_field,
        }
    }

    /// `NEWTABLE`  
    /// Creates a new table value
    ///
    /// `dst`: Location on the stack to store the table  
    /// `array_len`: Amount of items to allocate on the list  
    /// `table_len`: Amount of items to allocate for the map
    pub const fn new_table(dst: u8, array_len: u8, table_len: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::NewTable, dst, array_len, table_len, 0),
            function: Self::execute_new_table,
        }
    }

    /// `SELF`  
    /// Get a method and pass self as the first argument  
    ///
    /// `dst`: Destination of the closure  
    /// `table`: Location of the table on the `stack`     
    /// `key`: Location of the key on `constants`  
    pub const fn table_self(dst: u8, table: u8, key: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::TableSelf, dst, table, key, 0),
            function: Self::execute_table_self,
        }
    }

    /// `ADDI`  
    /// Performs arithmetic addition with an integer.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `integer`: Integer value to add
    pub const fn add_integer(dst: u8, lhs: u8, integer: i8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_absck(OpCode::AddInteger, dst, lhs, integer, 0),
            function: Self::execute_add_integer,
        }
    }
    /// `ADDK`  
    /// Performs arithmetic addition with a constant.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    pub const fn add_constant(dst: u8, lhs: u8, constant: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::AddConstant, dst, lhs, constant, 0),
            function: Self::execute_add_constant,
        }
    }

    /// `MULK`  
    /// Performs arithmetic multiplication with a constant.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    pub const fn mul_constant(dst: u8, lhs: u8, constant: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::MulConstant, dst, lhs, constant, 0),
            function: Self::execute_mul_constant,
        }
    }

    /// `ADD`  
    /// Performs arithmetic addition.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn add(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Add, dst, lhs, rhs, 0),
            function: Self::execute_add,
        }
    }

    /// `SUB`  
    /// Performs arithmetic subtraction.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn sub(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Sub, dst, lhs, rhs, 0),
            function: Self::execute_sub,
        }
    }

    /// `MUL`  
    /// Performs arithmetic multiplication.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn mul(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Mul, dst, lhs, rhs, 0),
            function: Self::execute_mul,
        }
    }

    /// `MOD`  
    /// Performs arithmetic modulus.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn mod_bytecode(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Mod, dst, lhs, rhs, 0),
            function: Self::execute_mod,
        }
    }

    /// `POW`  
    /// Performs arithmetic power.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn pow(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Pow, dst, lhs, rhs, 0),
            function: Self::execute_pow,
        }
    }

    /// `DIV`  
    /// Performs arithmetic division.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn div(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Div, dst, lhs, rhs, 0),
            function: Self::execute_div,
        }
    }

    /// `IDIV`  
    /// Performs arithmetic whole division.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn idiv(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::IDiv, dst, lhs, rhs, 0),
            function: Self::execute_idiv,
        }
    }

    /// `BAND`  
    /// Performs bitwise `and`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn bit_and(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::BitAnd, dst, lhs, rhs, 0),
            function: Self::execute_bit_and,
        }
    }

    /// `BOR`  
    /// Performs bitwise `or`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn bit_or(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::BitOr, dst, lhs, rhs, 0),
            function: Self::execute_bit_or,
        }
    }

    /// `BXOR`  
    /// Performs bitwise `xor`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn bit_xor(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::BitXor, dst, lhs, rhs, 0),
            function: Self::execute_bit_xor,
        }
    }

    /// `SHL`  
    /// Performs bitwise shift left.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn shift_left(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::ShiftLeft, dst, lhs, rhs, 0),
            function: Self::execute_shift_left,
        }
    }

    /// `SHR`  
    /// Performs bitwise shift right.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub const fn shift_right(dst: u8, lhs: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::ShiftRight, dst, lhs, rhs, 0),
            function: Self::execute_shift_right,
        }
    }

    /// `UNM`  
    /// Performs negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub const fn neg(dst: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Neg, dst, rhs, 0, 0),
            function: Self::execute_neg,
        }
    }

    /// `BNOT`
    /// Performs bit-wise not.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub const fn bit_not(dst: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::BitNot, dst, rhs, 0, 0),
            function: Self::execute_bit_not,
        }
    }

    /// `NOT`  
    /// Performs logical negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub const fn not(dst: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Not, dst, rhs, 0, 0),
            function: Self::execute_not,
        }
    }

    /// `LEN`  
    /// Performs length calculation on String.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub const fn len(dst: u8, rhs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Len, dst, rhs, 0, 0),
            function: Self::execute_len,
        }
    }

    /// `CONCAT`  
    /// Performs concatenation.
    ///
    /// `first`: Location on stack of first string, the result is
    /// stored here  
    /// `string_count`: Number of strings to concat
    pub const fn concat(first: u8, count: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Concat, first, count, 0, 0),
            function: Self::execute_concat,
        }
    }

    /// `JMP`  
    /// Performs jump.
    ///
    /// `jump`: Number of intructions to jump
    pub const fn jump(jump: i32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asj(OpCode::Jump, jump),
            function: Self::execute_jump,
        }
    }

    /// `LT`  
    /// Performs less than (<) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    pub const fn less_than(lst: u8, rhs: u8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LessThan, lst, rhs, 0, test),
            function: Self::execute_less_than,
        }
    }

    /// `LE`  
    /// Performs less than or equal (<=) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    pub const fn less_equal(lhs: u8, rhs: u8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LessEqual, lhs, rhs, 0, test),
            function: Self::execute_less_equal,
        }
    }

    /// `EQK`
    /// Peforms equal comparison (==) between the register and constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `constant`: Id of constant  
    /// `test`: If it should test for `true` or `false`
    pub const fn equal_constant(lhs: u8, rhs: u8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::EqualConstant, lhs, rhs, 0, test),
            function: Self::execute_equal_constant,
        }
    }

    /// `EQI`
    /// Peforms equal comparison (==) between the register and i8.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant  
    /// `test`: If it should test for `true` or `false`
    pub const fn equal_integer(lhs: u8, rhs: i8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(OpCode::EqualInteger, lhs, rhs, 0, test),
            function: Self::execute_equal_integer,
        }
    }

    /// `GTI`
    /// Peforms a greater than (>) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    pub const fn greater_than_integer(lhs: u8, rhs: i8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(OpCode::GreaterThanInteger, lhs, rhs, 0, test),
            function: Self::execute_greater_than_integer,
        }
    }

    /// `GEI`
    /// Peforms a greater or equal (>=) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    pub const fn greater_equal_integer(lhs: u8, rhs: i8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(OpCode::GreaterEqualInteger, lhs, rhs, test, 0),
            function: Self::execute_greater_equal_integer,
        }
    }

    /// `TEST`  
    /// Performs test.
    ///
    /// `register`: Location on stack of the value that if going to be tested  
    /// `test`: Test to perform  
    pub const fn test(register: u8, test: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Test, register, 0, 0, test),
            function: Self::execute_test,
        }
    }

    /// `CALL`  
    /// Calls a function
    ///
    /// `func`: Location on the stack where the function was loaded  
    /// `in`: Number of items on the stack going into the function, the function
    /// itself counts as one item, `0` means a variable number of items   
    /// `out`: How many items coming out of the function should be moved into
    /// the caller stack frame, `0` means all  
    pub const fn call(func: u8, in_params: u8, out_params: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Call, func, in_params, out_params, 0),
            function: Self::execute_call,
        }
    }

    /// `TAILCALL`  
    /// Calls a function
    ///
    /// `func`: Location on the stack where the function was loaded  
    /// `args`: Count of arguments  
    /// `variadics`: Number of variadic arguments
    pub const fn tail_call(func: u8, args: u8, variadics: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::TailCall, func, args, variadics, 0),
            function: Self::execute_tail_call,
        }
    }

    /// `RETURN0`  
    /// Returns from function
    ///
    /// `register`: First item on stack to return  
    /// `count`: Number of items to return, the actual value is subtracting
    /// it by 1, if zero, return all items on stack  
    /// `varargs`: If `var_args` > 0, the function is has variadic arguments
    /// and the number of fixed arguments is `var_args - 1`
    pub const fn return_bytecode(register: u8, count: u8, varargs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Return, register, count, varargs, 0),
            function: Self::execute_return,
        }
    }

    /// `RETURN0`  
    /// Returns from function with 0 out values
    pub const fn zero_return() -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::ZeroReturn, 0, 0, 0, 0),
            function: Self::execute_zero_return,
        }
    }
    /// `RETURN1`  
    /// Returns from function with 1 out values
    ///
    /// `return`: Location on stack of the returned value
    pub const fn one_return(register: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::OneReturn, register, 0, 0, 0),
            function: Self::execute_one_return,
        }
    }

    /// `FORLOOP`  
    /// Increment counter and jumps back to start of loop
    ///
    /// `for`: Location on the stack counter information is stored  
    /// `jump`: Number of byte codes to jump to reach start of for block
    pub const fn for_loop(counter: u8, jump: u32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::ForLoop, counter, jump),
            function: Self::execute_for_loop,
        }
    }

    /// `FORPREP`  
    /// Prepares for loop counter
    ///
    /// `for`: Location on the stack counter information is stored  
    /// `jump`: Number of byte codes to jump to reach end of for loop
    pub const fn for_prepare(counter: u8, jump: u32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::ForPrepare, counter, jump),
            function: Self::execute_for_prepare,
        }
    }

    /// `SETLIST`  
    /// Stores multiple values from the stack into the table
    ///
    /// `table`: Location of the table on the stack  
    /// `array_len`: Number of items on the stack to store  
    /// `c`: ?
    pub const fn set_list(table: u8, array_len: u8, c: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::SetList, table, array_len, c, 0),
            function: Self::execute_set_list,
        }
    }

    /// `CLOSURE`
    /// Puts reference to a local function into the stack
    ///
    /// `dst`: Stack location to store function reference  
    /// `func_id`: Id of function
    pub const fn closure(dst: u8, func_id: u32) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::Closure, dst, func_id),
            function: Self::execute_closure,
        }
    }

    /// `VARARG`
    /// Collect variable arguments
    ///
    /// `register`: First destination of variable arguments  
    /// `count`: Count of variable arguments, `0` means use all, other values
    /// are subtracted by `2`
    pub const fn variadic_arguments(register: u8, count: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::VariadicArguments, register, 0, count, 0),
            function: Self::execute_variadic_arguments,
        }
    }

    /// `VARARGPREP`
    /// Prepares the variadic arguments of the closure.
    ///
    /// `fixed`: Number of fixed arguments
    pub const fn variadic_arguments_prepare(varargs: u8) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::VariadicArgumentsPrepare, varargs, 0, 0, 0),
            function: Self::execute_variadic_arguments_prepare,
        }
    }

    fn execute_move(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, src, _, _) = self.decode_abck();
        let value = vm.get_stack(src)?.clone();
        vm.set_stack(dst, value)
    }

    fn execute_load_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, value) = self.decode_asbx();
        vm.set_stack(dst, Value::Integer(i64::from(value)))
    }

    fn execute_load_float(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, value) = self.decode_asbx();
        vm.set_stack(dst, Value::Float(value as f64))
    }

    fn execute_load_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (dst, constant) = self.decode_abx();

        vm.set_stack(
            dst,
            vm.get_running_closure(main_program).constants[usize::try_from(constant)?].clone(),
        )
    }

    fn execute_load_false(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, _, _, _) = self.decode_abck();
        vm.set_stack(dst, Value::Boolean(false))
    }

    fn execute_load_false_skip(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, _, _, _) = self.decode_abck();
        vm.jump(1)?;
        vm.set_stack(dst, Value::Boolean(false))
    }

    fn execute_load_true(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, _, _, _) = self.decode_abck();
        vm.set_stack(dst, Value::Boolean(true))
    }

    fn execute_load_nil(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, extras, _, _) = self.decode_abck();
        // If `extra` is 0, runs once
        for dst in dst..=(dst + extras) {
            vm.set_stack(dst, Value::Nil)?;
        }
        Ok(())
    }

    fn execute_get_upvalue(&self, vm: &mut Lua, _main_program: &Program) -> Result<(), Error> {
        let (dst, upvalue, _, _) = self.decode_abck();

        let upvalue = vm
            .get_up_table(usize::from(upvalue))
            .cloned()
            .ok_or(Error::UpvalueDoesNotExist)?;

        vm.set_stack(dst, upvalue)
    }

    fn execute_get_uptable(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (dst, upvalue, key, _) = self.decode_abck();

        let Value::Table(upvalue) = vm
            .get_up_table(usize::from(upvalue))
            .ok_or(Error::UpvalueDoesNotExist)?
        else {
            return Err(Error::ExpectedTable);
        };

        let key = &vm.get_running_closure(main_program).constants[usize::from(key)];
        let value = upvalue.deref().borrow().get(ValueKey(key.clone())).clone();

        vm.set_stack(dst, value)
    }

    fn execute_get_table(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, table, src, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(table)?.clone() {
            let key = ValueKey::from(vm.get_stack(src)?.clone());
            let bin_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);

            let value = match bin_search {
                Ok(i) => (*table).borrow().table[i].1.clone(),
                Err(_) => Value::Nil,
            };
            vm.set_stack(dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_get_index(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, table, index, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(table)?.clone() {
            let value = if index == 0 {
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
                    .get(usize::from(index) - 1)
                    .cloned()
                    .unwrap_or(Value::Nil)
            };
            vm.set_stack(dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_get_field(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (dst, table, key, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(table)?.clone() {
            let key = ValueKey::from(
                vm.get_running_closure(main_program).constants[usize::from(key)].clone(),
            );
            let bin_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);

            let value = match bin_search {
                Ok(i) => (*table).borrow().table[i].1.clone(),
                Err(_) => Value::Nil,
            };
            vm.set_stack(dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_set_uptable(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (upvalue, key, src, constant) = self.decode_abck();

        let running_program = vm.get_running_closure(main_program);
        let key = running_program.constants[usize::from(key)].clone();
        let value = if constant == 1 {
            running_program.constants[usize::from(src)].clone()
        } else {
            vm.get_stack(src)?.clone()
        };

        match vm.get_up_table(usize::from(upvalue)) {
            Some(Value::Table(upvalue)) => upvalue.borrow_mut().set(ValueKey(key), value),
            Some(_) => Err(Error::ExpectedTable),
            None => Err(Error::UpvalueDoesNotExist),
        }
    }

    fn execute_set_table(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (table, key, src, constant) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(table)?.clone() {
            let key = ValueKey::from(vm.get_stack(key)?.clone());
            let value = if constant == 1 {
                main_program.constants[usize::from(src)].clone()
            } else {
                vm.get_stack(src)?.clone()
            };

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

    fn execute_set_field(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (table, key, src, constant) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(table)?.clone() {
            let running_program = vm.get_running_closure(main_program);
            let key = ValueKey::from(running_program.constants[usize::from(key)].clone());
            let value = if constant == 1 {
                running_program.constants[usize::from(src)].clone()
            } else {
                vm.get_stack(src)?.clone()
            };

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

    fn execute_new_table(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, array_initial_size, table_initial_size, _) = self.decode_abck();

        vm.set_stack(
            dst,
            Value::Table(Rc::new(RefCell::new(Table::new(
                usize::from(array_initial_size),
                usize::from(table_initial_size),
            )))),
        )
    }

    fn execute_table_self(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (dst, table, key, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(table).cloned()? {
            vm.set_stack(dst + 1, Value::Table(table.clone()))?;

            let key = ValueKey::from(
                vm.get_running_closure(main_program).constants[usize::from(key)].clone(),
            );
            let bin_search = (*table)
                .borrow()
                .table
                .binary_search_by_key(&&key, |a| &a.0);

            let value = match bin_search {
                Ok(i) => (*table).borrow().table[i].1.clone(),
                Err(_) => Value::Nil,
            };
            vm.set_stack(dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_add_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, int, _) = self.decode_absck();

        let res = match &vm.get_stack(lhs)? {
            Value::Integer(l) => Value::Integer(l + i64::from(int)),
            Value::Float(l) => Value::Float(l + int as f64),
            lhs => {
                return Err(Error::ArithmeticOperand(
                    "add",
                    lhs.static_type_name(),
                    "integer",
                ))
            }
        };
        vm.set_stack(dst, res)
    }

    fn execute_add_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (dst, lhs, constant, _) = self.decode_abck();

        let res = match (
            &vm.get_stack(lhs)?,
            &vm.get_running_closure(main_program).constants[usize::from(constant)],
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
        vm.set_stack(dst, res)
    }

    fn execute_mul_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (dst, lhs, constant, _) = self.decode_abck();

        let res = match (
            &vm.get_stack(lhs)?,
            &vm.get_running_closure(main_program).constants[usize::from(constant)],
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
        vm.set_stack(dst, res)
    }

    fn execute_add(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_sub(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_mul(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_mod(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_pow(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_div(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_idiv(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
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
        vm.set_stack(dst, res)
    }

    fn execute_bit_and(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l & r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "and",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(dst, res)
    }

    fn execute_bit_or(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l | r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "or",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(dst, res)
    }

    fn execute_bit_xor(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l ^ r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "xor",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(dst, res)
    }

    fn execute_shift_left(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(lhs)?, &vm.get_stack(rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l << r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "shift left",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ))
            }
        };
        vm.set_stack(dst, res)
    }

    fn execute_shift_right(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (
            &vm.get_stack(lhs)?.clone().try_int(),
            &vm.get_stack(rhs)?.clone().try_int(),
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
        vm.set_stack(dst, res)
    }

    fn execute_neg(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match vm.get_stack(rhs)? {
            Value::Integer(integer) => Value::Integer(-integer),
            Value::Float(float) => Value::Float(-float),
            _ => return Err(Error::InvalidNegOperand),
        };
        vm.set_stack(dst, value)
    }

    fn execute_bit_not(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match vm.get_stack(rhs)? {
            Value::Integer(integer) => Value::Integer(!integer),
            _ => return Err(Error::InvalidBitNotOperand),
        };
        vm.set_stack(dst, value)
    }

    fn execute_not(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match &vm.get_stack(rhs)? {
            Value::Boolean(false) | Value::Nil => Value::Boolean(true),
            _ => Value::Boolean(false),
        };
        vm.set_stack(dst, value)
    }

    fn execute_len(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match &vm.get_stack(rhs)? {
            Value::String(string) => Value::Integer(i64::try_from(string.len())?),
            Value::ShortString(string) => Value::Integer(i64::try_from(string.len())?),
            _ => return Err(Error::InvalidLenOperand),
        };
        vm.set_stack(dst, value)
    }

    fn execute_concat(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (first, count, _, _) = self.decode_abck();

        let mut strings = Vec::with_capacity(usize::from(count));

        for src in first..(first + count) {
            match &vm.get_stack(src)? {
                Value::Integer(lhs) => strings.push(lhs.to_string()),
                Value::Float(lhs) => strings.push(lhs.to_string()),
                Value::ShortString(lhs) => strings.push(lhs.to_string()),
                Value::String(lhs) => strings.push(lhs.to_string()),
                other => return Err(Error::ConcatOperand(other.static_type_name())),
            };
        }

        let concatenated = strings.into_iter().collect::<String>();
        vm.set_stack(first, concatenated.as_str().into())
    }

    fn execute_jump(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let jump = self.decode_sj();

        vm.jump(isize::try_from(jump)?)
    }

    fn execute_less_than(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (lhs, rhs, _, test) = self.decode_abck();

        let lhs = vm.get_stack(lhs)?;
        let rhs = vm.get_stack(rhs)?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Less, test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_less_equal(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (lhs, rhs, _, test) = self.decode_abck();

        let lhs = &vm.get_stack(lhs)?;
        let rhs = &vm.get_stack(rhs)?;

        Self::relational_comparison(
            lhs,
            rhs,
            |ordering| ordering != Ordering::Greater,
            test == 1,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    fn execute_equal_constant(&self, vm: &mut Lua, main_program: &Program) -> Result<(), Error> {
        let (register, constant, _, test) = self.decode_abck();

        let lhs = vm.get_stack(register)?;
        let rhs = &vm.get_running_closure(main_program).constants[usize::from(constant)];

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Equal, test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_equal_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(register)?;
        let rhs = Value::Integer(i64::from(integer));

        Self::relational_comparison(lhs, &rhs, |ordering| ordering == Ordering::Equal, test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_greater_than_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(register)?;
        let rhs = Value::Integer(i64::from(integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering == Ordering::Greater,
            test == 1,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    fn execute_greater_equal_integer(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(register)?;
        let rhs = Value::Integer(i64::from(integer));

        Self::relational_comparison(lhs, &rhs, |ordering| ordering != Ordering::Less, test == 1)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_test(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (src, _, _, test) = self.decode_abck();

        let cond = vm.get_stack(src)?;
        match (cond, test) {
            (Value::Nil | Value::Boolean(false), 0) => (),
            (Value::Nil | Value::Boolean(false), 1) => vm.jump(1)?,
            (_, 1) => (),
            (_, 0) => vm.jump(1)?,
            (_, _) => unreachable!("Test is always 0 or 1."),
        };

        Ok(())
    }

    fn execute_call(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (func_index, in_items, out, _) = self.decode_abck();

        let func_index_usize = usize::from(func_index);
        let in_items = usize::from(in_items);
        let out_params = usize::from(out);

        let func = &vm.get_stack(func_index)?;
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

    fn execute_tail_call(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (func_index, args, out_params, _) = self.decode_abck();

        let func_index_usize = usize::from(func_index);
        let args = usize::from(args);
        let out_params = usize::from(out_params);

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

    fn execute_return(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        // TODO treat out params
        let (return_start, count, _, _) = self.decode_abck();
        vm.drop_stack_frame(usize::from(return_start), usize::from(count - 1));
        Ok(())
    }

    fn execute_zero_return(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        vm.drop_stack_frame(0, 0);
        Ok(())
    }

    fn execute_one_return(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (return_loc, _, _, _) = self.decode_abck();
        vm.drop_stack_frame(usize::from(return_loc), 1);
        Ok(())
    }

    fn execute_for_loop(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        let (for_stack, jmp) = self.decode_abx();

        if let Value::Integer(counter) = &mut vm.get_stack_mut(for_stack + 1)? {
            if counter != &0 {
                *counter -= 1;
                Bytecode::add(for_stack + 3, for_stack + 3, for_stack + 2).execute(vm, program)?;
                vm.jump(-isize::try_from(jmp)?)?;
            }
            Ok(())
        } else {
            log::error!("For loop counter should be an Integer.");
            Err(Error::ForZeroStep)
        }
    }

    fn execute_for_prepare(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (for_stack, jmp) = self.decode_abx();

        let init = vm.get_stack(for_stack)?;
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
                vm.jump(isize::try_from(jmp)? + 1)?;
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

            vm.set_stack(for_stack, Value::Float(init))?;
            vm.set_stack(for_stack + 1, Value::Integer(count as i64))?;
            vm.set_stack(for_stack + 2, Value::Float(step))?;
            vm.set_stack(for_stack + 3, Value::Float(init))?;
            if count <= 0.0 {
                vm.jump(isize::try_from(jmp)? + 1)?;
            }
            Ok(())
        }
    }

    fn execute_set_list(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (table, count, _c, _) = self.decode_abck();

        let top_stack = vm.get_stack_frame();

        let table_items_start =
            top_stack.stack_frame + top_stack.variadic_arguments + usize::from(table) + 1;
        if let Value::Table(table) = vm.get_stack(table)?.clone() {
            let values = if count == 0 {
                let true_count = vm.stack.len() - table_items_start;
                vm.stack
                    .drain(table_items_start..(table_items_start + true_count))
            } else {
                vm.stack
                    .drain(table_items_start..(table_items_start + usize::from(count)))
            };

            table.borrow_mut().array.extend(values);
            Ok(())
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_closure(&self, vm: &mut Lua, program: &Program) -> Result<(), Error> {
        let (dst, func_id) = self.decode_abx();
        let func = program.functions[usize::try_from(func_id)?].clone();
        vm.set_stack(dst, func)
    }

    fn execute_variadic_arguments(&self, vm: &mut Lua, _program: &Program) -> Result<(), Error> {
        let (register, _, count, _) = self.decode_abck();

        let top_stack = vm.get_stack_frame();

        let variadics = top_stack.variadic_arguments;

        if count == 0 {
            let start = top_stack.stack_frame;
            let end = start + variadics;

            let move_vals = vm.stack[start..end].to_vec();

            vm.stack.truncate(start + variadics + usize::from(register));
            vm.stack.extend(move_vals);
        } else {
            let true_count = usize::from(count - 1);
            let start = top_stack.stack_frame;
            let end = start + true_count.min(variadics);

            let move_vals = vm.stack[start..end].to_vec();

            vm.stack.truncate(start + variadics + usize::from(register));
            vm.stack.extend(move_vals);

            if true_count > variadics {
                let remaining = true_count - variadics;
                vm.stack.resize(vm.stack.len() + remaining, Value::Nil);
            }
        }

        Ok(())
    }

    fn execute_variadic_arguments_prepare(
        &self,
        _vm: &mut Lua,
        _program: &Program,
    ) -> Result<(), Error> {
        // Do nothing
        Ok(())
    }

    pub fn flip_test(&mut self) {
        assert!(self.get_opcode().is_relational());
        let op = self.get_opcode();
        let (a, b, c, test) = self.decode_abck();
        assert_eq!(c, 0);

        self.bytecode = Self::encode_abck(op, a, b, c, test ^ 1);
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

    pub(crate) const fn encode_abck(op: OpCode, a: u8, b: u8, c: u8, k: u8) -> u32 {
        let mut bytecode = 0;
        Self::set_opcode(&mut bytecode, op);
        Self::set_a(&mut bytecode, a);
        Self::set_b(&mut bytecode, b);
        Self::set_c(&mut bytecode, c);
        Self::set_k(&mut bytecode, k);
        bytecode
    }

    pub(crate) const fn encode_asbck(op: OpCode, a: u8, sb: i8, c: u8, k: u8) -> u32 {
        let mut bytecode = 0;
        Self::set_opcode(&mut bytecode, op);
        Self::set_a(&mut bytecode, a);
        Self::set_sb(&mut bytecode, sb);
        Self::set_c(&mut bytecode, c);
        Self::set_k(&mut bytecode, k);
        bytecode
    }

    pub(crate) const fn encode_absck(op: OpCode, a: u8, b: u8, sc: i8, k: u8) -> u32 {
        let mut bytecode = 0;
        Self::set_opcode(&mut bytecode, op);
        Self::set_a(&mut bytecode, a);
        Self::set_b(&mut bytecode, b);
        Self::set_sc(&mut bytecode, sc);
        Self::set_k(&mut bytecode, k);
        bytecode
    }

    pub(crate) const fn encode_abx(op: OpCode, a: u8, bx: u32) -> u32 {
        let mut bytecode = 0;
        Self::set_opcode(&mut bytecode, op);
        Self::set_a(&mut bytecode, a);
        Self::set_bx(&mut bytecode, bx);
        bytecode
    }

    pub(crate) const fn encode_asbx(op: OpCode, a: u8, sbx: i32) -> u32 {
        let mut bytecode = 0;
        Self::set_opcode(&mut bytecode, op);
        Self::set_a(&mut bytecode, a);
        Self::set_sbx(&mut bytecode, sbx);
        bytecode
    }

    pub(crate) const fn encode_asj(op: OpCode, j: i32) -> u32 {
        let mut bytecode = 0;
        Self::set_opcode(&mut bytecode, op);
        Self::set_sj(&mut bytecode, j);
        bytecode
    }

    pub(crate) const fn decode_abck(&self) -> (u8, u8, u8, u8) {
        let a = self.get_a();
        let b = self.get_b();
        let c = self.get_c();
        let k = self.get_k();
        (a, b, c, k)
    }

    pub(crate) const fn decode_asbck(&self) -> (u8, i8, u8, u8) {
        let a = self.get_a();
        let b = self.get_sb();
        let c = self.get_c();
        let k = self.get_k();
        (a, b, c, k)
    }

    pub(crate) const fn decode_absck(&self) -> (u8, u8, i8, u8) {
        let a = self.get_a();
        let b = self.get_b();
        let c = self.get_sc();
        let k = self.get_k();
        (a, b, c, k)
    }

    pub(crate) const fn decode_abx(&self) -> (u8, u32) {
        let a = self.get_a();
        let b = self.get_bx();
        (a, b)
    }

    pub(crate) const fn decode_asbx(&self) -> (u8, i32) {
        let a = self.get_a();
        let b = self.get_sbx();
        (a, b)
    }

    pub(crate) const fn decode_ax(&self) -> u32 {
        (self.bytecode & 0xffffff80) >> A_SHIFT
    }

    pub(crate) fn decode_sj(&self) -> i32 {
        self.get_sj()
    }

    pub const fn get_opcode(&self) -> OpCode {
        OpCode::from_id((self.bytecode & 0x7f) as u8)
    }

    const fn set_opcode(bytecode: &mut u32, op: OpCode) {
        *bytecode |= (op as u8) as u32;
    }

    const fn get_a(&self) -> u8 {
        ((self.bytecode & A_MASK) >> A_SHIFT) as u8
    }

    const fn set_a(bytecode: &mut u32, a: u8) {
        *bytecode |= (a as u32) << A_SHIFT;
    }

    const fn get_b(&self) -> u8 {
        ((self.bytecode & B_MASK) >> B_SHIFT) as u8
    }

    const fn set_b(bytecode: &mut u32, b: u8) {
        *bytecode |= (b as u32) << B_SHIFT;
    }

    const fn get_c(&self) -> u8 {
        ((self.bytecode & C_MASK) >> C_SHIFT) as u8
    }

    const fn set_c(bytecode: &mut u32, c: u8) {
        *bytecode |= (c as u32) << C_SHIFT;
    }

    const fn get_k(&self) -> u8 {
        ((self.bytecode & K_MASK) >> K_SHIFT) as u8
    }

    const fn set_k(bytecode: &mut u32, k: u8) {
        assert!(k <= 1);
        *bytecode |= (k as u32) << K_SHIFT;
    }

    const fn get_sb(&self) -> i8 {
        let b = self.get_b();
        if b > I8_OFFSET {
            (b - I8_OFFSET) as i8
        } else {
            b as i8 - I8_OFFSET as i8
        }
    }

    const fn set_sb(bytecode: &mut u32, sb: i8) {
        let b = I8_OFFSET.saturating_add_signed(sb);
        Self::set_b(bytecode, b);
    }

    const fn get_sc(&self) -> i8 {
        let c = self.get_c();
        if c > I8_OFFSET {
            (c - I8_OFFSET) as i8
        } else {
            (c as i8) - (I8_OFFSET as i8)
        }
    }

    const fn set_sc(bytecode: &mut u32, sc: i8) {
        let c = I8_OFFSET.saturating_add_signed(sc);
        Self::set_c(bytecode, c);
    }

    const fn get_bx(&self) -> u32 {
        (self.bytecode & BX_MASK) >> BX_SHIFT
    }

    const fn set_bx(bytecode: &mut u32, bx: u32) {
        assert!(bx <= BX_MAX);
        *bytecode |= bx << BX_SHIFT;
    }

    const fn get_sbx(&self) -> i32 {
        let bx = self.get_bx();
        if bx > I17_OFFSET {
            (bx - I17_OFFSET) as i32
        } else {
            bx as i32 - I17_OFFSET as i32
        }
    }

    const fn set_sbx(bytecode: &mut u32, sbx: i32) {
        let bx = I17_OFFSET.saturating_add_signed(sbx);
        Self::set_bx(bytecode, bx);
    }

    const fn get_sj(&self) -> i32 {
        let j = self.bytecode >> J_SHIFT;
        if j > I25_OFFSET {
            (j - I25_OFFSET) as i32
        } else {
            j as i32 - I25_OFFSET as i32
        }
    }

    const fn set_sj(bytecode: &mut u32, sj: i32) {
        let j = I25_OFFSET.saturating_add_signed(sj);
        assert!(j <= J_MAX);
        *bytecode |= j << J_SHIFT;
    }
}

impl Debug for Bytecode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let op = self.get_opcode();
        match op {
            OpCode::ZeroReturn => {
                write!(f, "{:?}", op)
            }
            OpCode::LoadConstantExtraArgs
            | OpCode::LoadFalse
            | OpCode::LoadFalseSkip
            | OpCode::LoadTrue
            | OpCode::Close
            | OpCode::ToBeClosed
            | OpCode::OneReturn
            | OpCode::VariadicArgumentsPrepare => {
                let (a, _, _, _) = self.decode_abck();
                write!(f, "{:?}({})", op, a)
            }
            OpCode::Test => {
                let (a, _, _, k) = self.decode_abck();
                write!(f, "{:?}({}, {})", op, a, k)
            }
            OpCode::Move
            | OpCode::LoadNil
            | OpCode::GetUpValue
            | OpCode::SetUpValue
            | OpCode::Neg
            | OpCode::BitNot
            | OpCode::Not
            | OpCode::Len
            | OpCode::Concat => {
                let (a, b, _, _) = self.decode_abck();
                write!(f, "{:?}({}, {})", op, a, b)
            }
            OpCode::Equal
            | OpCode::LessThan
            | OpCode::LessEqual
            | OpCode::EqualConstant
            | OpCode::TestSet => {
                let (a, b, _, k) = self.decode_abck();
                write!(f, "{:?}({}, {}, {})", op, a, b, k)
            }
            OpCode::EqualInteger
            | OpCode::LessThanInteger
            | OpCode::LessEqualInteger
            | OpCode::GreaterThanInteger
            | OpCode::GreaterEqualInteger => {
                let (a, sb, _, k) = self.decode_asbck();
                write!(f, "{:?}({}, {}, {})", op, a, sb, k)
            }
            OpCode::LoadConstant
            | OpCode::ForPrepare
            | OpCode::ForLoop
            | OpCode::TailForPrepare
            | OpCode::TailForCall
            | OpCode::TailForLoop
            | OpCode::Closure => {
                let (a, bx) = self.decode_abx();
                write!(f, "{:?}({}, {})", op, a, bx)
            }
            OpCode::LoadInteger | OpCode::LoadFloat => {
                let (a, sbx) = self.decode_asbx();
                write!(f, "{:?}({}, {})", op, a, sbx)
            }
            OpCode::GetUpTable
            | OpCode::GetTable
            | OpCode::GetIndex
            | OpCode::GetField
            | OpCode::NewTable
            | OpCode::AddConstant
            | OpCode::SubConstant
            | OpCode::MulConstant
            | OpCode::ModConstant
            | OpCode::PowConstant
            | OpCode::DivConstant
            | OpCode::IDivConstant
            | OpCode::BitAndConstant
            | OpCode::BitOrConstant
            | OpCode::BitXorConstant
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Mod
            | OpCode::Pow
            | OpCode::Div
            | OpCode::IDiv
            | OpCode::BitAnd
            | OpCode::BitOr
            | OpCode::BitXor
            | OpCode::ShiftLeft
            | OpCode::ShiftRight
            | OpCode::MetaMethod
            | OpCode::Call => {
                let (a, b, c, _) = self.decode_abck();
                write!(f, "{:?}({}, {}, {})", op, a, b, c)
            }
            OpCode::MetaMethodInteger => {
                let (a, sb, c, _) = self.decode_asbck();
                write!(f, "{:?}({}, {}, {})", op, a, sb, c)
            }
            OpCode::AddInteger | OpCode::ShiftRightInteger | OpCode::ShiftLeftInteger => {
                let (a, b, sc, _) = self.decode_absck();
                write!(f, "{:?}({}, {}, {})", op, a, b, sc)
            }
            OpCode::SetUpTable
            | OpCode::SetTable
            | OpCode::SetIndex
            | OpCode::SetField
            | OpCode::TableSelf
            | OpCode::MetaMethodConstant
            | OpCode::TailCall
            | OpCode::Return
            | OpCode::SetList => {
                let (a, b, c, k) = self.decode_abck();
                write!(f, "{:?}({}, {}, {}, {})", op, a, b, c, k)
            }
            OpCode::VariadicArguments => {
                let (a, _, c, _) = self.decode_abck();
                write!(f, "{:?}({}, {})", op, a, c)
            }
            OpCode::ExtraArguments => {
                let ax = self.decode_ax();
                write!(f, "{:?}({})", op, ax)
            }
            OpCode::Jump => {
                let sj = self.decode_sj();
                write!(f, "{:?}({})", op, sj)
            }
        }
    }
}

impl Display for Bytecode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self, f)
    }
}
