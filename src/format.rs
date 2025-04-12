use anyhow::anyhow;

use std::{io::Write, process::Stdio};

use crate::codegen::Code;

pub fn format(code: &Code) -> anyhow::Result<Code> {
    let mut child = std::process::Command::new("clang-format")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or(anyhow!("error getting child stdin"))?;

    stdin.write(code.as_bytes())?;
    drop(stdin);

    let output = child.wait_with_output()?;
    let code = String::from_utf8(output.stdout)?;
    Ok(code)
}
