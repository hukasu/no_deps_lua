#[derive(Debug, PartialEq)]
pub enum ByteCode {
    /// Gets a value from `globals` and place it on stack
    ///
    /// `dst`: Location on stack to place the global  
    /// `name`: Location on `constants` where the name of the
    /// global resides
    GetGlobal(u8, u8),
    /// Sets a value a global with a value from the stack
    ///
    /// `name`: Location on `constants` where the name of the
    /// global resides  
    /// `src`: Location of the value on stack to set the global
    SetGlobal(u8, u8),
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
    /// Loads the value of a constant into the stack
    ///
    /// `dst`: Location on the stack to place constant  
    /// `src`: Location of the value on `constants` to load into stack
    LoadConstant(u8, u8),
    /// Loads a `nil` into the stack
    ///
    /// `dst`: Location on the stack to place nil  
    LoadNil(u8),
    /// Loads a `bool` into the stack
    ///
    /// `dst`: Location on the stack to place boolean  
    /// `value`: Boolean value to load into stack
    LoadBool(u8, bool),
    /// Loads a `integer` into the stack
    ///
    /// `dst`: Location on the stack to place integer  
    /// `value`: Integer value to load into stack, this is limited
    /// to a i16
    LoadInt(u8, i16),
    /// Creates a new table value
    ///
    /// `dst`: Location on the stack to store the table  
    /// `array_len`: Amount of items to allocate on the list  
    /// `table_len`: Amount of items to allocate for the map
    NewTable(u8, u8, u8),
    /// Sets a table field to a value using a value
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location on the stack of the value that will be used
    /// as key  
    /// `value`: Location of the value on the stack
    SetTable(u8, u8, u8),
    /// Sets a table field to a value using a name
    ///
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    /// `value`: Location of the value on the stack
    SetField(u8, u8, u8),
    /// Stores multiple values from the stack into the table
    ///
    /// `table`: Location of the table on the stack  
    /// `array_len`: Number of items on the stack to store
    SetList(u8, u8),
    /// Loads a table field to the stack using a stack value
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `src`: Location of the name on the stack  
    GetTable(u8, u8, u8),
    /// Loads a table field to the stack using a name
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `key`: Location of the name on `constants`  
    GetField(u8, u8, u8),
    /// Loads a value from the table into the stack using integer index
    ///
    /// `dst`: Location on the stack to store the table's value  
    /// `table`: Location of the table on the stack  
    /// `index`: Index of the item to load
    GetInt(u8, u8, u8),
    /// Performs logical negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    Not(u8, u8),
    /// Performs length calculation on String.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    Len(u8, u8),
    /// Performs negation.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    Neg(u8, u8),
    /// Performs bit-wise not.
    ///
    /// `dst`: Location on stack to store result of operation  
    /// `src`: Location on stack to load value
    BitNot(u8, u8),
    /// Moves a value from one location on the stack to another
    ///
    /// `dst`: Location on the stack to store the value  
    /// `src`: Location on the stack to load the value
    Move(u8, u8),
    /// Calls a function
    ///
    /// `func`: Location on the stack where the function was loaded  
    /// `args`: Count of arguments
    Call(u8, u8),
}
