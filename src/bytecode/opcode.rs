use super::arguments::BytecodeArgument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Move,
    LoadInteger,
    LoadFloat,
    LoadConstant,
    LoadConstantExtraArgs,
    LoadFalse,
    LoadFalseSkip,
    LoadTrue,
    LoadNil,
    GetUpValue,
    SetUpValue,
    GetUpTable,
    GetTable,
    GetIndex,
    GetField,
    SetUpTable,
    SetTable,
    SetIndex,
    SetField,
    NewTable,
    TableSelf,
    AddInteger,
    AddConstant,
    SubConstant,
    MulConstant,
    ModConstant,
    PowConstant,
    DivConstant,
    IDivConstant,
    BitAndConstant,
    BitOrConstant,
    BitXorConstant,
    ShiftRightInteger,
    ShiftLeftInteger,
    Add,
    Sub,
    Mul,
    Mod,
    Pow,
    Div,
    IDiv,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    MetaMethod,
    MetaMethodInteger,
    MetaMethodConstant,
    Neg,
    BitNot,
    Not,
    Len,
    Concat,
    Close,
    ToBeClosed,
    Jump,
    Equal,
    LessThan,
    LessEqual,
    EqualConstant,
    EqualInteger,
    LessThanInteger,
    LessEqualInteger,
    GreaterThanInteger,
    GreaterEqualInteger,
    Test,
    TestSet,
    Call,
    TailCall,
    Return,
    ZeroReturn,
    OneReturn,
    ForLoop,
    ForPrepare,
    TailForPrepare,
    TailForCall,
    TailForLoop,
    SetList,
    Closure,
    VariadicArguments,
    VariadicArgumentsPrepare,
    ExtraArguments,
}

impl OpCode {
    pub const fn from_id(id: u8) -> Self {
        match id {
            0 => Self::Move,
            1 => Self::LoadInteger,
            2 => Self::LoadFloat,
            3 => Self::LoadConstant,
            4 => Self::LoadConstantExtraArgs,
            5 => Self::LoadFalse,
            6 => Self::LoadFalseSkip,
            7 => Self::LoadTrue,
            8 => Self::LoadNil,
            9 => Self::GetUpValue,
            10 => Self::SetUpValue,
            11 => Self::GetUpTable,
            12 => Self::GetTable,
            13 => Self::GetIndex,
            14 => Self::GetField,
            15 => Self::SetUpTable,
            16 => Self::SetTable,
            17 => Self::SetIndex,
            18 => Self::SetField,
            19 => Self::NewTable,
            20 => Self::TableSelf,
            21 => Self::AddInteger,
            22 => Self::AddConstant,
            23 => Self::SubConstant,
            24 => Self::MulConstant,
            25 => Self::ModConstant,
            26 => Self::PowConstant,
            27 => Self::DivConstant,
            28 => Self::IDivConstant,
            29 => Self::BitAndConstant,
            30 => Self::BitOrConstant,
            31 => Self::BitXorConstant,
            32 => Self::ShiftRightInteger,
            33 => Self::ShiftLeftInteger,
            34 => Self::Add,
            35 => Self::Sub,
            36 => Self::Mul,
            37 => Self::Mod,
            38 => Self::Pow,
            39 => Self::Div,
            40 => Self::IDiv,
            41 => Self::BitAnd,
            42 => Self::BitOr,
            43 => Self::BitXor,
            44 => Self::ShiftLeft,
            45 => Self::ShiftRight,
            46 => Self::MetaMethod,
            47 => Self::MetaMethodInteger,
            48 => Self::MetaMethodConstant,
            49 => Self::Neg,
            50 => Self::BitNot,
            51 => Self::Not,
            52 => Self::Len,
            53 => Self::Concat,
            54 => Self::Close,
            55 => Self::ToBeClosed,
            56 => Self::Jump,
            57 => Self::Equal,
            58 => Self::LessThan,
            59 => Self::LessEqual,
            60 => Self::EqualConstant,
            61 => Self::EqualInteger,
            62 => Self::LessThanInteger,
            63 => Self::LessEqualInteger,
            64 => Self::GreaterThanInteger,
            65 => Self::GreaterEqualInteger,
            66 => Self::Test,
            67 => Self::TestSet,
            68 => Self::Call,
            69 => Self::TailCall,
            70 => Self::Return,
            71 => Self::ZeroReturn,
            72 => Self::OneReturn,
            73 => Self::ForLoop,
            74 => Self::ForPrepare,
            75 => Self::TailForPrepare,
            76 => Self::TailForCall,
            77 => Self::TailForLoop,
            78 => Self::SetList,
            79 => Self::Closure,
            80 => Self::VariadicArguments,
            81 => Self::VariadicArgumentsPrepare,
            82 => Self::ExtraArguments,
            _ => panic!("Invalid OpCode id"),
        }
    }

    pub fn is_relational(&self) -> bool {
        // TODO add missing opcodes
        matches!(
            self,
            OpCode::EqualConstant
                | OpCode::EqualInteger
                | OpCode::LessThan
                | OpCode::LessEqual
                | OpCode::GreaterThanInteger
                | OpCode::GreaterEqualInteger
        )
    }
}

impl BytecodeArgument for OpCode {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= *self as u32;
    }

    fn read(bytecode: u32) -> Self {
        Self::from_id((bytecode & 0x7f) as u8)
    }
}
