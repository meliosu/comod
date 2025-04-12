use crate::codegen::Code;

pub fn compile(code: &Code, name: &str) -> anyhow::Result<()> {
    let src_path = format!("/tmp/{name}_temp.c");
    std::fs::write(&src_path, code)?;

    let mut child = std::process::Command::new("gcc")
        .arg("-O1")
        .arg("-o")
        .arg(name)
        .arg(src_path)
        .spawn()?;

    child.wait()?;
    Ok(())
}
