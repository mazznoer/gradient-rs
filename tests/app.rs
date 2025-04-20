use assert_cmd::Command;

fn gradient() -> Command {
    Command::cargo_bin("gradient").unwrap()
}

#[test]
fn basic() {
    gradient().assert().failure();

    gradient().arg("--list-presets").assert().success();

    gradient().arg("--named-colors").assert().success();

    gradient().arg("--preset").arg("magma").assert().success();

    gradient()
        .arg("--preset")
        .arg("rainbow")
        .arg("--sample")
        .args(&["0", "0.35", "0.77"])
        .assert()
        .success()
        .stdout("#6e40aa\n#fb9633\n#1cbccc\n");

    gradient()
        .arg("--css")
        .arg("#f05, rgb(0, 255, 90)")
        .arg("--take")
        .arg("5")
        .arg("--array")
        .assert()
        .success()
        .stdout(concat!(
            r##"["#ff0055", "#ed7458", "#d0a95a", "#a0d55b", "#00ff5a"]"##,
            "\n"
        ));

    gradient()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("--file")
        .arg("data/gradients.svg")
        .arg("data/Neon_Green.ggr")
        .assert()
        .success();
}
