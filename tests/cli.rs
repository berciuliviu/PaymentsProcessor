use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs

#[test]
fn test_csv_file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("toy_processor")?;
    cmd.arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(predicates::str::contains(
        "File test/file/doesnt/exist does not exist.",
    ));

    Ok(())
}

#[test]
fn test_argument_not_provided() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("toy_processor")?;
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Error! No argument provided."));

    Ok(())
}

#[test]
fn test_multiple_arguments_provided() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("toy_processor")?;
    cmd.arg("one").arg("two");
    cmd.assert().failure().stderr(predicates::str::contains(
        "There should be only one argument given to the program.",
    ));

    Ok(())
}

#[test]
fn test_basic_csv_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("toy_processor")?;

    cmd.arg(format!(
        "{}/tests/resources/basic_example.csv",
        env!("CARGO_MANIFEST_DIR")
    ));

    cmd.assert()
        .success()
        .stdout(predicates::str::contains(
            "client,available,held,total,locked",
        ))
        .stdout(predicates::str::contains("2,2.0000,0.0000,2.0000,false"))
        .stdout(predicates::str::contains("1,1.5000,0.0000,1.5000,false"));

    Ok(())
}

#[test]
fn test_csv_example_with_errors() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("toy_processor")?;

    cmd.arg(format!(
        "{}/tests/resources/example_with_errors.csv",
        env!("CARGO_MANIFEST_DIR")
    ));

    cmd.assert()
        .success()
        // .stderr(predicates::str::contains("PROCESSOR ERROR: Invalid withdrawal transaction 5. Available amount is smaller than withdraw amount."))
        // .stderr(predicates::str::contains("PROCESSOR ERROR: Invalid withdrawal transaction 6. Available amount is smaller than withdraw amount."))
        .stdout(predicates::str::contains(
            "client,available,held,total,locked",
        ))
        .stdout(predicates::str::contains("2,2.0000,0.0000,2.0000,false"))
        .stdout(predicates::str::contains("1,1.5000,0.0000,1.5000,false"));

    Ok(())
}

#[test]
fn test_stress_csv_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("toy_processor")?;

    cmd.arg(format!(
        "{}/tests/resources/deposit_stress.csv",
        env!("CARGO_MANIFEST_DIR")
    ));

    cmd.assert()
        .success()
        .stdout(predicates::str::contains(
            "client,available,held,total,locked",
        ))
        .stdout(predicates::str::contains(
            "1,10000005.0000,0.0000,10000005.0000,false",
        ));

    Ok(())
}
