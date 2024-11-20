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
