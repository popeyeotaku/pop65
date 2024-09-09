//! Opcode support.

use std::{collections::HashMap, ops::Deref, sync::LazyLock};

use crate::{action::Action, asm::Assembler, expr::ExprNode, source::LineSlice};

/// A 6502 addressing mode.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum AMode {
    Imm,
    Imp,
    Zp,
    ZpX,
    ZpY,
    Abs,
    AbsX,
    AbsY,
    Ind,
    IndX,
    IndY,
}

impl AMode {
    /// Return the size of this addressing mode in bytes.
    pub fn byte_size(&self) -> u8 {
        match self {
            AMode::Imm => 2,
            AMode::Imp => 1,
            AMode::Zp => 2,
            AMode::ZpX => 2,
            AMode::ZpY => 2,
            AMode::Abs => 3,
            AMode::AbsX => 3,
            AMode::AbsY => 3,
            AMode::Ind => 2,
            AMode::IndX => 2,
            AMode::IndY => 2,
        }
    }
}

/// A 6502 opcode.
pub struct Op {
    op_bytes: HashMap<AMode, u8>,
}

impl Op {
    pub fn new<T: IntoIterator<Item = (AMode, u8)>>(op_bytes: T) -> Self {
        Op {
            op_bytes: HashMap::from_iter(op_bytes),
        }
    }
}

static OP_TABLE: LazyLock<HashMap<&'static str, Op>> = LazyLock::new(|| HashMap::from([]));

/// Lookup an opcode in the op table.
pub fn find_op(op_name: &str) -> Option<&'static Op> {
    OP_TABLE.deref().get(op_name)
}

/// A 6502 opcode in the actual source code.
pub struct OpCode {
    op: &'static Op,
    op_slice: LineSlice,
    amode: AMode,
    expr: Option<Box<ExprNode>>,
}

impl OpCode {
    pub fn new(
        op: &'static Op,
        op_slice: LineSlice,
        amode: AMode,
        expr: Option<Box<ExprNode>>,
    ) -> Self {
        Self {
            op,
            op_slice,
            amode,
            expr,
        }
    }

    /// Construct the *real* addressing mode, taking into account zero-page, etc.
    fn real_amode(&self) -> AMode {
        todo!()
    }

    /// Evaluate our expression, if any, and return its bytes corresponding to
    /// the given amode.
    fn eval(&self, amode: AMode, asm: &mut Assembler) -> Result<Vec<u8>, String> {
        if let Some(expr) = self.expr.as_ref() {
            let val = expr.eval(asm)?;
            let val_bytes = Vec::from(val.to_le_bytes());
            todo!();
            Ok(val_bytes)
        } else {
            Ok(Vec::new())
        }
    }
}

impl Action for OpCode {
    fn pass1(
        &self,
        assembler: &mut crate::asm::Assembler,
        label: &Option<Box<LineSlice>>,
    ) -> Result<u16, String> {
        Ok(self.real_amode().byte_size() as u16)
    }

    fn pass2(&self, assembler: &mut crate::asm::Assembler) -> Result<Vec<u8>, String> {
        let amode = self.real_amode();
        let mut bytes = self.eval(amode, assembler)?;
        bytes.insert(0, self.op.op_bytes[&amode]);
        Ok(bytes)
    }

    fn line_slice(&self) -> LineSlice {
        if let Some(expr) = self.expr.as_ref() {
            self.op_slice.join(&expr.slice)
        } else {
            self.op_slice.clone()
        }
    }
}
