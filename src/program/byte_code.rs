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
    /// `rhs`: Location on stack of right-hand operand
    AddInteger(u8, u8, u8),
    /// `ADDK`  
    /// Performs arithmetic addition.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `lhs`: Location on stack of left-hand operand  
    /// `rhs`: Location on stack of right-hand operand
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
