use std::process::Command;


pub struct Output {
  pub status: i32,
  pub stdout: String,
  pub stderr: String,
}


/// Run an AppleScript.
///
/// * `script`: The AppleScript code to run.
///
pub fn run(script: &str) -> Output {
  let cmd_result = Command::new("osascript")
    .arg("-e")
    .arg(script)
    .output()
    .expect("failed to execute AppleScript");

  Output {
    status: cmd_result.status.code().unwrap_or(1),
    stdout: String::from_utf8(cmd_result.stdout).unwrap(),
    stderr: String::from_utf8(cmd_result.stderr).unwrap(),
  }
}
