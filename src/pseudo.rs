//! Pseudo-Op support.

use std::{fs, rc::Rc};

use crate::{
    action::Action,
    asm::Assembler,
    expr::{ExLab, ExprNode},
    source::{self, LineSlice},
};

/// Indicates a pseudo-op.
pub struct PseudoOp {
    op_name: Rc<LineSlice>,
    op_name_lcase: String,
    #[allow(clippy::vec_box)]
    args: Vec<Box<ExprNode>>,
}

impl PseudoOp {
    #[allow(clippy::vec_box)]
    pub fn new(op_name: Rc<LineSlice>, args: Vec<Box<ExprNode>>) -> Self {
        let op_name_lcase = op_name.clone().text().to_ascii_lowercase();
        Self {
            op_name,
            args,
            op_name_lcase,
        }
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
        label: Option<Rc<LineSlice>>,
    ) -> Result<u16, String> {
        match self.op_name_lcase.as_str() {
            ".dbg" => {
                if self.args.is_empty() {
                    assembler.debug_fmt = None;
                    Ok(0)
                } else if self.args.len() != 1 {
                    self.arg_count_err()
                } else if let Some(s) = Self::is_str_arg(&self.args[0]) {
                    assembler.debug_fmt = Some(s.to_string());
                    Ok(0)
                } else {
                    self.line_slice().err("expected string argument")
                }
            }
            ".ds" => match self.args.len() {
                1 | 2 => Ok(self.args[0].eval(assembler)?),
                _ => self.line_slice().err("Expected one or two args"),
            },
            ".bin" | ".incbin" => {
                let bytes = self.pass2(assembler)?;
                Ok(bytes.len() as u16)
            }
            ".inc" | ".lib" | ".fil" => {
                for arg in &self.args {
                    if let Some(path) = Self::is_str_arg(arg) {
                        match source::from_file(path) {
                            Ok(src) => assembler.src_stk.push(src),
                            Err(e) => {
                                return self
                                    .line_slice()
                                    .err(&format!("Error including '{}': {}", path, e))
                            }
                        }
                    } else {
                        return self.line_slice().err("Missing string for include path");
                    }
                }
                Ok(0)
            }
            "=" | ".equ" => {
                if self.args.len() != 1 {
                    return self.arg_count_err();
                }
                if let Some(label) = label {
                    let value = self.args[0].eval(assembler)?;
                    assembler.def_symbol(label.clone().text(), label.clone(), value)?;
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
        match self.op_name_lcase.as_str() {
            ".ds" => match self.args.len() {
                1 => Ok(vec![0; self.args[0].eval(assembler)? as usize]),
                2 => Ok(vec![
                    self.args[1].eval(assembler)?.to_le_bytes()[0];
                    self.args[0].eval(assembler)? as usize
                ]),
                _ => panic!(),
            },
            ".bin" | ".incbin" => {
                if self.args.len() != 1 {
                    self.arg_count_err()
                } else if let Some(path) = Self::is_str_arg(&self.args[0]) {
                    match fs::read(path) {
                        Ok(bytes) => Ok(bytes),
                        Err(e) => self
                            .line_slice()
                            .err(&format!("error loading '{}': {}", path, e)),
                    }
                } else {
                    self.line_slice().err("expected filename")
                }
            }
            ".inc" | ".lib" | ".fil" => Ok(vec![]),
            "=" | ".equ" => Ok(vec![]),
            ".org" => self.pass1(assembler, None).map(|_| vec![]),
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
            _ => self.pass1(assembler, None).map(|_| vec![]),
        }
    }

    fn line_slice(&self) -> Rc<LineSlice> {
        if let Some(last_arg) = self.args.last() {
            Rc::new(self.op_name.join(&last_arg.slice))
        } else {
            self.op_name.clone()
        }
    }

    fn is_equ(&self) -> bool {
        matches!(self.op_name_lcase.as_str(), "=" | ".equ")
    }
}
