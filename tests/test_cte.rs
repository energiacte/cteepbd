#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use pretty_assertions::assert_eq;

use cteepbd::{cte::*, types::*, *};

const TESTFPJ: &str = "vector, fuente, uso, step, ren, nren, co2
ELECTRICIDAD, RED, SUMINISTRO, A, 0.5, 2.0, 0.42
ELECTRICIDAD, INSITU, SUMINISTRO,   A, 1.0, 0.0, 0.0
ELECTRICIDAD, INSITU, A_RED, A, 1.0, 0.0, 0.0
ELECTRICIDAD, INSITU, A_RED, B, 0.5, 2.0, 0.42
ELECTRICIDAD, INSITU, A_NEPB, A, 1.0, 0.0, 0.0
ELECTRICIDAD, INSITU, A_NEPB, B, 0.5, 2.0, 0.42
GASNATURAL, RED, SUMINISTRO,A, 0.0, 1.1, 0.22
BIOCARBURANTE, RED, SUMINISTRO, A, 1.1, 0.1, 0.07
MEDIOAMBIENTE, INSITU, SUMINISTRO,  A, 1.0, 0.0, 0.0
MEDIOAMBIENTE, RED, SUMINISTRO,  A, 1.0, 0.0, 0.0
";

const TESTFPJ7: &str = "vector, fuente, uso, step, ren, nren, co2
ELECTRICIDAD, RED, SUMINISTRO, A, 0.5, 2.0, 0.42
GASNATURAL, RED, SUMINISTRO,A, 0.0, 1.1, 0.22
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.0, 0.0, 0.0
ELECTRICIDAD, COGENERACION, A_RED, A, 0.0, 2.5, 0.82
ELECTRICIDAD, COGENERACION, A_RED, B, 0.5, 2.0, 0.42
";

const TESTFPJ8: &str = "vector, fuente, uso, step, ren, nren, co2
ELECTRICIDAD, RED, SUMINISTRO, A, 0.5, 2.0, 0.42
GASNATURAL, RED, SUMINISTRO,A, 0.0, 1.1, 0.22
BIOCARBURANTE, RED, SUMINISTRO, A, 1.0, 0.1, 0.07
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.0, 0.0, 0.0
ELECTRICIDAD, COGENERACION, A_RED, A, 2.27, 0.23, 0.07
ELECTRICIDAD, COGENERACION, A_RED, B, 0.5, 2.0, 0.42
";

const TESTFP: &str = "vector, fuente, uso, step, ren, nren

ELECTRICIDAD, RED, SUMINISTRO, A, 0.5, 2.0, 0.42

ELECTRICIDAD, INSITU, SUMINISTRO,   A, 1.0, 0.0, 0.0
ELECTRICIDAD, INSITU, A_RED, A, 1.0, 0.0, 0.0
ELECTRICIDAD, INSITU, A_NEPB, A, 1.0, 0.0, 0.0
ELECTRICIDAD, INSITU, A_RED, B, 0.5, 2.0, 0.42
ELECTRICIDAD, INSITU, A_NEPB, B, 0.5, 2.0, 0.42

GASNATURAL, RED, SUMINISTRO,A, 0.0, 1.1, 0.22

BIOCARBURANTE, RED, SUMINISTRO, A, 1.1, 0.1, 0.07
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034, 0.018

MEDIOAMBIENTE, INSITU, SUMINISTRO,  A, 1.0, 0.0, 0.0
MEDIOAMBIENTE, RED, SUMINISTRO,  A, 1.0, 0.0, 0.0

ELECTRICIDAD, COGENERACION, SUMINISTRO,   A, 0.0, 0.0, 0.0
ELECTRICIDAD, COGENERACION, A_RED, A, 0.0, 2.5, 0.82
ELECTRICIDAD, COGENERACION, A_NEPB, A, 1.0, 0.0, 0.0
ELECTRICIDAD, COGENERACION, A_RED, B, 0.5, 2.0, 0.42
ELECTRICIDAD, COGENERACION, A_NEPB, B, 0.5, 2.0, 0.42
";

const TESTKEXP: f32 = 1.0;

fn get_ctefp_peninsula() -> Factors {
    let user_wf = UserWF {
        red1: None,
        red2: None,
        cogen_to_grid: None,
        cogen_to_nepb: None,
    };
    wfactors_from_loc("PENINSULA", &CTE_LOCWF_RITE2014, user_wf, CTE_USERWF).unwrap()
}

fn get_energydatalist() -> Components {
    use CSubtype::*;
    use CType::*;
    use Carrier::*;

    //3 PV BdC_normativo
    Components {
        cmeta: vec![],
        cdata: vec![
            Component {
                id: 0,
                values: vec![
                    9.67, 7.74, 4.84, 4.35, 2.42, 2.9, 3.87, 3.39, 2.42, 3.87, 5.8, 7.74,
                ],
                carrier: ELECTRICIDAD,
                ctype: CONSUMO,
                csubtype: EPB,
                service: Service::NDEF,
                comment: "".into(),
            },
            Component {
                id: 0,
                values: vec![
                    1.13, 1.42, 1.99, 2.84, 4.82, 5.39, 5.67, 5.11, 4.54, 3.40, 2.27, 1.42,
                ],
                carrier: ELECTRICIDAD,
                ctype: PRODUCCION,
                csubtype: INSITU,
                service: Service::NDEF,
                comment: "".into(),
            },
            Component {
                id: 0,
                values: vec![
                    21.48, 17.18, 10.74, 9.66, 5.37, 6.44, 8.59, 7.52, 5.37, 8.59, 12.89, 17.18,
                ],
                carrier: MEDIOAMBIENTE,
                ctype: CONSUMO,
                csubtype: EPB,
                service: Service::NDEF,
                comment: "".into(),
            },
            Component {
                id: 0,
                values: vec![
                    21.48, 17.18, 10.74, 9.66, 5.37, 6.44, 8.59, 7.52, 5.37, 8.59, 12.89, 17.18,
                ],
                carrier: MEDIOAMBIENTE,
                ctype: PRODUCCION,
                csubtype: INSITU,
                service: Service::NDEF,
                comment: "".into(),
            },
        ],
        zones: vec![],
        systems: vec![],
    }
}

fn components_from_file(path: &str) -> Components {
    let path = Path::new(path);
    let mut f = File::open(path).unwrap();
    let mut componentsstring = String::new();
    f.read_to_string(&mut componentsstring).unwrap();
    componentsstring.parse::<Components>().unwrap().normalize()
}

fn wfactors_from_file(path: &str) -> Factors {
    let path = Path::new(path);
    let mut f = File::open(path).unwrap();
    let mut wfactors_string = String::new();
    f.read_to_string(&mut wfactors_string).unwrap();
    let user_wf = UserWF {
        red1: None,
        red2: None,
        cogen_to_grid: None,
        cogen_to_nepb: None,
    };
    wfactors_from_str(&wfactors_string, user_wf, CTE_USERWF).unwrap()
}

///Approximate equality for RenNrenCo2 values
pub fn approx_equal(expected: RenNrenCo2, got: RenNrenCo2) -> bool {
    let dif_ren = expected.ren - got.ren;
    let dif_nren = expected.nren - got.nren;
    let dif_co2 = expected.co2 - got.co2;
    let res = dif_ren.abs() < 0.1 && dif_nren.abs() < 0.1 && dif_co2.abs() < 0.1;
    if !res {
        eprintln!(
            "Expected: {}, Got: {}, Diff: {:?}",
            expected,
            got,
            (dif_ren, dif_nren, dif_co2)
        );
    }
    res
}

#[test]
fn cte_balance_from_data() {
    let ENERGYDATALIST = get_energydatalist();
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&ENERGYDATALIST, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 178.9,
            nren: 37.1,
            co2: 6.3,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1_base() {
    let comps = components_from_file("test_data/extra/ejemplo1base.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 50.0,
            nren: 200.0,
            co2: 42.0
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1_base_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo1base.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 41.4,
            nren: 195.4,
            co2: 33.1
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1_PV() {
    let comps = components_from_file("test_data/extra/ejemplo1PV.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 75.0,
            nren: 100.0,
            co2: 21.0,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1_PV_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo1PV.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 70.7,
            nren: 97.7,
            co2: 16.5,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1xPV() {
    let comps = components_from_file("test_data/extra/ejemplo1xPV.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 120.0,
            nren: -80.0,
            co2: -16.8,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1xPV_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo1xPV.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 123.4,
            nren: -78.2,
            co2: -13.24
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1xPVk0() {
    let comps = components_from_file("test_data/extra/ejemplo1xPV.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, 0.0, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 100.0,
            nren: 0.0,
            co2: 0.0,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_1xPVk0_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo1xPV.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, 0.0, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 100.0,
            nren: 0.0,
            co2: 0.0,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_2xPVgas() {
    let comps = components_from_file("test_data/extra/ejemplo2xPVgas.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 30.0,
            nren: 169.0,
            co2: 33.4,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_2xPVgas_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo2xPVgas.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 32.7,
            nren: 187.0,
            co2: 41.3,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_3_PV_BdC() {
    let comps = components_from_file("test_data/extra/ejemplo3PVBdC.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 180.5,
            nren: 38.0,
            co2: 8.0,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_3_PV_BdC_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo3PVBdC.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 178.9,
            nren: 37.1,
            co2: 6.3,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_4_cgn_fosil() {
    let comps = components_from_file("test_data/extra/ejemplo4cgnfosil.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: -14.0,
            nren: 227.8,
            co2: 45.0
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_4_cgn_fosil_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo4cgnfosil.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: -10.3,
            nren: 252.4,
            co2: 55.8,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_5_cgn_biogas() {
    let comps = components_from_file("test_data/extra/ejemplo5cgnbiogas.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 159.8,
            nren: 69.8,
            co2: 21.3,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_5_cgn_biogas_normativo() {
    let comps = components_from_file("test_data/extra/ejemplo5cgnbiogas.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 151.3,
            nren: 77.8,
            co2: 18.8,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_6_K3() {
    let comps = components_from_file("test_data/extra/ejemplo6K3.csv");
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 1385.5,
            nren: -662.0,
            co2: -139.0,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_6_K3_wfactors_file() {
    let comps = components_from_file("test_data/extra/ejemplo6K3.csv");
    let FP: Factors = wfactors_from_file("test_data/factores_paso_test.csv");
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 1385.5,
            nren: -662.0,
            co2: 176.8,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 1009.5,
            nren: 842.0,
            co2: 176.8,
        },
        bal.balance_m2.A
    ));
}

// *** Ejemplos ISO/TR 52000-2:2016 ---------------------------

#[test]
fn cte_J1_Base_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ1_base.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 50.0,
            nren: 200.0,
            co2: 42.0,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 50.0,
            nren: 200.0,
            co2: 42.0,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J2_Base_PV_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ2_basePV.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 75.0,
            nren: 100.0,
            co2: 21.0,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 75.0,
            nren: 100.0,
            co2: 21.0,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J3_Base_PV_excess_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ3_basePVexcess.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 120.0,
            nren: -80.0,
            co2: -16.8,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 100.0,
            nren: 0.0,
            co2: 0.0,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J4_Base_PV_excess_kexp_0() {
    let comps = components_from_file("test_data/ejemploJ3_basePVexcess.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, 0.0, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 100.0,
            nren: 0.0,
            co2: 0.0
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 100.0,
            nren: 0.0,
            co2: 0.0,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J5_Gas_boiler_PV_aux_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ5_gasPV.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 30.0,
            nren: 169.0,
            co2: 33.4,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 20.0,
            nren: 209.0,
            co2: 41.8,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J6_Heat_pump_PV_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ6_HPPV.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 180.5,
            nren: 38.0,
            co2: 8.0,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 180.5,
            nren: 38.0,
            co2: 8.0,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J7_Co_generator_gas_plus_gas_boiler_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ7_cogenfuelgasboiler.csv");
    let FP: Factors = TESTFPJ7.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: -13.7,
            nren: 229.0,
            co2: 45.3,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 0.0,
            nren: 215.3,
            co2: 34.3,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J8_Co_generator_biogas_plus_gas_boiler_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ8_cogenbiogasboiler.csv");
    let FP: Factors = TESTFPJ8.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 144.3,
            nren: 71.0,
            co2: 21.6,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 95.8,
            nren: 119.5,
            co2: 31.1,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_J9_electricity_monthly_kexp_1() {
    let comps = components_from_file("test_data/ejemploJ9_electr.csv");
    let FP: Factors = TESTFPJ.parse().unwrap();
    let bal = energy_performance(&comps, &FP, TESTKEXP, 1.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 1385.5,
            nren: -662.0,
            co2: -139.0,
        },
        bal.balance_m2.B
    ));
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 1009.5,
            nren: 842.0,
            co2: 176.8,
        },
        bal.balance_m2.A
    ));
}

#[test]
fn cte_test_carriers_kexp_0() {
    let comps = components_from_file("test_data/cte_test_carriers.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, 0.0, 200.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 24.6,
            nren: 18.9,
            co2: 3.2,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_EPBD() {
    let comps = components_from_file("test_data/cteEPBD-N_R09_unif-ET5-V048R070-C1_peninsula.csv");
    let user_wf = UserWF {
        red1: Some(CTE_USERWF.red1),
        red2: Some(CTE_USERWF.red2),
        cogen_to_grid: None,
        cogen_to_nepb: None,
    };
    let FP = wfactors_from_loc("PENINSULA", &CTE_LOCWF_RITE2014, user_wf, CTE_USERWF).unwrap();
    let bal = energy_performance(&comps, &FP, 0.0, 217.4).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 2.2,
            nren: 38.4,
            co2: 8.2,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_new_services_format() {
    // Igual que N_R09, y usamos valores por defecto en función de normalize
    let comps = components_from_file("test_data/newServicesFormat.csv");
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, 0.0, 217.4).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 2.2,
            nren: 38.4,
            co2: 8.2,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_new_services_format_ACS() {
    // Igual que N_R09, y usamos valores por defecto en función de normalize
    let comps =
        components_from_file("test_data/newServicesFormat.csv").filter_by_epb_service(Service::ACS);
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&comps, &FP, 0.0, 217.4).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 0.0,
            nren: 12.4,
            co2: 2.9,
        },
        bal.balance_m2.B
    ));
}

#[test]
fn cte_force_electricity_prod_to_NDEF() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,CAL,20
ELECTRICIDAD,PRODUCCION,INSITU,CAL,40"
        .parse::<Components>()
        .unwrap()
        .normalize();
    assert_eq!(comps.cdata[1].service, Service::NDEF);
}

#[test]
fn cte_balance_byuse() {
    let ENERGYDATALIST = get_energydatalist();
    let FP = get_ctefp_peninsula();
    let bal = energy_performance(&ENERGYDATALIST, &FP, TESTKEXP, 1.0).unwrap();

    let mut result: HashMap<Service, RenNrenCo2> = HashMap::new();
    result.insert(
        Service::NDEF,
        RenNrenCo2 {
            ren: 178.88016,
            nren: 37.14554,
            co2: 6.292_309_8,
        },
    );

    assert_eq!(result, bal.balance_m2.B_byuse);
}

// Tests para demanda renovable de ACS

/// Efecto Joule con 60% PV (100kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_joule_60pv() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,100
ELECTRICIDAD,PRODUCCION,INSITU,ACS,60"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.60");
}

/// Gas natural (fp_nren = 1.1, con rend=0.9) y 60% de cobertura solar (100kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_gn_60pst() {
    let comps = "GASNATURAL,CONSUMO,EPB,ACS,44.44
MEDIOAMBIENTE,CONSUMO,EPB,ACS,60"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.60");
}

/// Biomasa rend 75% y PST (75kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_biomasa_10PST_100() {
    let comps = "BIOMASA,CONSUMO,EPB,ACS,100
MEDIOAMBIENTE,CONSUMO,EPB,ACS,10"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 75.0).unwrap();
    assert_eq!(format!("{:.3}", fraccion_ren_acs), "0.972");
}

/// Biomasa rend 75% (75kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_biomasa_100() {
    let comps = "BIOMASA,CONSUMO,EPB,ACS,100"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 75.0).unwrap();
    assert_eq!(format!("{:.3}", fraccion_ren_acs), "0.967");
}

/// Biomasa rend 75% + Biomasa densificada rend 75% cada una participando al 50% (75kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_biomasa_y_biomasa_densificada_100() {
    let comps = "#META CTE_DEMANDA_ACS_PCT_BIOMASA: 50
    #META CTE_DEMANDA_ACS_PCT_BIOMASADENSIFICADA: 50
    BIOMASA,CONSUMO,EPB,ACS,50
    BIOMASADENSIFICADA,CONSUMO,EPB,ACS,50"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let TESTFPEXT = format!(
        "{}\n{}\n{}",
        TESTFP,
        "BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034, 0.018",
        "BIOMASADENSIFICADA,RED,SUMINISTRO, A, 1.028, 0.085, 0.018" // Red de distrito 50% renovable
    );
    let FP: Factors = TESTFPEXT.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 75.0).unwrap();
    assert_eq!(format!("{:.3}", fraccion_ren_acs), "0.945");
}

/// Gas rend 90% (40% demanda -> 50kWh) + Biomasa rend 75% + Biomasa densificada rend 75% cada una participando al 50% (75kWh demanda ACS las dos)
#[test]
fn cte_ACS_demanda_ren_gas_biomasa_y_biomasa_densificada_125() {
    let comps = "#META CTE_DEMANDA_ACS_PCT_BIOMASA: 30
    #META CTE_DEMANDA_ACS_PCT_BIOMASADENSIFICADA: 30
    GASNATURAL,CONSUMO,EPB,ACS,55.556
    BIOMASA,CONSUMO,EPB,ACS,50
    BIOMASADENSIFICADA,CONSUMO,EPB,ACS,50"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let TESTFPEXT = format!(
        "{}\n{}\n{}",
        TESTFP,
        "BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034, 0.018",
        "BIOMASADENSIFICADA,RED,SUMINISTRO, A, 1.028, 0.085, 0.018" // Red de distrito 50% renovable
    );
    let FP: Factors = TESTFPEXT.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 125.0).unwrap();
    // Las dos biomasas producen lo mismo que antes de renovable = 0.945 * 75 = 70.875 , con el nuevo total de demanda (125kWh) -> 70.875 / 125.0 = 0.567
    assert_eq!(format!("{:.3}", fraccion_ren_acs), "0.567");
}

/// Red de distrito, red1 50% renovable y red2 10% renovable (100kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_red1_red2() {
    let comps = "RED1,CONSUMO,EPB,ACS,50
RED2,CONSUMO,EPB,ACS,50"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let TESTFPEXT = format!(
        "{}\n{}\n{}",
        TESTFP,
        "RED1,RED,SUMINISTRO,A,0.5,0.5,0.0", // Red de distrito 50% renovable
        "RED2,RED,SUMINISTRO,A,0.1,0.9,0.0"  // Red de distrito 10% renovable
    );
    let FP: Factors = TESTFPEXT.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.30");
}

/// Bomba de calor (SCOP=2.5) (100kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_bdc_60ma() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,40.0
MEDIOAMBIENTE,CONSUMO,EPB,ACS,60"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.60");
}

/// Bomba de calor (SCOP=2.5) + 10kWh PV (100kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_bdc_60ma_10pv() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,40.0
MEDIOAMBIENTE,CONSUMO,EPB,ACS,60
ELECTRICIDAD,PRODUCCION,INSITU,NDEF,10"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.70");
}

/// Bomba de calor (SCOP=2.5) + 10kWh PV con 5kWh consumo eléctrico NEPB (100kWh demanda ACS)
/// Debería dar igual el tipo de uso definido para NEPB
#[test]
fn cte_ACS_demanda_ren_bdc_60ma_10pv_nEPB() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,40.0
MEDIOAMBIENTE,CONSUMO,EPB,ACS,60
ELECTRICIDAD,PRODUCCION,INSITU,NDEF,10
ELECTRICIDAD,CONSUMO,NEPB,NDEF,40.0"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.70");
}

/// Bomba de calor (SCOP=2.5) con electricidad cogenerada (100kWh demanda ACS)
// Caso no cubierto, al definirse cogeneración (solo permitimos la parte térmica)
// Si hay más de un suministro que no sea insitu no podemos hacer el cálculo
#[test]
fn cte_ACS_demanda_ren_fail_bdc_60ma_10cgn() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,40.0
MEDIOAMBIENTE,CONSUMO,EPB,ACS,60
ELECTRICIDAD,PRODUCCION,COGENERACION,ACS,10"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0);
    assert!(fraccion_ren_acs.is_err());
}

/// Bomba de calor (SCOP=2.5) y 25% caldera de GN (rend. 0.9) (100kWh demanda ACS)
#[test]
fn cte_ACS_demanda_ren_bdc_45ma_25gn() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,30.0
MEDIOAMBIENTE,CONSUMO,EPB,ACS,45
GASNATURAL,CONSUMO,EPB,ACS,27.88"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.45");
}

/// Bomba de calor (SCOP=2.0) y 25% caldera de GN (rend. 0.9) (100kWh demanda ACS)
/// En este caso se excluye la producción de medioambiente puesto que no es renovable
#[test]
fn cte_ACS_demanda_ren_bdc_38ma__25gn_excluye_medioambiente() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,37.5
MEDIOAMBIENTE,CONSUMO,EPB,ACS,37.5# CTEEPBD_EXCLUYE_SCOP_ACS
GASNATURAL,CONSUMO,EPB,ACS,27.88"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0).unwrap();
    assert_eq!(format!("{:.2}", fraccion_ren_acs), "0.00");
}

/// Bomba de calor (SCOP=2.5) y 25% caldera de GN y de BIOMASA (rend. 0.9) (100kWh demanda ACS)
// Falla al haber BIOMASA y otro suministro de red que no es insitu
#[test]
fn cte_ACS_demanda_ren_fail_bdc_45ma_25gn_y_biomasa() {
    let comps = "ELECTRICIDAD,CONSUMO,EPB,ACS,30.0
MEDIOAMBIENTE,CONSUMO,EPB,ACS,45
BIOMASA,CONSUMO,EPB,ACS,13.94
GASNATURAL,CONSUMO,EPB,ACS,13.94"
        .parse::<Components>()
        .unwrap()
        .normalize();
    let FP: Factors = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 100.0);
    assert!(fraccion_ren_acs.is_err());
}

#[test]
fn cte_ACS_demanda_ren_excluye_aux() {
    // Caso de GT con exclusión de líneas de consumo eléctrico auxiliar
    let comps = components_from_file("test_data/acs_demanda_ren_con_exclusion_auxiliares.csv");
    let FP = TESTFP.parse().unwrap();
    let fraccion_ren_acs = fraccion_renovable_acs_nrb(&comps, &FP, 4549.0).unwrap();
    assert_eq!(format!("{:.3}", fraccion_ren_acs), "0.967");
}

/// Componentes con id de sistema explicitados, usos no EPB y exportación a usos nEPB y a la red
#[test]
fn new_format_with_system_id() {
    let comps = "# Bomba de calor 1
    1,ELECTRICIDAD,CONSUMO,EPB,ACS,100 # BdC 1
    1,MEDIOAMBIENTE,CONSUMO,EPB,ACS,150 # BdC 1
    2,ELECTRICIDAD,CONSUMO,NEPB,NDEF,10 # Ascensores
    # Bomba de calor 2
    2,ELECTRICIDAD,CONSUMO,EPB,CAL,200 # BdC 2
    2,MEDIOAMBIENTE,CONSUMO,EPB,CAL,300 # BdC 2
    # Producción fotovoltaica in situ
    1,ELECTRICIDAD,PRODUCCION,INSITU,NDEF,200 # PV
    2,ELECTRICIDAD,PRODUCCION,INSITU,ACS,100 # PV
    3,ELECTRICIDAD,PRODUCCION,INSITU,NDEF,5 # PV
    # Producción de energía ambiente dada por el usuario
    0,MEDIOAMBIENTE,PRODUCCION,INSITU,ACS,100 # Producción declarada de sistema sin consumo (no reduce energía a compensar)
    1,MEDIOAMBIENTE,PRODUCCION,INSITU,ACS,100 # Producción declarada de sistema con consumo (reduce energía a compensar)
    2,MEDIOAMBIENTE,PRODUCCION,INSITU,ACS,100 # Producción declarada de sistema sin ese servicio consumo (no reduce energía a compensar)
    # Compensación de energía ambiente a completar por CteEPBD"
        .parse::<Components>()
        .unwrap();
    let FP: Factors = TESTFP.parse().unwrap();
    let bal = energy_performance(&comps, &FP, 1.0, 100.0).unwrap();
    assert!(approx_equal(
        RenNrenCo2 {
            ren: 7.525,
            nren: -0.10,
            co2: -0.021,
        },
        bal.balance_m2.B
    ));
    let balance_el = &bal.balance_cr[&Carrier::ELECTRICIDAD];
    // NEPB used energy
    assert_eq!("10.000", format!("{:.3}", balance_el.used_nEPB_an));
    // Produced energy from all origins and used for EPB services
    assert_eq!("300.000", format!("{:.3}", balance_el.produced_used_EPus_an));
    // Exported energy to non EPB uses
    assert_eq!("5.000", format!("{:.3}", balance_el.exported_nEPB_an));
}
