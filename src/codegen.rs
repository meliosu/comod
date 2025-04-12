use std::fmt::Write;

use crate::types::*;

pub type Code = String;

pub struct Generator {
    model: Model,
    ucodes: Code,
}

impl Generator {
    pub fn new(model: Model, ucodes: Code) -> Self {
        Self { model, ucodes }
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
        write!(o, "int main(){{}}")?;
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

        write!(o, "}} variables;")?;
        Ok(())
    }

    fn gen_operations_struct(&self, o: &mut Code) -> anyhow::Result<()> {
        write!(o, "struct {{")?;

        for (name, _) in &self.model.operations {
            write!(o, "struct {{int status;}} {};", name)?;
        }

        write!(o, "}} operations;")?;
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
            write!(o, "variables.{output}.computed = 1;")?;
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
            write!(o, "enqueue(op_{name});")?;
            write!(o, "}}")?;
        }

        write!(o, "operations.{name}.status = STOPPED;")?;
        write!(o, "}}")?;
        Ok(())
    }
}
