// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

pub mod annotation;
pub use annotation::*;

pub mod finalize;
pub use finalize::*;

pub mod function_input;
pub use function_input::*;

pub mod function_output;
pub use function_output::*;

pub mod mode;
pub use mode::*;

use crate::{Block, Identifier, Node, Tuple, Type};
use leo_span::{sym, Span, Symbol};

use serde::{Deserialize, Serialize};
use std::fmt;

/// A function definition.
#[derive(Clone, Serialize, Deserialize)]
pub struct Function {
    /// Annotations on the function.
    pub annotations: Vec<Annotation>,
    /// The function identifier, e.g., `foo` in `function foo(...) { ... }`.
    pub identifier: Identifier,
    /// The function's input parameters.
    pub input: Vec<FunctionInput>,
    /// The function's output declarations.
    pub output: Vec<FunctionOutput>,
    /// The function's output type.
    pub output_type: Type,
    /// The body of the function.
    pub block: Block,
    /// An optional finalize block
    pub finalize: Option<Finalize>,
    /// The entire span of the function definition.
    pub span: Span,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for Function {}

impl Function {
    /// Initialize a new function.
    pub fn new(
        annotations: Vec<Annotation>,
        identifier: Identifier,
        input: Vec<FunctionInput>,
        output: Vec<FunctionOutput>,
        block: Block,
        finalize: Option<Finalize>,
        span: Span,
    ) -> Self {
        // Determine the output type of the function
        let output_type = match output.len() {
            0 => Type::Unit,
            1 => output[0].type_.clone(),
            _ => Type::Tuple(Tuple(output.iter().map(|output| output.type_.clone()).collect())),
        };

        Function {
            annotations,
            identifier,
            input,
            output,
            output_type,
            block,
            finalize,
            span,
        }
    }
    /// Returns function name.
    pub fn name(&self) -> Symbol {
        self.identifier.name
    }

    /// Returns `true` if the function name is `main`.
    pub fn is_main(&self) -> bool {
        self.name() == sym::main
    }

    ///
    /// Private formatting method used for optimizing [fmt::Debug] and [fmt::Display] implementations.
    ///
    fn format(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "function {}", self.identifier)?;

        let parameters = self.input.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
        let returns = match self.output.len() {
            0 => "()".to_string(),
            1 => self.output[0].to_string(),
            _ => self.output.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
        };
        write!(f, "({}) -> {} {}", parameters, returns, self.block)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.format(f)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.format(f)
    }
}

crate::simple_node_impl!(Function);