use assert_cmd::Command;

fn gradient() -> Command {
    let mut cmd = Command::cargo_bin("gradient").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
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
        .arg("--custom")
        .arg("hwb(75, 25%, 10%)")
        .arg("#bad455")
        .arg("goldenrod")
        .assert()
        .success();

    /*
    gradient()
        .args(&[
            "--custom",
            "gold;purple;red",
            "--position",
            "0",
            "70",
            "100",
            "--blend-mode",
            "lab",
            "--interpolation",
            "basis",
        ])
        .assert()
        .success();

    gradient()
        .arg("-c")
        .arg("#46f; #ab7; #abc456")
        .arg("-P")
        .arg("0, 73,100 ")
        .arg("-s")
        .arg(" 0,73.0, 100 , 120")
        .assert()
        .success()
        .stdout("#4466ff\n#aabb77\n#abc456\n#abc456\n");
    */

    gradient()
        .arg("--file")
        .arg("data/gradients.svg")
        .arg("data/Neon_Green.ggr")
        .assert()
        .success();
}

#[test]
fn others() {
    // contains invalid gradient
    gradient()
        .arg("-f")
        .arg("data/test1.svg")
        .assert()
        .failure();

    // #grad-1 is a valid gradient
    gradient()
        .arg("-f")
        .arg("data/test1.svg")
        .arg("--svg-id")
        .arg("grad-1")
        .assert()
        .success();

    // #grad-0 is an invalid gradient
    gradient()
        .arg("-f")
        .arg("data/test1.svg")
        .arg("--svg-id")
        .arg("grad-0")
        .assert()
        .failure();
}

#[test]
fn invalid() {
    // invalid preset name
    gradient().arg("--preset").arg("sunset").assert().failure();

    // invalid CSS gradient
    gradient()
        .arg("--css")
        .arg("red, 25%, 70%, blue")
        .assert()
        .failure();

    // invalid position
    /*
    gradient()
        .arg("--custom")
        .arg("red;lime")
        .arg("--position")
        .args(&["0", "0.5", "1"])
        .assert()
        .failure();
    */

    // invalid SVG gradient
    gradient()
        .arg("--file")
        .arg("data/invalid.svg")
        .assert()
        .failure();

    // invalid GIMP gradient
    gradient()
        .arg("--file")
        .arg("data/invalid.ggr")
        .assert()
        .failure()
        .stderr("data/invalid.ggr (invalid GIMP gradient)\n");

    // SVG without gradient
    gradient()
        .arg("--file")
        .arg("data/no-gradient.svg")
        .assert()
        .failure();

    // non-existent file
    gradient()
        .arg("--file")
        .arg("gradients.svg")
        .assert()
        .failure()
        .stderr("gradients.svg: file not found.\n");

    // unsupported file format
    gradient()
        .arg("--file")
        .arg("Cargo.toml")
        .assert()
        .failure()
        .stderr("Cargo.toml: file format not supported.\n");

    gradient()
        .arg("--file")
        .arg("Makefile")
        .assert()
        .failure()
        .stderr("Makefile: file format not supported.\n");
}
