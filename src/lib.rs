use std::io::{Read, Write};

/// A piece of compiled code that's ready to be evaluated.
pub struct Program {}

/// Represents failures during execution.
///
/// Use it to get access to the cause, backtraces, etc.
pub struct EvalError {}

impl Program {
    /// Evaluate a program with given I/O buffers.
    pub fn eval<R: Read, W: Write>(
        &self,
        _input: &mut R,
        _output: &mut W,
    ) -> Result<(), EvalError> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {}
