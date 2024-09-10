//! Pseudo-Op support.

use crate::{
    action::Action,
    asm::Assembler,
    expr::{ExLab, ExprNode},
    source::LineSlice,
};

/// Indicates a pseudo-op.
pub struct PseudoOp {
    op_name: LineSlice,
    #[allow(clippy::vec_box)]
    args: Vec<Box<ExprNode>>,
}

impl PseudoOp {
    #[allow(clippy::vec_box)]
    pub fn new(op_name: LineSlice, args: Vec<Box<ExprNode>>) -> Self {
        Self { op_name, args }
    }

    fn arg_count_err<T>(&self) -> Result<T, String> {
        self.line_slice().err("incorrect number of arguments")
    }

    /// If the expression tree is a string node, return that.
    fn is_str_arg(arg: &ExprNode) -> Option<&str> {
        match &arg.label {
            ExLab::Expr(e) => Self::is_str_arg(e),
            ExLab::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl Action for PseudoOp {
    fn pass1(
        &self,
        assembler: &mut Assembler,
        label: &Option<Box<LineSlice>>,
    ) -> Result<u16, String> {
        let name = self.op_name.text().to_ascii_lowercase();
        match name.as_str() {
            "=" => {
                if self.args.len() != 1 {
                    return self.arg_count_err();
                }
                if let Some(label) = label {
                    let value = Some(self.args[0].eval(assembler)?);
                    let sym = assembler.lookup(label.text(), label);
                    sym.defined_at = Some(*label.clone());
                    sym.value = value;
                    Ok(0)
                } else {
                    self.line_slice().err("missing label for '='")
                }
            }
            ".org" => {
                if self.args.len() == 1 {
                    let val = self.args[0].eval(assembler)?;
                    assembler.program_counter = Some(val);
                    Ok(0)
                } else {
                    self.arg_count_err()
                }
            }
            ".byte" => {
                let mut sum = 0;
                for arg in &self.args {
                    if let Some(s) = Self::is_str_arg(arg) {
                        sum += s.len() as u16;
                    } else {
                        sum += 1;
                    }
                }
                Ok(sum)
            }
            ".word" => Ok((self.args.len() * 2) as u16),
            _ => self
                .line_slice()
                .err(&format!("bad pseudo-op '{}'", self.op_name.text())),
        }
    }

    fn pass2(&self, assembler: &mut Assembler) -> Result<Vec<u8>, String> {
        let name = self.op_name.text().to_ascii_lowercase();
        match name.as_str() {
            "=" => Ok(vec![]),
            ".org" => self.pass1(assembler, &None).map(|_| vec![]),
            ".byte" => {
                let mut bytes = Vec::with_capacity(self.args.len());
                for arg in &self.args {
                    if let Some(s) = Self::is_str_arg(arg) {
                        bytes.extend(s.bytes());
                    } else {
                        bytes.push(arg.eval(assembler)?.to_le_bytes()[0]);
                    }
                }
                Ok(bytes)
            }
            ".word" => {
                let mut bytes = Vec::with_capacity(self.args.len() * 2);
                for arg in &self.args {
                    bytes.extend(arg.eval(assembler)?.to_le_bytes());
                }
                Ok(bytes)
            }
            _ => self.pass1(assembler, &None).map(|_| vec![]),
        }
    }

    fn line_slice(&self) -> LineSlice {
        if let Some(last_arg) = self.args.last() {
            self.op_name.join(&last_arg.slice)
        } else {
            self.op_name.clone()
        }
    }
}