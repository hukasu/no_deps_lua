pub mod arguments;
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
use arguments::{A, Ax, B, Bx, BytecodeArgument, C, K, Sb, Sbx, Sc, Sj};

use crate::{
    Lua,
    closure::{Closure, FunctionType, NativeClosure, Upvalue},
    function::Function,
    table::Table,
    value::{Value, ValueKey},
};

use super::Error;

pub use self::opcode::OpCode;

#[derive(Clone, Copy, PartialEq)]
pub struct Bytecode {
    bytecode: u32,
    function: BytecodeFunction,
}

type BytecodeFunction = fn(bytecode: &Bytecode, vm: &mut Lua) -> Result<(), Error>;

impl Bytecode {
    pub fn execute(&self, vm: &mut Lua) -> Result<(), Error> {
        (self.function)(self, vm)
    }

    /// `MOVE`  
    /// Moves a value from one location on the stack to another
    ///
    /// `dst`: Location on the stack to store the value  
    /// `src`: Location on the stack to load the value
    pub fn move_bytecode(dst: impl Into<A>, src: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Move, dst.into(), src.into(), C::ZERO, K::ZERO),
            function: Self::execute_move,
        }
    }

    /// `LOADI`
    /// Loads a `integer` into the stack
    ///
    /// `dst`: Location on the stack to place integer
    /// `integer`: Integer value to load into stack, this is limited
    /// 17 bits
    pub fn load_integer(dst: impl Into<A>, integer: impl Into<Sbx>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbx(OpCode::LoadInteger, dst.into(), integer.into()),
            function: Self::execute_load_integer,
        }
    }

    /// `LOADF`  
    /// Loads a `float` into the stack
    ///
    /// `dst`: Location on the stack to place integer  
    /// `value`: Float value to load into stack, this is limited
    /// to a whole floats that can be expressed in 17 bits
    pub fn load_float(dst: impl Into<A>, value: impl Into<Sbx>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbx(OpCode::LoadFloat, dst.into(), value.into()),
            function: Self::execute_load_float,
        }
    }

    /// `LOADK`  
    /// Loads the value of a constant into the stack
    ///
    /// `dst`: Location on the stack to place constant  
    /// `constant`: Id of `constant`
    pub fn load_constant(dst: impl Into<A>, constant: impl Into<Bx>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::LoadConstant, dst.into(), constant.into()),
            function: Self::execute_load_constant,
        }
    }

    /// `LOADFALSE`  
    /// Loads a `false` value into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    pub fn load_false(dst: impl Into<A>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LoadFalse, dst.into(), B::ZERO, C::ZERO, K::ZERO),
            function: Self::execute_load_false,
        }
    }

    /// `LFALSESKIP`  
    /// Loads a `false` value into the stack and skips next instruction
    ///
    /// `dst`: Location on the stack to place boolean
    pub fn load_false_skip(dst: impl Into<A>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::LoadFalseSkip,
                dst.into(),
                B::ZERO,
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_load_false_skip,
        }
    }

    /// `LOADTRUE`  
    /// Loads a `false` value into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    pub fn load_true(dst: impl Into<A>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::LoadTrue, dst.into(), B::ZERO, C::ZERO, K::ZERO),
            function: Self::execute_load_true,
        }
    }

    /// `LOADNIL`  
    /// Loads a `nil` into the stack
    ///
    /// `dst`: Location on the stack to place nil  
    /// `extras`: Extra number of `nil`s to load
    pub fn load_nil(dst: impl Into<A>, extras: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::LoadNil,
                dst.into(),
                extras.into(),
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_load_nil,
        }
    }

    /// `GETUPVAL`  
    /// Gets a upvalue and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `upvalue`: Upvalue to load
    pub fn get_upvalue(dst: impl Into<A>, upvalue: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::GetUpValue,
                dst.into(),
                upvalue.into(),
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_get_upvalue,
        }
    }

    /// `SETUPVAL`  
    /// Sets a upvalue with value on stack
    ///
    /// `value`: Value on to set upvalue to
    /// `upvalue`: Upvalue to set  
    pub fn set_upvalue(value: impl Into<A>, upvalue: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::SetUpValue,
                value.into(),
                upvalue.into(),
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_set_upvalue,
        }
    }

    /// `GETTABUP`  
    /// Get key into a upvalue and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `upvalue`: Upvalue to collect the field from  
    /// `key`: Location on `constants` where the key into the table resides
    pub fn get_uptable(dst: impl Into<A>, upvalue: impl Into<B>, key: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::GetUpTable,
                dst.into(),
                upvalue.into(),
                key.into(),
                K::ZERO,
            ),
            function: Self::execute_get_uptable,
        }
    }

    /// `GETTABLE`  
    /// Loads a table field to the stack using a stack value
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on the stack  
    pub fn get_table(dst: impl Into<A>, table: impl Into<B>, key: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::GetTable,
                dst.into(),
                table.into(),
                key.into(),
                K::ZERO,
            ),
            function: Self::execute_get_table,
        }
    }

    /// `GETI`  
    /// Loads a value from the table into the stack using integer index
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `index`: Index of the item to load
    pub fn get_index(dst: impl Into<A>, table: impl Into<B>, index: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::GetIndex,
                dst.into(),
                table.into(),
                index.into(),
                K::ZERO,
            ),
            function: Self::execute_get_index,
        }
    }

    /// `GETFIELD`  
    /// Loads a table field to the stack using a key
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the key on `constants`  
    pub fn get_field(dst: impl Into<A>, table: impl Into<B>, index: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::GetField,
                dst.into(),
                table.into(),
                index.into(),
                K::ZERO,
            ),
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
    pub fn set_uptable(
        uptable: impl Into<A>,
        key: impl Into<B>,
        src: impl Into<C>,
        constant: impl Into<K>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::SetUpTable,
                uptable.into(),
                key.into(),
                src.into(),
                constant.into(),
            ),
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
    pub fn set_table(
        table: impl Into<A>,
        key: impl Into<B>,
        src: impl Into<C>,
        constant: impl Into<K>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::SetTable,
                table.into(),
                key.into(),
                src.into(),
                constant.into(),
            ),
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
    pub fn set_field(
        table: impl Into<A>,
        key: impl Into<B>,
        src: impl Into<C>,
        constant: impl Into<K>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::SetField,
                table.into(),
                key.into(),
                src.into(),
                constant.into(),
            ),
            function: Self::execute_set_field,
        }
    }

    /// `NEWTABLE`  
    /// Creates a new table value
    ///
    /// `dst`: Location on the stack to store the table  
    /// `array_len`: Amount of items to allocate on the list  
    /// `table_len`: Amount of items to allocate for the map
    pub fn new_table(
        dst: impl Into<A>,
        table_len: impl Into<B>,
        array_len: impl Into<C>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::NewTable,
                dst.into(),
                table_len.into(),
                array_len.into(),
                K::ZERO,
            ),
            function: Self::execute_new_table,
        }
    }

    /// `SELF`  
    /// Get a method and pass self as the first argument  
    ///
    /// `dst`: Destination of the closure  
    /// `table`: Location of the table on the `stack`     
    /// `key`: Location of the key on `constants`  
    pub fn table_self(dst: impl Into<A>, table: impl Into<B>, key: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::TableSelf,
                dst.into(),
                table.into(),
                key.into(),
                K::ZERO,
            ),
            function: Self::execute_table_self,
        }
    }

    /// `ADDI`  
    /// Performs arithmetic addition with an integer.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `integer`: Integer value to add
    pub fn add_integer(dst: impl Into<A>, lhs: impl Into<B>, integer: impl Into<Sc>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_absck(
                OpCode::AddInteger,
                dst.into(),
                lhs.into(),
                integer.into(),
                K::ZERO,
            ),
            function: Self::execute_add_integer,
        }
    }
    /// `ADDK`  
    /// Performs arithmetic addition with a constant.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    pub fn add_constant(dst: impl Into<A>, lhs: impl Into<B>, constant: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::AddConstant,
                dst.into(),
                lhs.into(),
                constant.into(),
                K::ZERO,
            ),
            function: Self::execute_add_constant,
        }
    }

    /// `MULK`  
    /// Performs arithmetic multiplication with a constant.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `constant`: Location on `constant` of right-hand operand
    pub fn mul_constant(dst: impl Into<A>, lhs: impl Into<B>, constant: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::MulConstant,
                dst.into(),
                lhs.into(),
                constant.into(),
                K::ZERO,
            ),
            function: Self::execute_mul_constant,
        }
    }

    /// `ADD`  
    /// Performs arithmetic addition.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn add(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Add, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_add,
        }
    }

    /// `SUB`  
    /// Performs arithmetic subtraction.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn sub(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Sub, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_sub,
        }
    }

    /// `MUL`  
    /// Performs arithmetic multiplication.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn mul(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Mul, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_mul,
        }
    }

    /// `MOD`  
    /// Performs arithmetic modulus.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn mod_bytecode(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Mod, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_mod,
        }
    }

    /// `POW`  
    /// Performs arithmetic power.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn pow(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Pow, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_pow,
        }
    }

    /// `DIV`  
    /// Performs arithmetic division.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn div(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Div, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_div,
        }
    }

    /// `IDIV`  
    /// Performs arithmetic whole division.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn idiv(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::IDiv, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_idiv,
        }
    }

    /// `BAND`  
    /// Performs bitwise `and`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn bit_and(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::BitAnd,
                dst.into(),
                lhs.into(),
                rhs.into(),
                K::ZERO,
            ),
            function: Self::execute_bit_and,
        }
    }

    /// `BOR`  
    /// Performs bitwise `or`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn bit_or(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::BitOr, dst.into(), lhs.into(), rhs.into(), K::ZERO),
            function: Self::execute_bit_or,
        }
    }

    /// `BXOR`  
    /// Performs bitwise `xor`.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn bit_xor(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::BitXor,
                dst.into(),
                lhs.into(),
                rhs.into(),
                K::ZERO,
            ),
            function: Self::execute_bit_xor,
        }
    }

    /// `SHL`  
    /// Performs bitwise shift left.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn shift_left(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::ShiftLeft,
                dst.into(),
                lhs.into(),
                rhs.into(),
                K::ZERO,
            ),
            function: Self::execute_shift_left,
        }
    }

    /// `SHR`  
    /// Performs bitwise shift right.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
    pub fn shift_right(dst: impl Into<A>, lhs: impl Into<B>, rhs: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::ShiftRight,
                dst.into(),
                lhs.into(),
                rhs.into(),
                K::ZERO,
            ),
            function: Self::execute_shift_right,
        }
    }

    /// `UNM`  
    /// Performs negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub fn neg(dst: impl Into<A>, rhs: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Neg, dst.into(), rhs.into(), C::ZERO, K::ZERO),
            function: Self::execute_neg,
        }
    }

    /// `BNOT`
    /// Performs bit-wise not.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub fn bit_not(dst: impl Into<A>, rhs: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::BitNot, dst.into(), rhs.into(), C::ZERO, K::ZERO),
            function: Self::execute_bit_not,
        }
    }

    /// `NOT`  
    /// Performs logical negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub fn not(dst: impl Into<A>, rhs: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Not, dst.into(), rhs.into(), C::ZERO, K::ZERO),
            function: Self::execute_not,
        }
    }

    /// `LEN`  
    /// Performs length calculation on String.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    pub fn len(dst: impl Into<A>, rhs: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Len, dst.into(), rhs.into(), C::ZERO, K::ZERO),
            function: Self::execute_len,
        }
    }

    /// `CONCAT`  
    /// Performs concatenation.
    ///
    /// `first`: Location on stack of first string, the result is
    /// stored here  
    /// `string_count`: Number of strings to concat
    pub fn concat(first: impl Into<A>, count: impl Into<B>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::Concat,
                first.into(),
                count.into(),
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_concat,
        }
    }

    /// `CLOSE`  
    /// Closes an upvalue at the end of a block.
    ///
    /// `first`: Location on stack of first register to be closed
    pub fn close(first: impl Into<A>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::Close, first.into(), B::ZERO, C::ZERO, K::ZERO),
            function: Self::execute_close,
        }
    }

    /// `JMP`  
    /// Performs jump.
    ///
    /// `jump`: Number of intructions to jump
    pub fn jump(jump: impl Into<Sj>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asj(OpCode::Jump, jump.into()),
            function: Self::execute_jump,
        }
    }
    /// `EQ`  
    /// Performs equal (==) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    pub fn equal(lst: impl Into<A>, rhs: impl Into<B>, test: impl Into<K>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::Equal,
                lst.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_equal,
        }
    }

    /// `LT`  
    /// Performs less than (<) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    pub fn less_than(lst: impl Into<A>, rhs: impl Into<B>, test: impl Into<K>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::LessThan,
                lst.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_less_than,
        }
    }

    /// `LE`  
    /// Performs less than or equal (<=) comparison between 2 registers.
    ///
    /// `lhs`: Location on stack of left operand  
    /// `rhs`: Location on stack of light operand  
    /// `test`: If it should test for `true` or `false`
    pub fn less_equal(lhs: impl Into<A>, rhs: impl Into<B>, test: impl Into<K>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::LessEqual,
                lhs.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_less_equal,
        }
    }

    /// `EQK`
    /// Peforms equal comparison (==) between the register and constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `constant`: Id of constant  
    /// `test`: If it should test for `true` or `false`
    pub fn equal_constant(lhs: impl Into<A>, rhs: impl Into<B>, test: impl Into<K>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::EqualConstant,
                lhs.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_equal_constant,
        }
    }

    /// `EQI`
    /// Peforms equal comparison (==) between the register and i8.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant  
    /// `test`: If it should test for `true` or `false`
    pub fn equal_integer(lhs: impl Into<A>, rhs: impl Into<Sb>, test: impl Into<K>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(
                OpCode::EqualInteger,
                lhs.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_equal_integer,
        }
    }

    /// `LTI`
    /// Peforms a less than (<) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    pub fn less_than_integer(
        lhs: impl Into<A>,
        rhs: impl Into<Sb>,
        test: impl Into<K>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(
                OpCode::LessThanInteger,
                lhs.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_less_than_integer,
        }
    }

    /// `GTI`
    /// Peforms a greater than (>) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    pub fn greater_than_integer(
        lhs: impl Into<A>,
        rhs: impl Into<Sb>,
        test: impl Into<K>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(
                OpCode::GreaterThanInteger,
                lhs.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_greater_than_integer,
        }
    }

    /// `GEI`
    /// Peforms a greater or equal (>=) comparison between the register and integer constant.
    ///
    /// `register`: Location on stack of left operand  
    /// `integer`: Integer constant of right operand  
    /// `test`: If it should test for `true` or `false`
    pub fn greater_equal_integer(
        lhs: impl Into<A>,
        rhs: impl Into<Sb>,
        test: impl Into<K>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_asbck(
                OpCode::GreaterEqualInteger,
                lhs.into(),
                rhs.into(),
                C::ZERO,
                test.into(),
            ),
            function: Self::execute_greater_equal_integer,
        }
    }

    /// `TEST`  
    /// Performs test.
    ///
    /// `register`: Location on stack of the value that if going to be tested  
    /// `test`: Test to perform  
    pub fn test(register: impl Into<A>, test: impl Into<K>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::Test,
                register.into(),
                B::ZERO,
                C::ZERO,
                test.into(),
            ),
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
    pub fn call(func: impl Into<A>, in_params: impl Into<B>, out_params: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::Call,
                func.into(),
                in_params.into(),
                out_params.into(),
                K::ZERO,
            ),
            function: Self::execute_call,
        }
    }

    /// `TAILCALL`  
    /// Calls a function
    ///
    /// `func`: Location on the stack where the function was loaded  
    /// `args`: Count of arguments  
    /// `variadics`: Number of variadic arguments
    pub fn tail_call(func: impl Into<A>, args: impl Into<B>, variadics: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::TailCall,
                func.into(),
                args.into(),
                variadics.into(),
                K::ZERO,
            ),
            function: Self::execute_tail_call,
        }
    }

    /// `RETURN`  
    /// Returns from function
    ///
    /// `register`: First item on stack to return  
    /// `count`: Number of items to return, the actual value is subtracting
    /// it by 1, if zero, return all items on stack  
    /// `varargs`: If `var_args` > 0, the function is has variadic arguments
    /// and the number of fixed arguments is `var_args - 1`
    pub fn return_bytecode(
        register: impl Into<A>,
        count: impl Into<B>,
        varargs: impl Into<C>,
    ) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::Return,
                register.into(),
                count.into(),
                varargs.into(),
                K::ZERO,
            ),
            function: Self::execute_return,
        }
    }

    /// `RETURN0`  
    /// Returns from function with 0 out values
    pub fn zero_return() -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(OpCode::ZeroReturn, A::ZERO, B::ZERO, C::ZERO, K::ZERO),
            function: Self::execute_zero_return,
        }
    }
    /// `RETURN1`  
    /// Returns from function with 1 out values
    ///
    /// `return`: Location on stack of the returned value
    pub fn one_return(register: impl Into<A>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::OneReturn,
                register.into(),
                B::ZERO,
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_one_return,
        }
    }

    /// `FORLOOP`  
    /// Increment counter and jumps back to start of loop
    ///
    /// `for`: Location on the stack counter information is stored  
    /// `jump`: Number of byte codes to jump to reach start of for block
    pub fn for_loop(counter: impl Into<A>, jump: impl Into<Bx>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::ForLoop, counter.into(), jump.into()),
            function: Self::execute_for_loop,
        }
    }

    /// `FORPREP`  
    /// Prepares for loop counter
    ///
    /// `for`: Location on the stack counter information is stored  
    /// `jump`: Number of byte codes to jump to reach end of for loop
    pub fn for_prepare(counter: impl Into<A>, jump: impl Into<Bx>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::ForPrepare, counter.into(), jump.into()),
            function: Self::execute_for_prepare,
        }
    }

    /// `SETLIST`  
    /// Stores multiple values from the stack into the table
    ///
    /// `table`: Location of the table on the stack  
    /// `array_len`: Number of items on the stack to store  
    /// `c`: ?
    pub fn set_list(table: impl Into<A>, array_len: impl Into<B>, c: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::SetList,
                table.into(),
                array_len.into(),
                c.into(),
                K::ZERO,
            ),
            function: Self::execute_set_list,
        }
    }

    /// `CLOSURE`
    /// Puts reference to a local function into the stack
    ///
    /// `dst`: Stack location to store function reference  
    /// `func_id`: Id of function
    pub fn closure(dst: impl Into<A>, func_id: impl Into<Bx>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abx(OpCode::Closure, dst.into(), func_id.into()),
            function: Self::execute_closure,
        }
    }

    /// `VARARG`
    /// Collect variable arguments
    ///
    /// `register`: First destination of variable arguments  
    /// `count`: Count of variable arguments, `0` means use all, other values
    /// are subtracted by `1`, i.e. the range is (register..(register + count - 1))
    pub fn variadic_arguments(register: impl Into<A>, count: impl Into<C>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::VariadicArguments,
                register.into(),
                B::ZERO,
                count.into(),
                K::ZERO,
            ),
            function: Self::execute_variadic_arguments,
        }
    }

    /// `VARARGPREP`
    /// Prepares the variadic arguments of the closure.
    ///
    /// `fixed`: Number of fixed arguments
    pub fn variadic_arguments_prepare(varargs: impl Into<A>) -> Bytecode {
        Bytecode {
            bytecode: Self::encode_abck(
                OpCode::VariadicArgumentsPrepare,
                varargs.into(),
                B::ZERO,
                C::ZERO,
                K::ZERO,
            ),
            function: Self::execute_variadic_arguments_prepare,
        }
    }

    fn execute_move(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, src, _, _) = self.decode_abck();
        let value = vm.get_stack(*src)?.clone();
        vm.set_stack(*dst, value)
    }

    fn execute_load_integer(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, value) = self.decode_asbx();
        vm.set_stack(*dst, Value::Integer(i64::from(*value)))
    }

    fn execute_load_float(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, value) = self.decode_asbx();
        vm.set_stack(*dst, Value::Float(*value as f64))
    }

    fn execute_load_constant(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, constant) = self.decode_abx();

        let closure = vm.get_running_closure();
        let value = closure.constant(usize::try_from(*constant)?)?;
        vm.set_stack(*dst, value)
    }

    fn execute_load_false(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, _, _, _) = self.decode_abck();
        vm.set_stack(*dst, Value::Boolean(false))
    }

    fn execute_load_false_skip(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, _, _, _) = self.decode_abck();
        vm.jump(1)?;
        vm.set_stack(*dst, Value::Boolean(false))
    }

    fn execute_load_true(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, _, _, _) = self.decode_abck();
        vm.set_stack(*dst, Value::Boolean(true))
    }

    fn execute_load_nil(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, extras, _, _) = self.decode_abck();
        // If `extra` is 0, runs once
        for dst in *dst..=(*dst + *extras) {
            vm.set_stack(dst, Value::Nil)?;
        }
        Ok(())
    }

    fn execute_get_upvalue(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, upvalue, _, _) = self.decode_abck();
        let upvalue = vm.get_upvalue(usize::from(*upvalue))?;
        vm.set_stack(*dst, upvalue)
    }

    fn execute_set_upvalue(&self, vm: &mut Lua) -> Result<(), Error> {
        let (value, upvalue, _, _) = self.decode_abck();

        let value = vm.get_stack(*value).cloned()?;
        vm.set_upvalue(usize::from(*upvalue), value)?;

        Ok(())
    }

    fn execute_get_uptable(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, upvalue, key, _) = self.decode_abck();

        let Value::Table(upvalue) = vm.get_upvalue(usize::from(*upvalue))? else {
            return Err(Error::ExpectedTable);
        };

        let closure = vm.get_running_closure();
        let key = closure.constant(usize::from(*key))?;
        let value = upvalue.deref().borrow().get(ValueKey(key.clone())).clone();

        vm.set_stack(*dst, value)
    }

    fn execute_get_table(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, table, src, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let key = vm.get_stack(*src)?.clone();
            let value = match key {
                Value::Integer(index @ 1..) => table
                    .borrow()
                    .array
                    .get(usize::try_from(index - 1)?)
                    .cloned()
                    .unwrap_or(Value::Nil),
                key => {
                    let key = ValueKey::from(key);
                    let bin_search = (*table)
                        .borrow()
                        .table
                        .binary_search_by_key(&&key, |a| &a.0);

                    match bin_search {
                        Ok(i) => (*table).borrow().table[i].1.clone(),
                        Err(_) => Value::Nil,
                    }
                }
            };
            vm.set_stack(*dst, value)
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_get_index(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, table, index, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let value = if *index == 0 {
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

    fn execute_get_field(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, table, key, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let closure = vm.get_running_closure();
            let key = ValueKey::from(closure.constant(usize::from(*key))?);
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

    fn execute_set_uptable(&self, vm: &mut Lua) -> Result<(), Error> {
        let (upvalue, key, src, constant) = self.decode_abck();

        let running_program = vm.get_running_closure();
        let key = running_program.constant(usize::from(*key))?;
        let value = if *constant {
            running_program.constant(usize::from(*src))?
        } else {
            vm.get_stack(*src)?.clone()
        };

        match vm.get_upvalue(usize::from(*upvalue))? {
            Value::Table(upvalue) => upvalue.borrow_mut().set(ValueKey(key), value),
            _ => Err(Error::ExpectedTable),
        }
    }

    fn execute_set_table(&self, vm: &mut Lua) -> Result<(), Error> {
        let (table, key, src, constant) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let program = vm.get_running_closure();
            let key = ValueKey::from(vm.get_stack(*key)?.clone());
            let value = if *constant {
                program.constant(usize::from(*src))?
            } else {
                vm.get_stack(*src)?.clone()
            };

            match key {
                ValueKey(Value::Integer(index)) if index > 0 => {
                    let array = &mut table.borrow_mut().array;
                    let index = usize::try_from(index)? - 1;
                    match index.cmp(&array.len()) {
                        Ordering::Less => array[index] = value,
                        Ordering::Equal => array.push(value),
                        Ordering::Greater => {
                            array.resize(index, Value::Nil);
                            array.push(value);
                        }
                    }
                }
                _ => {
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
                }
            }

            Ok(())
        } else {
            Err(Error::ExpectedTable)
        }
    }

    fn execute_set_field(&self, vm: &mut Lua) -> Result<(), Error> {
        let (table, key, src, constant) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(*table)?.clone() {
            let running_program = vm.get_running_closure();
            let key = ValueKey::from(running_program.constant(usize::from(*key))?);
            let value = if *constant {
                running_program.constant(usize::from(*src))?
            } else {
                vm.get_stack(*src)?.clone()
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

    fn execute_new_table(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, array_initial_size, table_initial_size, _) = self.decode_abck();

        vm.set_stack(
            *dst,
            Value::Table(Rc::new(RefCell::new(Table::new(
                usize::from(*array_initial_size),
                usize::from(*table_initial_size),
            )))),
        )
    }

    fn execute_table_self(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, table, key, _) = self.decode_abck();

        if let Value::Table(table) = vm.get_stack(*table).cloned()? {
            vm.set_stack(*dst + 1, Value::Table(table.clone()))?;

            let program = vm.get_running_closure();
            let key = ValueKey::from(program.constant(usize::from(*key))?);
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

    fn execute_add_integer(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, int, _) = self.decode_absck();

        let res = match &vm.get_stack(*lhs)? {
            Value::Integer(l) => Value::Integer(l + i64::from(*int)),
            Value::Float(l) => Value::Float(l + *int as f64),
            lhs => {
                return Err(Error::ArithmeticOperand(
                    "add",
                    lhs.static_type_name(),
                    "integer",
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_add_constant(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, constant, _) = self.decode_abck();

        let program = vm.get_running_closure();

        let res = match (
            &vm.get_stack(*lhs)?,
            &program.constant(usize::from(*constant))?,
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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_mul_constant(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, constant, _) = self.decode_abck();

        let program = vm.get_running_closure();

        let res = match (
            &vm.get_stack(*lhs)?,
            &program.constant(usize::from(*constant))?,
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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_add(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_sub(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_mul(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_mod(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_pow(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_div(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_idiv(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_bit_and(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l & r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "and",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_bit_or(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l | r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "or",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_bit_xor(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l ^ r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "xor",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_shift_left(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

        let res = match (&vm.get_stack(*lhs)?, &vm.get_stack(*rhs)?) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l << r),
            (lhs, rhs) => {
                return Err(Error::BitwiseOperand(
                    "shift left",
                    lhs.static_type_name(),
                    rhs.static_type_name(),
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_shift_right(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, lhs, rhs, _) = self.decode_abck();

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
                ));
            }
        };
        vm.set_stack(*dst, res)
    }

    fn execute_neg(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match vm.get_stack(*rhs)? {
            Value::Integer(integer) => Value::Integer(-integer),
            Value::Float(float) => Value::Float(-float),
            _ => return Err(Error::InvalidNegOperand),
        };
        vm.set_stack(*dst, value)
    }

    fn execute_bit_not(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match vm.get_stack(*rhs)? {
            Value::Integer(integer) => Value::Integer(!integer),
            _ => return Err(Error::InvalidBitNotOperand),
        };
        vm.set_stack(*dst, value)
    }

    fn execute_not(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match &vm.get_stack(*rhs)? {
            Value::Boolean(false) | Value::Nil => Value::Boolean(true),
            _ => Value::Boolean(false),
        };
        vm.set_stack(*dst, value)
    }

    fn execute_len(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, rhs, _, _) = self.decode_abck();

        let value = match &vm.get_stack(*rhs)? {
            Value::String(string) => Value::Integer(i64::try_from(string.len())?),
            Value::ShortString(string) => Value::Integer(i64::try_from(string.len())?),
            _ => return Err(Error::InvalidLenOperand),
        };
        vm.set_stack(*dst, value)
    }

    fn execute_concat(&self, vm: &mut Lua) -> Result<(), Error> {
        let (first, count, _, _) = self.decode_abck();

        let mut strings = Vec::with_capacity(usize::from(*count));

        for src in *first..(*first + *count) {
            match &vm.get_stack(src)? {
                Value::Integer(lhs) => strings.push(lhs.to_string()),
                Value::Float(lhs) => strings.push(lhs.to_string()),
                Value::ShortString(lhs) => strings.push(lhs.to_string()),
                Value::String(lhs) => strings.push(lhs.to_string()),
                other => return Err(Error::ConcatOperand(other.static_type_name())),
            };
        }

        let concatenated = strings.into_iter().collect::<String>();
        vm.set_stack(*first, concatenated.as_str().into())
    }

    fn execute_close(&self, vm: &mut Lua) -> Result<(), Error> {
        let (first, _, _, _) = self.decode_abck();

        let upvalues_to_close = vm
            .get_stack_frame_mut()
            .open_upvalues
            .iter()
            .enumerate()
            .filter_map(|(i, upvalue)| {
                if let Upvalue::Open(stack) = *upvalue.borrow() {
                    Some(i).filter(|_| stack > usize::from(*first))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for upvalue in upvalues_to_close.into_iter().rev() {
            vm.get_stack_frame_mut()
                .open_upvalues
                .swap_remove(upvalue)
                .borrow_mut()
                .close(vm);
        }

        Ok(())
    }

    fn execute_jump(&self, vm: &mut Lua) -> Result<(), Error> {
        let jump = self.decode_sj();

        vm.jump(isize::try_from(*jump)?)
    }

    fn execute_equal(&self, vm: &mut Lua) -> Result<(), Error> {
        let (lhs, rhs, _, test) = self.decode_abck();

        let lhs = vm.get_stack(*lhs)?;
        let rhs = vm.get_stack(*rhs)?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Equal, *test)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_less_than(&self, vm: &mut Lua) -> Result<(), Error> {
        let (lhs, rhs, _, test) = self.decode_abck();

        let lhs = vm.get_stack(*lhs)?;
        let rhs = vm.get_stack(*rhs)?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Less, *test)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_less_equal(&self, vm: &mut Lua) -> Result<(), Error> {
        let (lhs, rhs, _, test) = self.decode_abck();

        let lhs = &vm.get_stack(*lhs)?;
        let rhs = &vm.get_stack(*rhs)?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering != Ordering::Greater, *test)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_equal_constant(&self, vm: &mut Lua) -> Result<(), Error> {
        let (register, constant, _, test) = self.decode_abck();

        let program = vm.get_running_closure();

        let lhs = vm.get_stack(*register)?;
        let rhs = &program.constant(usize::from(*constant))?;

        Self::relational_comparison(lhs, rhs, |ordering| ordering == Ordering::Equal, *test)
            .and_then(|should_advance_pc| {
                if should_advance_pc {
                    vm.jump(1)?;
                }
                Ok(())
            })
    }

    fn execute_equal_integer(&self, vm: &mut Lua) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering == Ordering::Equal,
            test == K::ONE,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    fn execute_less_than_integer(&self, vm: &mut Lua) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering == Ordering::Less,
            test == K::ONE,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    fn execute_greater_than_integer(&self, vm: &mut Lua) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering == Ordering::Greater,
            test == K::ONE,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    fn execute_greater_equal_integer(&self, vm: &mut Lua) -> Result<(), Error> {
        let (register, integer, _, test) = self.decode_asbck();

        let lhs = vm.get_stack(*register)?;
        let rhs = Value::Integer(i64::from(*integer));

        Self::relational_comparison(
            lhs,
            &rhs,
            |ordering| ordering != Ordering::Less,
            test == K::ONE,
        )
        .and_then(|should_advance_pc| {
            if should_advance_pc {
                vm.jump(1)?;
            }
            Ok(())
        })
    }

    fn execute_test(&self, vm: &mut Lua) -> Result<(), Error> {
        let (src, _, _, test) = self.decode_abck();

        let cond = vm.get_stack(*src)?;
        match (cond, test) {
            (Value::Nil | Value::Boolean(false), K::ZERO) => (),
            (Value::Nil | Value::Boolean(false), K::ONE) => vm.jump(1)?,
            (_, K::ONE) => (),
            (_, K::ZERO) => vm.jump(1)?,
        };

        Ok(())
    }

    fn execute_call(&self, vm: &mut Lua) -> Result<(), Error> {
        let (func_index, in_items, out, _) = self.decode_abck();

        let func_index_usize = usize::from(*func_index);
        let in_items = usize::from(*in_items);
        let out_params = usize::from(*out);

        let func = vm.get_stack(*func_index)?.clone();
        Self::run_closure(func, vm, func_index_usize, in_items, out_params)?;

        // TODO deal with c
        Ok(())
    }

    fn execute_tail_call(&self, vm: &mut Lua) -> Result<(), Error> {
        let (func_index, args, out_params, _) = self.decode_abck();

        let func_index_usize = usize::from(*func_index);
        let args = usize::from(*args);
        let out_params = usize::from(*out_params);

        let top_stack = vm.get_stack_frame();
        let tail_start = top_stack.stack_frame + func_index_usize;
        let prev_func_index = top_stack.function_index;
        vm.drop_stack_frame(func_index_usize, vm.stack.len() - tail_start);

        let func = &vm.get_stack(u8::try_from(prev_func_index)?)?;
        if let Value::Closure(closure) = func {
            match closure.closure_type() {
                FunctionType::Native(closure) => {
                    Self::run_native_function(vm, prev_func_index, args, out_params, *closure)
                }
                FunctionType::Lua(closure) => {
                    let closure = closure.clone();
                    Self::setup_closure(vm, prev_func_index, args, out_params, closure.as_ref())
                }
            }
        } else {
            Err(Error::InvalidFunction((*func).clone()))
        }
    }

    fn execute_return(&self, vm: &mut Lua) -> Result<(), Error> {
        // TODO treat out params
        let (return_start, count, _, _) = self.decode_abck();
        vm.drop_stack_frame(usize::from(*return_start), usize::from(*count - 1));
        Ok(())
    }

    fn execute_zero_return(&self, vm: &mut Lua) -> Result<(), Error> {
        vm.drop_stack_frame(0, 0);
        Ok(())
    }

    fn execute_one_return(&self, vm: &mut Lua) -> Result<(), Error> {
        let (return_loc, _, _, _) = self.decode_abck();
        vm.drop_stack_frame(usize::from(*return_loc), 1);
        Ok(())
    }

    fn execute_for_loop(&self, vm: &mut Lua) -> Result<(), Error> {
        let (for_stack, jmp) = self.decode_abx();

        if let Value::Integer(counter) = &mut vm.get_stack_mut(*for_stack + 1)? {
            if counter != &0 {
                *counter -= 1;
                Bytecode::add(*for_stack + 3, *for_stack + 3, *for_stack + 2).execute(vm)?;
                vm.jump(-isize::try_from(*jmp)?)?;
            }
            Ok(())
        } else {
            log::error!("For loop counter should be an Integer.");
            Err(Error::ForZeroStep)
        }
    }

    fn execute_for_prepare(&self, vm: &mut Lua) -> Result<(), Error> {
        let (for_stack, jmp) = self.decode_abx();

        let init = vm.get_stack(*for_stack)?;
        let limit = vm.get_stack(*for_stack + 1)?;
        let step = vm.get_stack(*for_stack + 2)?;

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

            vm.set_stack(*for_stack + 1, Value::Integer(count))?;
            vm.set_stack(*for_stack + 3, Value::Integer(init))?;
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
            vm.set_stack(*for_stack + 1, Value::Integer(count as i64))?;
            vm.set_stack(*for_stack + 2, Value::Float(step))?;
            vm.set_stack(*for_stack + 3, Value::Float(init))?;
            if count <= 0.0 {
                vm.jump(isize::try_from(*jmp)? + 1)?;
            }
            Ok(())
        }
    }

    fn execute_set_list(&self, vm: &mut Lua) -> Result<(), Error> {
        let (table, count, _c, _) = self.decode_abck();

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

    fn execute_closure(&self, vm: &mut Lua) -> Result<(), Error> {
        let (dst, func_id) = self.decode_abx();
        let func_id = usize::try_from(*func_id)?;

        let program = vm.get_running_closure();

        let Ok(func) = program.function(func_id).inspect_err(|err| {
            log::error!("{}", err);
        }) else {
            return Err(Error::Expected(func_id, "closure", "nil"));
        };

        let upvalues = func
            .program()
            .upvalues
            .iter()
            .map(|upvalue| vm.find_upvalue(upvalue))
            .collect::<Result<Vec<_>, _>>()?;

        let closure = Value::Closure(Rc::new(Closure::new_lua(func, upvalues)));

        vm.set_stack(*dst, closure)
    }

    fn execute_variadic_arguments(&self, vm: &mut Lua) -> Result<(), Error> {
        let (register, _, count, _) = self.decode_abck();

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
            let true_count = usize::from(*count - 1);
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

    fn execute_variadic_arguments_prepare(&self, _vm: &mut Lua) -> Result<(), Error> {
        // Do nothing
        Ok(())
    }

    pub fn flip_test(&mut self) {
        let op = OpCode::read(self.bytecode);
        assert!(op.is_relational());
        let (a, b, c, test) = self.decode_abck();
        assert_eq!(c, C::ZERO);

        self.bytecode = Self::encode_abck(op, a, b, c, test.flip());
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

    fn run_closure(
        func: Value,
        vm: &mut Lua,
        func_index: usize,
        in_items: usize,
        out_params: usize,
    ) -> Result<(), Error> {
        if let Value::Closure(closure) = func {
            match closure.closure_type() {
                FunctionType::Native(closure) => {
                    Self::run_native_function(vm, func_index, in_items, out_params, *closure)
                }
                FunctionType::Lua(closure) => {
                    let closure = closure.clone();
                    Self::setup_closure(vm, func_index, in_items, out_params, closure.as_ref())
                }
            }
        } else {
            Err(Error::InvalidFunction(func))
        }
    }

    fn run_native_function(
        vm: &mut Lua,
        func_index: usize,
        args: usize,
        out_params: usize,
        func: NativeClosure,
    ) -> Result<(), Error> {
        log::trace!("Calling native function");

        let top_stack = vm.get_stack_frame();

        let args = if args == 0 {
            vm.stack.len() - (top_stack.stack_frame + top_stack.variadic_arguments + func_index) - 1
        } else {
            args - 1
        };

        vm.prepare_new_stack_frame(func_index, args, out_params, 0);

        let returns = func(vm)?;

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

        vm.prepare_new_stack_frame(func_index, args, out_params, var_args);

        Ok(())
    }

    pub(crate) fn encode_abck(op: OpCode, a: A, b: B, c: C, k: K) -> u32 {
        let mut bytecode = 0;
        op.write(&mut bytecode);
        a.write(&mut bytecode);
        b.write(&mut bytecode);
        c.write(&mut bytecode);
        k.write(&mut bytecode);
        bytecode
    }

    pub(crate) fn encode_asbck(op: OpCode, a: A, sb: Sb, c: C, k: K) -> u32 {
        let mut bytecode = 0;
        op.write(&mut bytecode);
        a.write(&mut bytecode);
        sb.write(&mut bytecode);
        c.write(&mut bytecode);
        k.write(&mut bytecode);
        bytecode
    }

    pub(crate) fn encode_absck(op: OpCode, a: A, b: B, sc: Sc, k: K) -> u32 {
        let mut bytecode = 0;
        op.write(&mut bytecode);
        a.write(&mut bytecode);
        b.write(&mut bytecode);
        sc.write(&mut bytecode);
        k.write(&mut bytecode);
        bytecode
    }

    pub(crate) fn encode_abx(op: OpCode, a: A, bx: Bx) -> u32 {
        let mut bytecode = 0;
        op.write(&mut bytecode);
        a.write(&mut bytecode);
        bx.write(&mut bytecode);
        bytecode
    }

    pub(crate) fn encode_asbx(op: OpCode, a: A, sbx: Sbx) -> u32 {
        let mut bytecode = 0;
        op.write(&mut bytecode);
        a.write(&mut bytecode);
        sbx.write(&mut bytecode);
        bytecode
    }

    pub(crate) fn encode_asj(op: OpCode, j: Sj) -> u32 {
        let mut bytecode = 0;
        op.write(&mut bytecode);
        j.write(&mut bytecode);
        bytecode
    }

    pub(crate) fn decode_abck(&self) -> (A, B, C, K) {
        (
            A::read(self.bytecode),
            B::read(self.bytecode),
            C::read(self.bytecode),
            K::read(self.bytecode),
        )
    }

    pub(crate) fn decode_asbck(&self) -> (A, Sb, C, K) {
        (
            A::read(self.bytecode),
            Sb::read(self.bytecode),
            C::read(self.bytecode),
            K::read(self.bytecode),
        )
    }

    pub(crate) fn decode_absck(&self) -> (A, B, Sc, K) {
        (
            A::read(self.bytecode),
            B::read(self.bytecode),
            Sc::read(self.bytecode),
            K::read(self.bytecode),
        )
    }

    pub(crate) fn decode_abx(&self) -> (A, Bx) {
        (A::read(self.bytecode), Bx::read(self.bytecode))
    }

    pub(crate) fn decode_asbx(&self) -> (A, Sbx) {
        (A::read(self.bytecode), Sbx::read(self.bytecode))
    }

    pub(crate) fn decode_ax(&self) -> Ax {
        Ax::read(self.bytecode)
    }

    pub(crate) fn decode_sj(&self) -> Sj {
        Sj::read(self.bytecode)
    }
}

impl Deref for Bytecode {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.bytecode
    }
}

impl Debug for Bytecode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let op = OpCode::read(self.bytecode);
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
                write!(f, "{:?}({})", op, *a)
            }
            OpCode::Test => {
                let (a, _, _, k) = self.decode_abck();
                write!(f, "{:?}({}, {})", op, *a, k == K::ONE)
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
                write!(f, "{:?}({}, {})", op, *a, *b)
            }
            OpCode::Equal
            | OpCode::LessThan
            | OpCode::LessEqual
            | OpCode::EqualConstant
            | OpCode::TestSet => {
                let (a, b, _, k) = self.decode_abck();
                write!(
                    f,
                    "{:?}({}, {}{})",
                    op,
                    *a,
                    *b,
                    if k == K::ONE { "k" } else { "" }
                )
            }
            OpCode::EqualInteger
            | OpCode::LessThanInteger
            | OpCode::LessEqualInteger
            | OpCode::GreaterThanInteger
            | OpCode::GreaterEqualInteger => {
                let (a, sb, _, k) = self.decode_asbck();
                write!(
                    f,
                    "{:?}({}, {}{})",
                    op,
                    *a,
                    *sb,
                    if k == K::ONE { "k" } else { "" }
                )
            }
            OpCode::LoadConstant
            | OpCode::ForPrepare
            | OpCode::ForLoop
            | OpCode::GenericForPrepare
            | OpCode::GenericForLoop
            | OpCode::Closure => {
                let (a, bx) = self.decode_abx();
                write!(f, "{:?}({}, {})", op, *a, *bx)
            }
            OpCode::LoadInteger | OpCode::LoadFloat => {
                let (a, sbx) = self.decode_asbx();
                write!(f, "{:?}({}, {})", op, *a, *sbx)
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
                write!(f, "{:?}({}, {}, {})", op, *a, *b, *c)
            }
            OpCode::MetaMethodInteger => {
                let (a, sb, c, _) = self.decode_asbck();
                write!(f, "{:?}({}, {}, {})", op, *a, *sb, *c)
            }
            OpCode::AddInteger | OpCode::ShiftRightInteger | OpCode::ShiftLeftInteger => {
                let (a, b, sc, _) = self.decode_absck();
                write!(f, "{:?}({}, {}, {})", op, *a, *b, *sc)
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
                write!(
                    f,
                    "{:?}({}, {}, {}{})",
                    op,
                    *a,
                    *b,
                    *c,
                    if k == K::ONE { "k" } else { "" }
                )
            }
            OpCode::VariadicArguments | OpCode::GenericForCall => {
                let (a, _, c, _) = self.decode_abck();
                write!(f, "{:?}({}, {})", op, *a, *c)
            }
            OpCode::ExtraArguments => {
                let ax = self.decode_ax();
                write!(f, "{:?}({})", op, *ax)
            }
            OpCode::Jump => {
                let sj = self.decode_sj();
                write!(f, "{:?}({})", op, *sj)
            }
        }
    }
}

impl Display for Bytecode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self, f)
    }
}
