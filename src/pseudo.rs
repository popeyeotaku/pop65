//! Pseudo-Op support.

use crate::{action::Action, asm::Assembler, expr::ExprNode, source::LineSlice};

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
}

impl Action for PseudoOp {
    fn pass1(
        &self,
        assembler: &mut Assembler,
        label: &Option<Box<LineSlice>>,
    ) -> Result<u16, String> {
        let name = self.op_name.text().to_ascii_lowercase();
        match name.as_str() {
            ".org" => {
                if self.args.len() == 1 {
                    let val = self.args[0].eval(assembler)?;
                    assembler.program_counter = Some(val);
                    Ok(0)
                } else {
                    self.arg_count_err()
                }
            }
            _ => self
                .line_slice()
                .err(&format!("bad pseudo-op '{}'", self.op_name.text())),
        }
    }

    fn pass2(&self, assembler: &mut Assembler) -> Result<Vec<u8>, String> {
        let name = self.op_name.text().to_ascii_lowercase();
        match name.as_str() {
            ".org" => self.pass1(assembler, &None).map(|_| vec![]),
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
