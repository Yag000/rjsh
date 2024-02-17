use colored::Colorize;

use crate::shell::Shell;

pub fn get_prompt(shell: &dyn Shell) -> anyhow::Result<String> {
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

    let invite = match shell.last_exit_code() {
        0 => "$ ".green(),
        _ => "$ ".red(),
    };

    let prompt = format!("{} {}", new_cwd.blue(), invite);

    let jobs = shell.job_number();
    if jobs > 0 {
        let jobs = format!("[{jobs}]").cyan();
        Ok(format!("{jobs} {prompt}"))
    } else {
        Ok(prompt)
    }
}
