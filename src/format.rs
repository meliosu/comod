use anyhow::anyhow;

use std::{
    io::{Read, Write},
    process::Stdio,
};

use crate::codegen::Code;

fn format(code: &Code) -> anyhow::Result<Code> {
    let child = std::process::Command::new("clang-format")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.ok_or(anyhow!("error getting child stdin"))?;
    let mut stdout = child.stdout.ok_or(anyhow!("error getting child stdout"))?;

    stdin.write(code.as_bytes())?;

    let mut buffer = Vec::new();
    stdout.read_to_end(&mut buffer)?;

    let code = String::from_utf8(buffer)?;
    Ok(code)
}
