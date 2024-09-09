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

static OP_TABLE: LazyLock<HashMap<&'static str, Op>> = LazyLock::new(|| {
    HashMap::from([
        (
            "adc",
            Op::new([
                (AMode::Imm, 105),
                (AMode::Zp, 101),
                (AMode::ZpX, 117),
                (AMode::Abs, 109),
                (AMode::AbsX, 125),
                (AMode::AbsY, 121),
                (AMode::IndX, 97),
                (AMode::IndY, 113),
            ]),
        ),
        (
            "and",
            Op::new([
                (AMode::Imm, 41),
                (AMode::Zp, 37),
                (AMode::ZpX, 53),
                (AMode::Abs, 45),
                (AMode::AbsX, 61),
                (AMode::AbsY, 57),
                (AMode::IndX, 33),
                (AMode::IndY, 49),
            ]),
        ),
        (
            "asl",
            Op::new([
                (AMode::Imp, 10),
                (AMode::Zp, 6),
                (AMode::ZpX, 22),
                (AMode::Abs, 14),
                (AMode::AbsX, 30),
            ]),
        ),
        ("bit", Op::new([(AMode::Zp, 36), (AMode::Abs, 44)])),
        ("bpl", Op::new([(AMode::Imp, 16)])),
        ("bmi", Op::new([(AMode::Imp, 48)])),
        ("bvc", Op::new([(AMode::Imp, 80)])),
        ("bvs", Op::new([(AMode::Imp, 112)])),
        ("bcc", Op::new([(AMode::Imp, 144)])),
        ("bcs", Op::new([(AMode::Imp, 176)])),
        ("bne", Op::new([(AMode::Imp, 208)])),
        ("beq", Op::new([(AMode::Imp, 240)])),
        ("brk", Op::new([(AMode::Imp, 0)])),
        (
            "cmp",
            Op::new([
                (AMode::Imm, 201),
                (AMode::Zp, 197),
                (AMode::ZpX, 213),
                (AMode::Abs, 205),
                (AMode::AbsX, 221),
                (AMode::AbsY, 217),
                (AMode::IndX, 193),
                (AMode::IndY, 209),
            ]),
        ),
        (
            "cpx",
            Op::new([(AMode::Imm, 224), (AMode::Zp, 228), (AMode::Abs, 236)]),
        ),
        (
            "cpy",
            Op::new([(AMode::Imm, 192), (AMode::Zp, 196), (AMode::Abs, 204)]),
        ),
        (
            "dec",
            Op::new([
                (AMode::Zp, 198),
                (AMode::ZpX, 214),
                (AMode::Abs, 206),
                (AMode::AbsX, 222),
            ]),
        ),
        (
            "eor",
            Op::new([
                (AMode::Imm, 73),
                (AMode::Zp, 69),
                (AMode::ZpX, 85),
                (AMode::Abs, 77),
                (AMode::AbsX, 93),
                (AMode::AbsY, 89),
                (AMode::IndX, 65),
                (AMode::IndY, 81),
            ]),
        ),
        ("clc", Op::new([(AMode::Imp, 24)])),
        ("sec", Op::new([(AMode::Imp, 56)])),
        ("cli", Op::new([(AMode::Imp, 88)])),
        ("sei", Op::new([(AMode::Imp, 120)])),
        ("clv", Op::new([(AMode::Imp, 184)])),
        ("cld", Op::new([(AMode::Imp, 216)])),
        ("sed", Op::new([(AMode::Imp, 248)])),
        (
            "inc",
            Op::new([
                (AMode::Zp, 230),
                (AMode::ZpX, 246),
                (AMode::Abs, 238),
                (AMode::AbsX, 254),
            ]),
        ),
        ("jmp", Op::new([(AMode::Abs, 76), (AMode::Ind, 108)])),
        ("jsr", Op::new([(AMode::Abs, 32)])),
        (
            "lda",
            Op::new([
                (AMode::Imm, 169),
                (AMode::Zp, 165),
                (AMode::ZpX, 181),
                (AMode::Abs, 173),
                (AMode::AbsX, 189),
                (AMode::AbsY, 185),
                (AMode::IndX, 161),
                (AMode::IndY, 177),
            ]),
        ),
        (
            "ldx",
            Op::new([
                (AMode::Imm, 162),
                (AMode::Zp, 166),
                (AMode::ZpY, 182),
                (AMode::Abs, 174),
                (AMode::AbsY, 190),
            ]),
        ),
        (
            "ldy",
            Op::new([
                (AMode::Imm, 160),
                (AMode::Zp, 164),
                (AMode::ZpX, 180),
                (AMode::Abs, 172),
                (AMode::AbsX, 188),
            ]),
        ),
        (
            "lsr",
            Op::new([
                (AMode::Imp, 74),
                (AMode::Zp, 70),
                (AMode::ZpX, 86),
                (AMode::Abs, 78),
                (AMode::AbsX, 94),
            ]),
        ),
        ("nop", Op::new([(AMode::Imp, 234)])),
        (
            "ora",
            Op::new([
                (AMode::Imm, 9),
                (AMode::Zp, 5),
                (AMode::ZpX, 21),
                (AMode::Abs, 13),
                (AMode::AbsX, 29),
                (AMode::AbsY, 25),
                (AMode::IndX, 1),
                (AMode::IndY, 17),
            ]),
        ),
        ("tax", Op::new([(AMode::Imp, 170)])),
        ("txa", Op::new([(AMode::Imp, 138)])),
        ("dex", Op::new([(AMode::Imp, 202)])),
        ("inx", Op::new([(AMode::Imp, 232)])),
        ("tay", Op::new([(AMode::Imp, 168)])),
        ("tya", Op::new([(AMode::Imp, 152)])),
        ("dey", Op::new([(AMode::Imp, 136)])),
        ("iny", Op::new([(AMode::Imp, 200)])),
        (
            "rol",
            Op::new([
                (AMode::Imp, 42),
                (AMode::Zp, 38),
                (AMode::ZpX, 54),
                (AMode::Abs, 46),
                (AMode::AbsX, 62),
            ]),
        ),
        (
            "ror",
            Op::new([
                (AMode::Imp, 106),
                (AMode::Zp, 102),
                (AMode::ZpX, 118),
                (AMode::Abs, 110),
                (AMode::AbsX, 126),
            ]),
        ),
        ("rti", Op::new([(AMode::Imp, 64)])),
        ("rts", Op::new([(AMode::Imp, 96)])),
        (
            "sbc",
            Op::new([
                (AMode::Imm, 233),
                (AMode::Zp, 229),
                (AMode::ZpX, 245),
                (AMode::Abs, 237),
                (AMode::AbsX, 253),
                (AMode::AbsY, 249),
                (AMode::IndX, 225),
                (AMode::IndY, 241),
            ]),
        ),
        (
            "sta",
            Op::new([
                (AMode::Zp, 133),
                (AMode::ZpX, 149),
                (AMode::Abs, 141),
                (AMode::AbsX, 157),
                (AMode::AbsY, 153),
                (AMode::IndX, 129),
                (AMode::IndY, 145),
            ]),
        ),
        ("txs", Op::new([(AMode::Imp, 154)])),
        ("tsx", Op::new([(AMode::Imp, 186)])),
        ("pha", Op::new([(AMode::Imp, 72)])),
        ("pla", Op::new([(AMode::Imp, 104)])),
        ("php", Op::new([(AMode::Imp, 8)])),
        ("plp", Op::new([(AMode::Imp, 40)])),
        (
            "stx",
            Op::new([(AMode::Zp, 134), (AMode::ZpY, 150), (AMode::Abs, 142)]),
        ),
        (
            "sty",
            Op::new([(AMode::Zp, 132), (AMode::ZpX, 148), (AMode::Abs, 140)]),
        ),
    ])
});

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
