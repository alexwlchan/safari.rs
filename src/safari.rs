use std::process::Command;


pub fn safari_furl() -> String {
    run_applescript("tell application \"Safari\" to get URL of document 1")
}


pub fn safari_2url() -> String {
    run_applescript("tell application \"Safari\" to get URL of document 2")
}


/// Runs an AppleScript and returns the stdout.
fn run_applescript(script: &str) -> String {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .expect("failed to execute AppleScript");

    // Strip the trailing newline and return a String
    let mut output = output.stdout;
    output.pop();
    String::from_utf8(output).unwrap()
}
