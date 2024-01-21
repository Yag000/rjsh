pub fn get_prompt() -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd
        .to_str()
        .ok_or(anyhow::anyhow!("invalid current directory"))?
        .replace(std::env::var("HOME")?.as_str(), "~");

    let new_cwd = if cwd.len() > 45 {
        format!("...{}", &cwd[cwd.len() - 45..])
    } else {
        cwd
    };

    Ok(format!("{} $ ", new_cwd))
}
