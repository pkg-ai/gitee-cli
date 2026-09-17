mod common;

#[test]
fn version_flag_prints_binary_name_and_package_version() {
    let output = common::cmd().args(["--version"]).output().unwrap();

    common::assert_ok(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        format!("gitee {}", env!("CARGO_PKG_VERSION"))
    );
}
