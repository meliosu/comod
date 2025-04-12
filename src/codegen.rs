use std::{collections::HashMap, fmt::Write};

use anyhow::bail;

use crate::types::*;

pub type Code = String;

const NUM_WORKERS: usize = 6;

pub struct Generator {
    model: Model,
    ucodes: Code,
    inputs: Vec<(String, String)>,
    outputs: Vec<String>,
}

impl Generator {
    pub fn new(
        model: Model,
        ucodes: Code,
        inputs: Vec<(String, String)>,
        outputs: Vec<String>,
    ) -> Self {
        Self {
            model,
            ucodes,
            inputs,
            outputs,
        }
    }

    pub fn generate(self) -> anyhow::Result<Code> {
        let mut o = Code::new();

        self.gen_runtime(&mut o)?;
        self.gen_ucodes(&mut o)?;
        self.gen_variables_struct(&mut o)?;
        self.gen_operations_struct(&mut o)?;
        self.gen_operation_signatures(&mut o)?;

        for (name, operation) in &self.model.operations {
            self.gen_operation(&mut o, name, operation)?;
        }

        self.gen_main(&mut o)?;

        Ok(o)
    }

    fn gen_main(&self, o: &mut Code) -> anyhow::Result<()> {
        write!(o, "int main(){{")?;

        write!(o, "tp_init({NUM_WORKERS});")?;

        for (key, value) in &self.inputs {
            write!(o, "variables.{key}.value = {value};")?;
            write!(o, "variables.{key}.computed = true;")?;
        }

        for (name, _) in self.model.operations.iter().filter(|(_, op)| {
            op.inputs
                .iter()
                .all(|i| self.inputs.iter().any(|(key, _)| key == i))
        }) {
            write!(o, "operations.{name}.status = RUNNING;")?;
            write!(o, "tp_enqueue(op_{name});")?;
        }

        write!(o, "tp_wait();")?;

        for output in &self.outputs {
            let Some(ty) = self
                .model
                .variables
                .iter()
                .find_map(|(name, var)| (*name == *output).then_some(&var.ty))
            else {
                todo!();
            };

            let fmt = match ty {
                Type::I8 => "d",
                Type::U8 => "d",
                Type::I16 => "d",
                Type::U16 => "d",
                Type::I32 => "d",
                Type::U32 => "d",
                Type::I64 => "ld",
                Type::U64 => "ld",
                Type::F32 => "f",
                Type::F64 => "lf",
                Type::Ptr => "p",
            };

            write!(o, r#"printf("{output}: ");"#)?;
            write!(o, r#"if (variables.{output}.computed) {{"#)?;
            write!(o, r#"printf("%{fmt}\n", variables.{output}.value);}}"#)?;
            write!(o, r#"else {{"#)?;
            write!(o, r#"printf("not computed\n");}}"#)?;
        }

        write!(o, "}}")?;
        Ok(())
    }

    fn gen_ucodes(&self, o: &mut Code) -> anyhow::Result<()> {
        write!(o, "{}", self.ucodes)?;
        Ok(())
    }

    fn gen_runtime(&self, o: &mut Code) -> anyhow::Result<()> {
        write!(o, "{}", include_str!("../c/runtime.c"))?;
        Ok(())
    }

    fn gen_variables_struct(&self, o: &mut Code) -> anyhow::Result<()> {
        write!(o, "struct {{")?;

        for (name, variable) in &self.model.variables {
            write!(
                o,
                "struct {{bool computed; {} value;}} {};",
                variable.ty, name
            )?;
        }

        write!(o, "}} variables = {{0}};")?;
        Ok(())
    }

    fn gen_operations_struct(&self, o: &mut Code) -> anyhow::Result<()> {
        write!(o, "struct {{")?;

        for (name, _) in &self.model.operations {
            write!(o, "struct {{int status;}} {};", name)?;
        }

        write!(o, "}} operations = {{0}};")?;
        Ok(())
    }

    fn gen_operation_signatures(&self, o: &mut Code) -> anyhow::Result<()> {
        for (name, _) in &self.model.operations {
            write!(o, "void op_{name}();")?;
        }

        Ok(())
    }

    fn gen_operation(&self, o: &mut Code, name: &str, operation: &Operation) -> anyhow::Result<()> {
        write!(o, "void op_{name}() {{")?;
        write!(o, "struct {{")?;

        for input in &operation.inputs {
            let Some(var) = self.model.variables.get(input) else {
                todo!();
            };

            write!(o, "{} {};", var.ty, input)?;
        }

        for output in &operation.outputs {
            let Some(var) = self.model.variables.get(output) else {
                todo!();
            };

            write!(o, "{} {};", var.ty, output)?;
        }

        write!(o, "}} ctx;")?;

        for input in &operation.inputs {
            write!(o, "ctx.{input} = variables.{input}.value;")?;
        }

        write!(o, "{}(", operation.module.name)?;

        for (i, argument) in operation.module.args.iter().enumerate() {
            if operation.inputs.contains(&argument) {
                write!(o, "ctx.{argument}")?;
            } else if operation.outputs.contains(&argument) {
                write!(o, "&ctx.{argument}")?;
            } else {
                todo!();
            }

            if i < operation.module.args.len() - 1 {
                write!(o, ",")?;
            }
        }

        write!(o, ");")?;

        for output in &operation.outputs {
            write!(o, "variables.{output}.value = ctx.{output};")?;
            write!(o, "variables.{output}.computed = true;")?;
        }

        for (name, child_operation) in self
            .model
            .operations
            .iter()
            .filter(|(_, op)| op.inputs.iter().any(|i| operation.outputs.contains(i)))
        {
            write!(o, "if (")?;

            write!(o, "operations.{name}.status == WAITING &&")?;

            for (i, input) in child_operation.inputs.iter().enumerate() {
                write!(o, "variables.{input}.computed")?;

                if i < child_operation.inputs.len() - 1 {
                    write!(o, "&&")?;
                }
            }

            write!(o, ") {{")?;
            write!(o, "operations.{name}.status = RUNNING;")?;
            write!(o, "tp_enqueue(op_{name});")?;
            write!(o, "}}")?;
        }

        write!(o, "operations.{name}.status = STOPPED;")?;
        write!(o, "}}")?;
        Ok(())
    }
}
