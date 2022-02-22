#[test]
fn ejemplo_j1_loc() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "test_data/ejemploJ1_base.csv", "-l", "PENINSULA"])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 41.4, nren = 195.4, tot = 236.8")
        .stdout()
        .contains("RER = 0.17")
        .unwrap();
}

#[test]
fn ejemplo_j1() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ1_base.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 50.0, nren = 200.0, tot = 250.0")
        .stdout()
        .contains("RER = 0.20")
        .unwrap();
}

#[test]
fn ejemplo_j2() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ2_basePV.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 75.0, nren = 100.0, tot = 175.0")
        .stdout()
        .contains("RER = 0.43")
        .unwrap();
}

#[test]
fn ejemplo_j3() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ3_basePVexcess.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 100.0, nren = 0.0, tot = 100.0")
        .stdout()
        .contains("RER = 1.00")
        .unwrap();
}

#[test]
fn ejemplo_j5() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ5_gasPV.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 20.0, nren = 209.0, tot = 229.0")
        .stdout()
        .contains("RER = 0.09")
        .unwrap();
}

#[test]
fn ejemplo_j6() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ6_HPPV.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 180.5, nren = 38.0, tot = 218.5")
        .stdout()
        .contains("RER = 0.83")
        .unwrap();
}

#[test]
fn ejemplo_j7() {
    // Step A
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ7_cogenfuelgasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 0.0, nren = 214.5, tot = 214.5")
        .stdout()
        .contains("RER = 0.0")
        .unwrap();
    // Step B, kexp=1
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ7_cogenfuelgasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
            "--kexp",
            "1.0",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = -14.0, nren = 227.8, tot = 213.8")
        .stdout()
        .contains("RER = -0.07")
        .stdout()
        .contains("RER_nrb = 0.00")
        .unwrap();
}

#[test]
fn ejemplo_j8() {
    // Step A
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ8_cogenbiogasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 95.0, nren = 119.5, tot = 214.5")
        .stdout()
        .contains("RER = 0.44")
        .stdout()
        .contains("RER_nrb = 0.44")
        .unwrap();
    // Step B
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ8_cogenbiogasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
            "--kexp",
            "1.0",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 144.0, nren = 69.8, tot = 213.8")
        .stdout()
        .contains("RER = 0.67")
        .stdout()
        .contains("RER_nrb = 0.74")
        .unwrap();
}

#[test]
fn ejemplo_j9() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ9_electr.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 1009.5, nren = 842.0, tot = 1851.5")
        .stdout()
        .contains("RER = 0.55")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/cte_test_carriers.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 25.4, nren = 19.4, tot = 44.8")
        .stdout()
        .contains("RER = 0.57")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers_loc() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "test_data/cte_test_carriers.csv", "-l", "PENINSULA"])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 24.6, nren = 18.9, tot = 43.5")
        .stdout()
        .contains("RER = 0.57")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_nepb.csv",
            "-l",
            "PENINSULA",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 77.3 [%]")
        .stdout()
        .contains("* generada y usada en servicios EPB, por origen:\n- EAMBIENTE: 26.00\n- EL_INSITU: 4.45")
        .unwrap();

    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_nepb.csv",
            "-l",
            "PENINSULA",
            "--load_matching"
        ])
        .stdout()
        .contains("* generada y usada en servicios EPB, por origen:\n- EAMBIENTE: 13.00\n- EL_INSITU: 2.92")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb_con_exclusion_aux() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_exclusion_auxiliares.csv",
            "-l",
            "PENINSULA"
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 96.7 [%]")
        .unwrap();
}
