//! Support for asm symbols.

use std::collections::HashSet;

use crate::source::LineSlice;

/// An entry in the symbol table.
pub struct Symbol {
    name: String,
    pub value: Option<u16>,
    pub defined_at: Option<LineSlice>,
    references: HashSet<LineSlice>,
}

impl Symbol {
    pub fn new(name: &str, first_ref: &LineSlice) -> Box<Self> {
        let mut refs = HashSet::with_capacity(1);
        refs.insert(first_ref.clone());
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
    pub fn add_ref(&mut self, ref_slice: &LineSlice) -> bool {
        self.references.insert(ref_slice.clone())
    }

    /// Try to define the value of this symbol; error if we're redefined.
    pub fn define(&mut self, value: u16, defined_at: &LineSlice) -> Result<(), String> {
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
