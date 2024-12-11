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
            ".if" => {
                if self.args.len() == 1 {
                    let cond_val = self.args[0].eval(assembler)?;
                    assembler.if_stack.push(cond_val != 0);
                    Ok(0)
                } else {
                    self.arg_count_err()
                }
            }
            ".endif" => {
                if assembler.if_stack.pop().is_none() {
                    self.line_slice().err("missing matching if")
                } else if !self.args.is_empty() {
                    self.line_slice().err("no args on endif")
                } else {
                    Ok(0)
                }
            }
            ".else" => {
                if let Some(cond_val) = assembler.if_stack.last_mut() {
                    *cond_val = !*cond_val;
                    if self.args.is_empty() {
                        Ok(0)
                    } else {
                        self.line_slice().err("no args on else")
                    }
                } else {
                    self.line_slice().err("missing matching if")
                }
            }
            ".assert" => Ok(0),
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
                    assembler.pc = val;
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
            ".off" => Ok(0),
            ".on" => Ok(0),
            _ => self
                .line_slice()
                .err(&format!("bad pseudo-op '{}'", self.op_name.text())),
        }
    }

    fn pass2(&self, assembler: &mut Assembler) -> Result<Vec<u8>, String> {
        match self.op_name_lcase.as_str() {
            ".if" | ".else" | ".endif" => {
                // statements skipped by these should already have been deleted, so do nothing.
                Ok(vec![])
            }
            ".assert" => {
                if self.args.len() == 1 {
                    let val = self.args[0].eval(assembler)?;
                    if val == 0 {
                        self.line_slice().err("assertion error")
                    } else {
                        Ok(vec![])
                    }
                } else {
                    self.arg_count_err()
                }
            }
            ".on" => {
                assembler.output_flag = true;
                Ok(vec![])
            }
            ".off" => {
                assembler.output_flag = false;
                Ok(vec![])
            }
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

    fn is_if_affiliated(&self) -> bool {
        matches!(self.op_name_lcase.as_str(), ".else" | ".endif")
    }
}

#[cfg(test)]
mod tests {
    use crate::{assemble, assemble_str, source};

    #[test]
    fn test_onoff() {
        let src = "
        .org $100
        .off
foo     .ds 2
bar     .ds 1
foobar";
        let info = assemble(source::from_str(src, src), false).unwrap();
        assert_eq!(info.symtab["foo"].value, Some(0x100));
        assert_eq!(info.symtab["bar"].value, Some(0x102));
        assert_eq!(info.symtab["foobar"].value, Some(0x103));
        assert!(info.bytes.is_empty());
    }

    #[test]
    fn test_assert() {
        assert!(assemble_str(
            ".ASSERT 1 > 2",
            "this test will print an assembler assertion error, no worries"
        )
        .is_err());
        assert!(assemble_str(".ASSERT 2 > 1", "").is_ok());
    }

    #[test]
    fn test_if() {
        let src = "
FOO = 1
BAR = 2
.IF FOO = BAR
    .BYTE 1,2,3
.ELSE
    .IF FOO < BAR
        .BYTE 4,5,6
    .ENDIF
    .BYTE 7,8,9
.ENDIF";
        assert_eq!(assemble_str(src, "src"), Ok(vec![4, 5, 6, 7, 8, 9]));
    }
}
