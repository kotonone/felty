/// 指定された URL を開きます。
pub fn open(url: &str) {
    // TODO: macOS
    use std::{ffi::OsString, os::windows::process::CommandExt, process::Command};
    let mut cmd = Command::new("cmd");
    let _ = cmd.arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .raw_arg({
            let mut p = OsString::from("\"");
            p.push(url);
            p.push("\"");
            p
        })
        .spawn();
}
