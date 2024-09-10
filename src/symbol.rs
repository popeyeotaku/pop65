//! Support for asm symbols.

use std::{collections::HashSet, fmt::Display, rc::Rc};

use crate::source::LineSlice;

/// An entry in the symbol table.
#[derive(Eq)]
pub struct Symbol {
    name: String,
    pub value: Option<u16>,
    pub defined_at: Option<Rc<LineSlice>>,
    references: HashSet<Rc<LineSlice>>,
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        if let Some(me) = self.value {
            if let Some(them) = other.value {
                return me == them;
            }
        }
        self.name == other.name
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let Some(me) = self.value {
            if let Some(them) = other.value {
                return me.cmp(&them);
            }
        }
        self.name.cmp(&other.name)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.value {
            f.write_fmt(format_args!("{v:04X}: "))?;
        } else {
            f.write_str("      ")?;
        }
        f.write_str(&self.name)?;
        Ok(())
    }
}

impl Symbol {
    pub fn new(name: &str, first_ref: Rc<LineSlice>) -> Box<Self> {
        let mut refs = HashSet::with_capacity(1);
        refs.insert(first_ref);
        Box::new(Self {
            name: name.to_string(),
            value: None,
            defined_at: None,
            references: refs,
        })
    }

    /// Add a new reference to this symbol.
    ///
    /// Returns `true` if the reference was already in the list.
    pub fn add_ref(&mut self, ref_slice: Rc<LineSlice>) -> bool {
        self.references.insert(ref_slice)
    }

    /// Try to define the value of this symbol; error if we're redefined.
    pub fn define(&mut self, value: u16, defined_at: Rc<LineSlice>) -> Result<(), String> {
        if self.value.is_none() {
            debug_assert!(self.defined_at.is_none());
            self.value = Some(value);
            self.defined_at = Some(defined_at.clone());
            self.add_ref(defined_at);
            Ok(())
        } else {
            debug_assert!(self.defined_at.is_some());
            defined_at.err(&format!(
                "'{}' redefined (orig. def. at {})",
                &self.name,
                self.defined_at.as_ref().unwrap().pos()
            ))
        }
    }
}
