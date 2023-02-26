// Copyright (C) 2019-2023 Aleo Systems Inc.
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

use crate::CodeGenerator;

use leo_ast::{
    AccessExpression, AssociatedFunction, BinaryExpression, BinaryOperation, CallExpression, ErrExpression, Expression,
    ExpressionConsumer, Identifier, Literal, MemberAccess, StructExpression, TernaryExpression, TupleAccess,
    TupleExpression, Type, UnaryExpression, UnaryOperation, UnitExpression,
};
use leo_span::sym;
use std::borrow::Borrow;

use std::fmt::Write as _;

impl ExpressionConsumer for CodeGenerator<'_> {
    type Output = (String, String);

    fn consume_identifier(&mut self, input: Identifier) -> Self::Output {
        (self.variable_mapping.get(&input.name).unwrap().clone(), String::new())
    }

    fn consume_err(&mut self, _input: ErrExpression) -> Self::Output {
        unreachable!("`ErrExpression`s should not be in the AST at this phase of compilation.")
    }

    fn consume_literal(&mut self, input: Literal) -> Self::Output {
        (format!("{input}"), String::new())
    }

    fn consume_binary(&mut self, input: BinaryExpression) -> Self::Output {
        let (left_operand, left_instructions) = self.consume_expression(*input.left);
        let (right_operand, right_instructions) = self.consume_expression(*input.right);

        let opcode = match input.op {
            BinaryOperation::Add => String::from("add"),
            BinaryOperation::AddWrapped => String::from("add.w"),
            BinaryOperation::And => String::from("and"),
            BinaryOperation::BitwiseAnd => String::from("and"),
            BinaryOperation::Div => String::from("div"),
            BinaryOperation::DivWrapped => String::from("div.w"),
            BinaryOperation::Eq => String::from("is.eq"),
            BinaryOperation::Gte => String::from("gte"),
            BinaryOperation::Gt => String::from("gt"),
            BinaryOperation::Lte => String::from("lte"),
            BinaryOperation::Lt => String::from("lt"),
            BinaryOperation::Mod => String::from("mod"),
            BinaryOperation::Mul => String::from("mul"),
            BinaryOperation::MulWrapped => String::from("mul.w"),
            BinaryOperation::Nand => String::from("nand"),
            BinaryOperation::Neq => String::from("is.neq"),
            BinaryOperation::Nor => String::from("nor"),
            BinaryOperation::Or => String::from("or"),
            BinaryOperation::BitwiseOr => String::from("or"),
            BinaryOperation::Pow => String::from("pow"),
            BinaryOperation::PowWrapped => String::from("pow.w"),
            BinaryOperation::Rem => String::from("rem"),
            BinaryOperation::RemWrapped => String::from("rem.w"),
            BinaryOperation::Shl => String::from("shl"),
            BinaryOperation::ShlWrapped => String::from("shl.w"),
            BinaryOperation::Shr => String::from("shr"),
            BinaryOperation::ShrWrapped => String::from("shr.w"),
            BinaryOperation::Sub => String::from("sub"),
            BinaryOperation::SubWrapped => String::from("sub.w"),
            BinaryOperation::Xor => String::from("xor"),
        };

        let destination_register = format!("r{}", self.next_register);
        let binary_instruction = format!("    {opcode} {left_operand} {right_operand} into {destination_register};\n",);

        // Increment the register counter.
        self.next_register += 1;

        // Concatenate the instructions.
        let mut instructions = left_instructions;
        instructions.push_str(&right_instructions);
        instructions.push_str(&binary_instruction);

        (destination_register, instructions)
    }

    fn consume_unary(&mut self, input: UnaryExpression) -> Self::Output {
        let (expression_operand, expression_instructions) = self.consume_expression(*input.receiver);

        let opcode = match input.op {
            UnaryOperation::Abs => String::from("abs"),
            UnaryOperation::AbsWrapped => String::from("abs.w"),
            UnaryOperation::Double => String::from("double"),
            UnaryOperation::Inverse => String::from("inv"),
            UnaryOperation::Not => String::from("not"),
            UnaryOperation::Negate => String::from("neg"),
            UnaryOperation::Square => String::from("square"),
            UnaryOperation::SquareRoot => String::from("sqrt"),
        };

        let destination_register = format!("r{}", self.next_register);
        let unary_instruction = format!("    {opcode} {expression_operand} into {destination_register};\n");

        // Increment the register counter.
        self.next_register += 1;

        // Concatenate the instructions.
        let mut instructions = expression_instructions;
        instructions.push_str(&unary_instruction);

        (destination_register, instructions)
    }

    fn consume_ternary(&mut self, input: TernaryExpression) -> Self::Output {
        let (condition_operand, condition_instructions) = self.consume_expression(*input.condition);
        let (if_true_operand, if_true_instructions) = self.consume_expression(*input.if_true);
        let (if_false_operand, if_false_instructions) = self.consume_expression(*input.if_false);

        let destination_register = format!("r{}", self.next_register);
        let ternary_instruction = format!(
            "    ternary {condition_operand} {if_true_operand} {if_false_operand} into {destination_register};\n",
        );

        // Increment the register counter.
        self.next_register += 1;

        // Concatenate the instructions.
        let mut instructions = condition_instructions;
        instructions.push_str(&if_true_instructions);
        instructions.push_str(&if_false_instructions);
        instructions.push_str(&ternary_instruction);

        (destination_register, instructions)
    }

    fn consume_struct_init(&mut self, input: StructExpression) -> Self::Output {
        // Lookup struct or record.
        let name = if let Some((is_record, type_)) = self.composite_mapping.get(&input.name.name) {
            if *is_record {
                // record.private;
                format!("{}.{type_}", input.name)
            } else {
                // foo; // no visibility for structs
                input.name.to_string()
            }
        } else {
            unreachable!("All composite types should be known at this phase of compilation")
        };

        // Initialize instruction builder strings.
        let mut instructions = String::new();
        let mut struct_init_instruction = String::from("    cast ");

        // Visit each struct member and accumulate instructions from expressions.
        for member in input.members.into_iter() {
            let operand = if let Some(expr) = member.expression {
                // Visit variable expression.
                let (variable_operand, variable_instructions) = self.consume_expression(expr);
                instructions.push_str(&variable_instructions);

                variable_operand
            } else {
                // Push operand identifier.
                let (ident_operand, ident_instructions) = self.consume_identifier(member.identifier);
                instructions.push_str(&ident_instructions);

                ident_operand
            };

            // Push operand name to struct init instruction.
            write!(struct_init_instruction, "{operand} ").expect("failed to write to string");
        }

        // Push destination register to struct init instruction.
        let destination_register = format!("r{}", self.next_register);
        writeln!(struct_init_instruction, "into {destination_register} as {name};",)
            .expect("failed to write to string");

        instructions.push_str(&struct_init_instruction);

        // Increment the register counter.
        self.next_register += 1;

        (destination_register, instructions)
    }

    fn consume_member_access(&mut self, input: MemberAccess) -> Self::Output {
        let (inner_struct, _inner_instructions) = self.consume_expression(*input.inner);
        let member_access_instruction = format!("{inner_struct}.{}", input.name);

        (member_access_instruction, String::new())
    }

    // Pedersen64::hash() -> hash.ped64
    fn consume_associated_function(&mut self, input: AssociatedFunction) -> Self::Output {
        // Write identifier as opcode. `Pedersen64` -> `ped64`.
        let symbol: &str = if let Type::Identifier(identifier) = input.ty {
            match identifier.name {
                sym::BHP256 => "bhp256",
                sym::BHP512 => "bhp512",
                sym::BHP768 => "bhp768",
                sym::BHP1024 => "bhp1024",
                sym::Pedersen64 => "ped64",
                sym::Pedersen128 => "ped128",
                sym::Poseidon2 => "psd2",
                sym::Poseidon4 => "psd4",
                sym::Poseidon8 => "psd8",
                _ => unreachable!("All core function calls should be known at this time."),
            }
        } else {
            unreachable!("All core function should be known at this time.")
        };

        // Construct associated function call.
        let mut associated_function_call = format!("    {}.{symbol} ", input.name);
        let mut instructions = String::new();

        // Visit each function argument and accumulate instructions from expressions.
        for arg in input.args.into_iter() {
            let (arg_string, arg_instructions) = self.consume_expression(arg);
            write!(associated_function_call, "{arg_string} ").expect("failed to write associated function argument");
            instructions.push_str(&arg_instructions);
        }

        // Push destination register to associated function call instruction.
        let destination_register = format!("r{}", self.next_register);
        writeln!(associated_function_call, "into {destination_register};")
            .expect("failed to write dest register for associated function");
        instructions.push_str(&associated_function_call);

        // Increment the register counter.
        self.next_register += 1;

        (destination_register, instructions)
    }

    fn consume_access(&mut self, input: AccessExpression) -> Self::Output {
        match input {
            AccessExpression::Member(access) => self.consume_member_access(access),
            AccessExpression::AssociatedFunction(function) => self.consume_associated_function(function),
            AccessExpression::Tuple(_) => {
                unreachable!("Tuples should have been flattened in previous compiler passes.")
            }
        }
    }

    // TODO: Cleanup
    fn consume_call(&mut self, input: CallExpression) -> Self::Output {
        let mut call_instruction = match &input.external {
            Some(external) => format!("    call {external}.aleo/{}", input.function),
            None => format!("    call {}", input.function),
        };
        let mut instructions = String::new();

        for argument in input.arguments.into_iter() {
            let (argument, argument_instructions) = self.consume_expression(argument);
            write!(call_instruction, " {argument}").expect("failed to write to string");
            instructions.push_str(&argument_instructions);
        }

        // Lookup the function return type.
        let function_name = match input.function.borrow() {
            Expression::Identifier(identifier) => identifier.name,
            _ => unreachable!("Parsing guarantees that all `input.function` is always an identifier."),
        };
        let return_type = &self
            .symbol_table
            .borrow()
            .functions
            .get(&function_name)
            .unwrap()
            .output_type;
        match return_type {
            Type::Unit => {
                call_instruction.push(';');
                instructions.push_str(&call_instruction);
                (String::new(), instructions)
            } // Do nothing
            Type::Tuple(tuple) => match tuple.len() {
                0 | 1 => unreachable!("Parsing guarantees that a tuple type has at least two elements"),
                len => {
                    let mut destinations = Vec::new();
                    for _ in 0..len {
                        let destination_register = format!("r{}", self.next_register);
                        destinations.push(destination_register);
                        self.next_register += 1;
                    }
                    let destinations = destinations.join(" ");
                    writeln!(call_instruction, " into {destinations};").expect("failed to write to string");
                    instructions.push_str(&call_instruction);

                    (destinations, instructions)
                }
            },
            _ => {
                // Push destination register to call instruction.
                let destination_register = format!("r{}", self.next_register);
                writeln!(call_instruction, " into {destination_register};").expect("failed to write to string");
                instructions.push_str(&call_instruction);

                // Increment the register counter.
                self.next_register += 1;

                (destination_register, instructions)
            }
        }
    }

    fn consume_tuple(&mut self, input: TupleExpression) -> Self::Output {
        // Need to return a single string here so we will join the tuple elements with ' '
        // and split them after this method is called.
        let mut tuple_elements = Vec::with_capacity(input.elements.len());
        let mut instructions = String::new();

        // Visit each tuple element and accumulate instructions from expressions.
        for element in input.elements.into_iter() {
            let (element, element_instructions) = self.consume_expression(element);
            tuple_elements.push(element);
            instructions.push_str(&element_instructions);
        }

        // CAUTION: does not return the destination_register.
        (tuple_elements.join(" "), instructions)
    }

    fn consume_unit(&mut self, _input: UnitExpression) -> Self::Output {
        unreachable!("`UnitExpression`s should not exist during code generation.")
    }

    fn consume_tuple_access(&mut self, _input: TupleAccess) -> Self::Output {
        todo!()
    }
}