use assert_cli;

#[test]
fn ejemplo_j1_loc() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "test_data/ejemploJ1_base.csv", "-l", "PENINSULA"])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 41.4, nren = 195.4, tot = 236.8, RER = 0.17")
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
        .contains("C_ep [kWh/m2.an]: ren = 50.0, nren = 200.0, tot = 250.0, RER = 0.20")
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
        .contains("C_ep [kWh/m2.an]: ren = 75.0, nren = 100.0, tot = 175.0, RER = 0.43")
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
        .contains("C_ep [kWh/m2.an]: ren = 100.0, nren = 0.0, tot = 100.0, RER = 1.00")
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
        .contains("C_ep [kWh/m2.an]: ren = 20.0, nren = 209.0, tot = 229.0, RER = 0.09")
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
        .contains("C_ep [kWh/m2.an]: ren = 180.5, nren = 38.0, tot = 218.5, RER = 0.83")
        .unwrap();
}

#[test]
fn ejemplo_j7() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ7_cogenfuelgasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = -27.4, nren = 283.8, tot = 256.4, RER = -0.11")
        .unwrap();
}

#[test]
fn ejemplo_j8() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ8_cogenbiogasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 146.4, nren = 125.8, tot = 272.2, RER = 0.54")
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
        .contains("C_ep [kWh/m2.an]: ren = 1009.5, nren = 842.0, tot = 1851.5, RER = 0.55")
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
        .contains("C_ep [kWh/m2.an]: ren = 25.4, nren = 19.4, tot = 44.8, RER = 0.57")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers_loc() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "test_data/cte_test_carriers.csv", "-l", "PENINSULA"])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 24.6, nren = 18.9, tot = 43.5, RER = 0.57")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers_loc_nearby() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/cte_test_carriers.csv",
            "-l",
            "PENINSULA",
            "--acs_nearby",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 9.2, nren = 4.7, tot = 13.9, RER = 0.66")
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
            "--demanda_anual_acs",
            "1823.8",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 77.3 [%]")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb_con_exclusion_aux() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_exclusion_auxiliares.csv",
            "-l",
            "PENINSULA",
            "--demanda_anual_acs",
            "4549.0",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 100.0 [%]")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb_con_exclusion_aux_meta() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_exclusion_auxiliares.csv",
            "-l",
            "PENINSULA",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 100.0 [%]")
        .unwrap();
}