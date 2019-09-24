// Copyright (c) 2016-2017 Ministerio de Fomento
//                    Instituto de Ciencias de la Construcción Eduardo Torroja (IETcc-CSIC)

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>,
//            Daniel Jiménez González <dani@ietcc.csic.es>

/*! # Manejo de factores de paso para el CTE

Factores de paso y utilidades para la gestión de factores de paso para el CTE

*/

use itertools::Itertools;

use crate::{
    CSubtype, Carrier, Components, Dest, EpbdError, Factor, Factors, Meta, MetaVec, RenNrenCo2,
    Result, Source, Step,
};

// Localizaciones válidas para CTE
// const CTE_LOCS: [&str; 4] = ["PENINSULA", "BALEARES", "CANARIAS", "CEUTAMELILLA"];

// Valores bien conocidos de metadatos:
// CTE_LOCALIZACION -> str

/// Vectores considerados dentro del perímetro NEARBY (a excepción de la ELECTRICIDAD in situ).
pub const CTE_NRBY: [Carrier; 5] = [
    Carrier::BIOMASA,
    Carrier::BIOMASADENSIFICADA,
    Carrier::RED1,
    Carrier::RED2,
    Carrier::MEDIOAMBIENTE,
]; // Ver B.23. Solo biomasa sólida

// ---------------- Valores por defecto y definibles por el usuario -----------------------
#[derive(Debug)]
pub struct CteUserWF<T> {
    /// Factores de paso de redes de distrito 1.
    /// RED1, RED, SUMINISTRO, A, ren, nren
    pub red1: T,
    /// Factores de paso de redes de distrito 2.
    /// RED2, RED, SUMINISTRO, A, ren, nren
    pub red2: T,
    /// Factores de paso para exportación a la red (paso A) de electricidad cogenerada.
    /// ELECTRICIDAD, COGENERACION, A_RED, A, ren, nren
    pub cogen_to_grid: T,
    /// Factores de paso para exportación a usos no EPB (paso A) de electricidad cogenerada.
    /// ELECTRICIDAD, COGENERACION, A_NEPB, A, ren, nren
    pub cogen_to_nepb: T,
}

/// Valores por defecto para factores de paso
pub struct CteDefaultsWF {
    /// Factores de paso de usuario
    pub user: CteUserWF<RenNrenCo2>,
    /// Factores de paso reglamentarios para la Península
    pub loc_peninsula: &'static str,
    /// Factores de paso reglamentarios para Baleares.
    pub loc_baleares: &'static str,
    /// Factores de paso reglamentarios para Canarias.
    pub loc_canarias: &'static str,
    /// Factores de paso reglamentarios para Ceuta y Melilla.
    pub loc_ceutamelilla: &'static str,
}

/// Valores por defecto para energía primaria
macro_rules! build_wf_2013 {
    ($loc:literal, $ren:literal, $nren:literal, $co2:literal) => {
        concat!("#META CTE_FUENTE: RITE2014", "\n",
        "#META CTE_LOCALIZACION: ", $loc, "\n",
        "#META CTE_FUENTE_COMENTARIO: Factores de paso (kWh/kWh_f,kWh/kWh_f,kg_CO2/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085, 0.018 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034, 0.018 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085, 0.018 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082, 0.472 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190, 0.252 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179, 0.311 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201, 0.254 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, ", stringify!($ren), ", ", stringify!($nren), ", ", stringify!($co2), " # Recursos usados para el suministro desde la red
")};
}

/// Factores de paso reglamentarios (RITE 20/07/2014). Usados también en DB-HE 2013
pub const WF_RITE2014: CteDefaultsWF = CteDefaultsWF {
    user: CteUserWF {
        red1: RenNrenCo2 {
            ren: 0.0,
            nren: 1.3,
            co2: 0.3,
        },
        red2: RenNrenCo2 {
            ren: 0.0,
            nren: 1.3,
            co2: 0.3,
        },
        cogen_to_grid: RenNrenCo2 {
            ren: 0.0,
            nren: 2.5,
            co2: 0.3,
        },
        cogen_to_nepb: RenNrenCo2 {
            ren: 0.0,
            nren: 2.5,
            co2: 0.3,
        },
    },
    loc_peninsula: build_wf_2013!("PENINSULA", 0.414, 1.954, 0.331),
    loc_baleares: build_wf_2013!("BALEARES", 0.082, 2.968, 0.932),
    loc_canarias: build_wf_2013!("CANARIAS", 0.070, 2.924, 0.776),
    loc_ceutamelilla: build_wf_2013!("CEUTAMELILLA", 0.072, 2.718, 0.721),
};

// --------------------- Utilidades E/S ------------------------

/// Lee factores de paso desde cadena y sanea los resultados.
pub fn wfactors_from_str(
    wfactorsstring: &str,
    user: &CteUserWF<Option<RenNrenCo2>>,
    defaults: &CteDefaultsWF,
) -> Result<Factors> {
    let mut wfactors: Factors = wfactorsstring.parse()?;
    set_user_wfactors(&mut wfactors, user);
    fix_wfactors(wfactors, defaults)
}

/// Genera factores de paso a partir de localización.
///
/// Usa localización (PENINSULA, CANARIAS, BALEARES, CEUTAMELILLA),
/// factores de paso de cogeneración, y factores de paso para RED1 y RED2
pub fn wfactors_from_loc(
    loc: &str,
    user: &CteUserWF<Option<RenNrenCo2>>,
    defaults: &CteDefaultsWF,
) -> Result<Factors> {
    let wfactorsstring = match &*loc {
        "PENINSULA" => defaults.loc_peninsula,
        "BALEARES" => defaults.loc_baleares,
        "CANARIAS" => defaults.loc_canarias,
        "CEUTAMELILLA" => defaults.loc_ceutamelilla,
        _ => Err(EpbdError::Location(loc.to_string()))?,
    };
    let mut wfactors: Factors = wfactorsstring.parse()?;
    set_user_wfactors(&mut wfactors, user);
    fix_wfactors(wfactors, defaults)
}

/// Genera factores de paso a partir de metadatos de componentes.
///
/// Usa localización (CTE_LOC), y factores de usuario (CTE_COGEN, CTE_COGENNEPB, CTE_RED1, CTE_RED2)
pub fn wfactors_from_meta(components: &Components, defaults: &CteDefaultsWF) -> Result<Factors> {
    let loc = components.get_meta("CTE_LOCALIZACION").unwrap_or_default();
    let user = CteUserWF {
        red1: components.get_meta_rennren("CTE_RED1"),
        red2: components.get_meta_rennren("CTE_RED2"),
        cogen_to_grid: components.get_meta_rennren("CTE_COGEN"),
        cogen_to_nepb: components.get_meta_rennren("CTE_COGENNEPB"),
    };
    let wfactorsstring = match loc.as_str() {
        "PENINSULA" => defaults.loc_peninsula,
        "BALEARES" => defaults.loc_baleares,
        "CANARIAS" => defaults.loc_canarias,
        "CEUTAMELILLA" => defaults.loc_ceutamelilla,
        _ => Err(EpbdError::Location(loc.to_string()))?,
    };
    let mut wfactors: Factors = wfactorsstring.parse()?;
    set_user_wfactors(&mut wfactors, &user);
    fix_wfactors(wfactors, defaults)
}

/// Actualiza los factores definibles por el usuario (cogen_to_grid, cogen_to_nepb, red1 y red2)
pub fn set_user_wfactors(wfactors: &mut Factors, user: &CteUserWF<Option<RenNrenCo2>>) {
    // ------ Cogeneración a red ----------
    if let Some(ucog) = user.cogen_to_grid {
        if let Some(factor) = wfactors.wdata.iter_mut().find(|f| {
            f.source == Source::COGENERACION && f.step == Step::A && f.dest == Dest::A_RED
        }) {
            factor.ren = ucog.ren;
            factor.nren = ucog.nren;
            factor.co2 = ucog.co2;
        } else {
            wfactors.wdata.push(Factor::new(
                Carrier::ELECTRICIDAD,
                Source::COGENERACION,
                Dest::A_RED,
                Step::A,
                ucog.ren,
                ucog.nren,
                ucog.co2,
                "Factor de usuario",
            ));
        };
    };

    // ------ Cogeneración a usos no EPB ----------
    if let Some(ucog) = user.cogen_to_nepb {
        if let Some(factor) = wfactors.wdata.iter_mut().find(|f| {
            f.source == Source::COGENERACION && f.step == Step::A && f.dest == Dest::A_NEPB
        }) {
            factor.ren = ucog.ren;
            factor.nren = ucog.nren;
            factor.co2 = ucog.co2;
        } else {
            wfactors.wdata.push(Factor::new(
                Carrier::ELECTRICIDAD,
                Source::COGENERACION,
                Dest::A_NEPB,
                Step::A,
                ucog.ren,
                ucog.nren,
                ucog.co2,
                "Factor de usuario",
            ));
        };
    };

    // ------ Red1 ----------
    if let Some(ured1) = user.red1 {
        if let Some(factor) = wfactors
            .wdata
            .iter_mut()
            .find(|f| f.carrier == Carrier::RED1 && f.step == Step::A && f.dest == Dest::SUMINISTRO)
        {
            factor.ren = ured1.ren;
            factor.nren = ured1.nren;
            factor.co2 = ured1.co2;
        } else {
            wfactors.wdata.push(Factor::new(
                Carrier::RED1,
                Source::RED,
                Dest::SUMINISTRO,
                Step::A,
                ured1.ren,
                ured1.nren,
                ured1.co2,
                "Factor de usuario",
            ));
        };
    };

    // ------ Red2 ----------
    if let Some(ured2) = user.red2 {
        if let Some(factor) = wfactors
            .wdata
            .iter_mut()
            .find(|f| f.carrier == Carrier::RED2 && f.step == Step::A && f.dest == Dest::SUMINISTRO)
        {
            factor.ren = ured2.ren;
            factor.nren = ured2.nren;
            factor.co2 = ured2.co2;
        } else {
            wfactors.wdata.push(Factor::new(
                Carrier::RED2,
                Source::RED,
                Dest::SUMINISTRO,
                Step::A,
                ured2.ren,
                ured2.nren,
                ured2.co2,
                "Factor de usuario",
            ));
        };
    };
}

/// Asegura consistencia de factores de paso definidos y deduce algunos de los que falten.
///
/// Realiza los siguientes pasos:
/// - asegura definición de factores de producción in situ
/// - asegura definición de factores desde la red para todos los vectores
/// - asegura que factor paso A para suministro de cogeneración es 0.0 (se considera en vector original)
/// - asegura definición de factores a la red para vectores con exportación
/// - asegura que existe RED1 | RED2 en suministro
///
/// TODO: se deberían separar algunos de estos pasos como métodos de CteFactorsExt
pub fn fix_wfactors(mut wfactors: Factors, defaults: &CteDefaultsWF) -> Result<Factors> {
    // Vectores existentes
    let wf_carriers: Vec<_> = wfactors.wdata.iter().map(|f| f.carrier).unique().collect();

    // Asegura que existe MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.0, 0.0
    let has_ma_insitu_input_a = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::MEDIOAMBIENTE
            && f.source == Source::INSITU
            && f.dest == Dest::SUMINISTRO
            && f.step == Step::A
    });
    if !has_ma_insitu_input_a {
        wfactors.wdata.push(Factor::new(
            Carrier::MEDIOAMBIENTE,
            Source::INSITU,
            Dest::SUMINISTRO,
            Step::A,
            1.0,
            0.0,
            0.0,
            "Recursos usados para obtener energía térmica del medioambiente",
        ));
    }
    // Asegura que existe MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.0, 0.0
    let has_ma_red_input_a = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::MEDIOAMBIENTE
            && f.source == Source::RED
            && f.dest == Dest::SUMINISTRO
            && f.step == Step::A
    });
    if !has_ma_red_input_a {
        // MEDIOAMBIENTE, RED, SUMINISTRO, A, ren, nren === MEDIOAMBIENTE, INSITU, SUMINISTRO, A, ren, nren
        wfactors.wdata.push(Factor::new(
            Carrier::MEDIOAMBIENTE,
            Source::RED,
            Dest::SUMINISTRO,
            Step::A,
            1.0,
            0.0,
            0.0,
            "Recursos usados para obtener energía térmica del medioambiente (red ficticia)",
        ));
    }
    // Asegura que existe ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.0, 0.0 si hay ELECTRICIDAD
    let has_elec_and_elec_insitu_input_a = wf_carriers.contains(&Carrier::ELECTRICIDAD)
        && !wfactors.wdata.iter().any(|f| {
            f.carrier == Carrier::ELECTRICIDAD
                && f.source == Source::INSITU
                && f.dest == Dest::SUMINISTRO
        });
    if has_elec_and_elec_insitu_input_a {
        wfactors.wdata.push(Factor::new(
            Carrier::ELECTRICIDAD,
            Source::INSITU,
            Dest::SUMINISTRO,
            Step::A,
            1.0,
            0.0,
            0.0,
            "Recursos usados para generar electricidad in situ",
        ));
    }
    // Asegura definición de factores de red para todos los vectores energéticos
    let has_grid_factors_for_all_carriers = wf_carriers.iter().all(|&c| {
        wfactors.wdata.iter().any(|f| {
            f.carrier == c
                && f.source == Source::RED
                && f.dest == Dest::SUMINISTRO
                && f.step == Step::A
        })
    });
    if !has_grid_factors_for_all_carriers {
        Err(EpbdError::FactorNotFound(
            "factores de red VECTOR, INSITU, SUMINISTRO, A, fren?, fnren?".into(),
        ))?;
    }
    // En paso A, el factor SUMINISTRO de cogeneración es 0.0, 0.0 ya que el impacto se tiene en cuenta en el suministro del vector de generación
    let has_cogen_input = wfactors
        .wdata
        .iter()
        .any(|f| f.source == Source::COGENERACION && f.dest == Dest::SUMINISTRO);
    if !has_cogen_input {
        wfactors.wdata.push(Factor::new(
            Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::SUMINISTRO, Step::A, 0.0, 0.0, 0.0,
            "Factor de paso generado (el impacto de la cogeneración se tiene en cuenta en el vector de suministro)"));
    }
    // Asegura que todos los vectores con exportación tienen factores de paso a la red y a usos no EPB
    let exp_carriers = [
        (Carrier::ELECTRICIDAD, Source::INSITU),
        (Carrier::ELECTRICIDAD, Source::COGENERACION),
        (Carrier::MEDIOAMBIENTE, Source::INSITU),
    ];
    for (c, s) in &exp_carriers {
        // Asegura que existe VECTOR, SRC, A_RED | A_NEPB, A, ren, nren
        let fp_a_input = wfactors
            .wdata
            .iter()
            .find(|f| {
                f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::SUMINISTRO
            })
            .and_then(|f| Some(f.clone()));

        let has_to_grid = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::A_RED
        });
        if !has_to_grid {
            if *s != Source::COGENERACION {
                // VECTOR, SRC, A_RED, A, ren, nren === VECTOR, SRC, SUMINISTRO, A, ren, nren
                if fp_a_input.is_some() {
                    let f = fp_a_input.as_ref().unwrap();
                    wfactors.wdata.push(Factor {
                        dest: Dest::A_RED,
                        step: Step::A,
                        comment: "Recursos usados para producir la energía exportada a la red"
                            .to_string(),
                        ..*f
                    });
                } else {
                    Err(EpbdError::FactorNotFound(format!(
                        "suministro del vector {} para definir exportación a la red en paso A",
                        c
                    )))?;
                }
            } else {
                // TODO: Igual aquí hay que indicar que se deben definir factores de usuario en un bail y no hacer nada
                // Asegura que existe ELECTRICIDAD, COGENERACION, A_RED, A, ren, nren - ver 9.6.6.2.3
                let has_cogen_to_grid = wfactors.wdata.iter().any(|f| {
                    f.carrier == Carrier::ELECTRICIDAD
                        && f.source == Source::COGENERACION
                        && f.dest == Dest::A_RED
                        && f.step == Step::A
                });
                if !has_cogen_to_grid {
                    let cogen = defaults.user.cogen_to_grid;
                    wfactors.wdata.push(Factor::new(
                    Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::A_RED, Step::A, cogen.ren, cogen.nren, cogen.co2,
                    "Recursos usados para producir electricidad cogenerada y vertida a la red. Valor predefinido"));
                }
            }
        }
        let has_to_nepb = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::A_NEPB
        });
        if !has_to_nepb {
            if *s != Source::COGENERACION {
                // VECTOR, SRC, A_NEPB, A, ren, nren == VECTOR, SRC, SUMINISTRO, A, ren, nren
                if fp_a_input.is_some() {
                    let f = fp_a_input.as_ref().unwrap();
                    wfactors.wdata.push(Factor {
                        dest: Dest::A_NEPB,
                        step: Step::A,
                        comment: "Recursos usados para producir la energía exportada a usos no EPB"
                            .to_string(),
                        ..*f
                    });
                } else {
                    Err(EpbdError::FactorNotFound(format!(
                        "suministro del vector {} para definir exportación a usos no EPB en paso A",
                        c
                    )))?;
                }
            } else {
                // TODO: Igual aquí hay que indicar que se deben definir factores de usuario en un bail y no hacer nada
                // TODO: Si está definido para A_RED (no por defecto) y no para A_NEPB, qué hacemos? usamos por defecto? usamos igual a A_RED?
                // Asegura que existe ELECTRICIDAD, COGENERACION, A_NEPB, A, ren, nren - ver 9.6.6.2.3
                let has_cogen_to_nepb = wfactors.wdata.iter().any(|f| {
                    f.carrier == Carrier::ELECTRICIDAD
                        && f.source == Source::COGENERACION
                        && f.dest == Dest::A_NEPB
                        && f.step == Step::A
                });
                if !has_cogen_to_nepb {
                    let cogen = defaults.user.cogen_to_nepb;
                    wfactors.wdata.push(Factor::new(
                        Carrier::ELECTRICIDAD,
                        Source::COGENERACION,
                        Dest::A_NEPB,
                        Step::A,
                        cogen.ren,
                        cogen.nren,
                        cogen.co2,
                        "Valor predefinido",
                    ));
                }
            }
        }
        // Asegura que existe VECTOR, SRC, A_RED | A_NEPB, B, ren, nren
        let fp_a_red_input = wfactors
            .wdata
            .iter()
            .find(|f| {
                f.carrier == *c
                    && f.source == Source::RED
                    && f.dest == Dest::SUMINISTRO
                    && f.step == Step::A
            })
            .and_then(|f| Some(f.clone()));
        let has_to_grid_b = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::A_RED
        });
        if !has_to_grid_b {
            // VECTOR, SRC, A_RED, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
            if fp_a_red_input.is_some() {
                let f = fp_a_red_input.as_ref().unwrap();
                wfactors.wdata.push(Factor::new(f.carrier, *s, Dest::A_RED, Step::B, f.ren, f.nren, f.co2,
                "Recursos ahorrados a la red por la energía producida in situ y exportada a la red"));
            } else {
                Err(EpbdError::FactorNotFound(format!(
                    "suministro del vector {} para exportación a la red en paso B",
                    c
                )))?;
            }
        }
        let has_to_nepb_b = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::A_NEPB
        });
        if !has_to_nepb_b {
            // VECTOR, SRC, A_NEPB, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
            if fp_a_red_input.is_some() {
                let f = fp_a_red_input.as_ref().unwrap();
                wfactors.wdata.push(Factor::new(f.carrier, *s, Dest::A_NEPB, Step::B, f.ren, f.nren, f.co2,
                "Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB"));
            } else {
                Err(EpbdError::FactorNotFound(format!(
                    "suministro del vector {} para exportación a usos no EPB en paso B",
                    c
                )))?;
            }
        }
    }

    // Asegura que existe RED1 | RED2, RED, SUMINISTRO, A, ren, nren
    let has_red1_red_input = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::RED1 && f.source == Source::RED && f.dest == Dest::SUMINISTRO
    });
    if !has_red1_red_input {
        let red1 = defaults.user.red1;
        wfactors.wdata.push(Factor::new(Carrier::RED1, Source::RED, Dest::SUMINISTRO, Step::A,
          red1.ren, red1.nren, red1.co2, "Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)"));
    }
    let has_red2_red_input = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::RED2 && f.source == Source::RED && f.dest == Dest::SUMINISTRO
    });
    if !has_red2_red_input {
        let red2 = defaults.user.red2;
        wfactors.wdata.push(Factor::new(Carrier::RED2, Source::RED, Dest::SUMINISTRO, Step::A,
          red2.ren, red2.nren, red2.co2, "Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)"));
    }

    Ok(wfactors)
}

/// Elimina factores de paso no usados en los datos de vectores energéticos.
///
/// Elimina los factores:
///  - de vectores que no aparecen en los datos
///  - de cogeneración si no hay cogeneración
///  - para exportación a usos no EPB si no se aparecen en los datos
///  - de electricidad in situ si no aparece una producción de ese tipo
pub fn strip_wfactors(wfactors: &mut Factors, components: &Components) {
    let wf_carriers: Vec<_> = components
        .cdata
        .iter()
        .map(|c| c.carrier)
        .unique()
        .collect();
    let has_cogen = components
        .cdata
        .iter()
        .any(|c| c.csubtype == CSubtype::COGENERACION);
    let has_nepb = components
        .cdata
        .iter()
        .any(|c| c.csubtype == CSubtype::NEPB);
    let has_elec_insitu = components
        .cdata
        .iter()
        .any(|c| c.carrier == Carrier::ELECTRICIDAD && c.csubtype == CSubtype::INSITU);
    wfactors.wdata.retain(|f| wf_carriers.contains(&f.carrier));
    wfactors
        .wdata
        .retain(|f| f.source != Source::COGENERACION || has_cogen);
    wfactors
        .wdata
        .retain(|f| f.dest != Dest::A_NEPB || has_nepb);
    wfactors.wdata.retain(|f| {
        f.carrier != Carrier::ELECTRICIDAD || f.source != Source::INSITU || has_elec_insitu
    });
}

/// Convierte factores de paso con perímetro "distant" a factores de paso "nearby".
pub fn wfactors_to_nearby(wfactors: &Factors) -> Factors {
    // Los elementos que tiene origen en la RED (!= INSITU, != COGENERACION)
    // y no están en la lista CTE_NRBY cambian sus factores de paso
    // de forma que ren' = 0 y nren' = ren + nren.
    // ATENCIÓN: ¡¡La producción eléctrica de la cogeneración entra con (factores ren:0, nren:0)!!
    let mut wmeta = wfactors.wmeta.clone();
    let mut wdata: Vec<Factor> = Vec::new();

    for f in wfactors.wdata.iter().cloned() {
        if f.source == Source::INSITU
            || f.source == Source::COGENERACION
            || CTE_NRBY.contains(&f.carrier)
        {
            wdata.push(f)
        } else {
            wdata.push(Factor::new(
                f.carrier,
                f.source,
                f.dest,
                f.step,
                0.0,
                f.ren + f.nren,
                f.co2, // ¿Esto es lo que tiene más sentido?
                format!("Perímetro nearby: {}", f.comment),
            ))
        }
    }
    wmeta.push(Meta::new("CTE_PERIMETRO", "NEARBY"));
    Factors { wmeta, wdata }
}
