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

use failure::Error;
use itertools::Itertools;
use std::f32::EPSILON;

use rennren::RenNren;
use types::{Balance, Component, Components, Factor, Factors, Meta, MetaVec};
use types::{CSubtype, CType, Carrier, Dest, Service, Source, Step};
use vecops::{veckmul, veclistsum, vecvecdif};

// ---------------------------------------------------------------------------------------------------------
// Valores reglamentarios
//
// Orientados al cumplimiento del DB-HE del (Código Técnico de la Edificación CTE).
//
// Factores de paso basados en el consumo de energía primaria
// Factores de paso constantes a lo largo de los intervalos de cálculo
// ---------------------------------------------------------------------------------------------------------

/// Valor por defecto del área de referencia.
pub const AREAREF_DEFAULT: f32 = 1.0;
/// Valor predefinido del factor de exportación. Valor reglamentario.
pub const KEXP_DEFAULT: f32 = 0.0;

/// Valores por defecto para factores de paso de redes de distrito 1.
pub const CTE_RED_DEFAULTS_RED1: RenNren = RenNren {
    ren: 0.0,
    nren: 1.3,
}; // RED1, RED, input, A, ren, nren

/// Valores por defecto para factores de paso de redes de distrito 2.
pub const CTE_RED_DEFAULTS_RED2: RenNren = RenNren {
    ren: 0.0,
    nren: 1.3,
}; // RED2, RED, input, A, ren, nren

/// Valores por defecto para exportación a la red (paso A) de electricidad cogenerada.
pub const CTE_COGEN_DEFAULTS_TO_GRID: RenNren = RenNren {
    ren: 0.0,
    nren: 2.5,
}; // ELECTRICIDAD, COGENERACION, to_grid, A, ren, nren

/// Valores por defecto para exportación a usos no EPB (paso A) de electricidad cogenerada.
pub const CTE_COGEN_DEFAULTS_TO_NEPB: RenNren = RenNren {
    ren: 0.0,
    nren: 2.5,
}; // ELECTRICIDAD, COGENERACION, to_nEPB, A, ren, nren

// Localizaciones válidas para CTE
// const CTE_LOCS: [&str; 4] = ["PENINSULA", "BALEARES", "CANARIAS", "CEUTAMELILLA"];

// Valores bien conocidos de metadatos:
// CTE_AREAREF -> num
// CTE_KEXP -> num
// CTE_LOCALIZACION -> str
// CTE_COGEN -> num, num
// CTE_RED1 -> num, num
// CTE_RED2 -> num, num

/// Factores de paso reglamentarios (RITE 20/07/2014) para Península.
const CTE_FP_PENINSULA: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: PENINSULA
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, input, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, input, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, input, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, input, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, input, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, input, A, 0.030, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, input, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, input, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, input, A, 0.414, 1.954 # Recursos usados para suministrar electricidad (PENINSULA) desde la red
";

/// Factores de paso reglamentarios (RITE 20/07/2014) para Baleares.
const CTE_FP_BALEARES: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: BALEARES
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, input, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, input, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, input, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, input, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, input, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, input, A, 0.030, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, input, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, input, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, input, A, 0.082, 2.968 # Recursos usados para suministrar electricidad (BALEARES) desde la red
";

/// Factores de paso reglamentarios (RITE 20/07/2014) para Canarias.
const CTE_FP_CANARIAS: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CANARIAS
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, input, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, input, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, input, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, input, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, input, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, input, A, 0.030, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, input, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, input, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, input, A, 0.070, 2.924 # Recursos usados para suministrar electricidad (CANARIAS) desde la red
";

/// Factores de paso reglamentarios (RITE 20/07/2014) para Ceuta y Melilla.
const CTE_FP_CEUTAMELILLA: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CEUTAMELILLA
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, input, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, input, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, input, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, input, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, input, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, input, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, input, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, input, A, 0.030, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, input, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, input, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, input, A, 0.072, 2.718 # Recursos usados para suministrar electricidad (CEUTA Y MELILLA) desde la red
";

// -------------------------------------------------------------------------------------
// Utilidades de validación y generación
// -------------------------------------------------------------------------------------

// -------------------- vectores energéticos -------------------------------------------

/// Asegura vectores válidos y balance de consumos de vectores de producción in situ.
///
/// Completa el balance de las producciones in situ cuando el consumo de esos vectores supera la producción
/// Los metadatos, servicios y coherencia de los vectores se aseguran ya en el parsing
pub fn fix_components(components: &mut Components) {
    let envcomps: Vec<_> = components
        .cdata
        .iter()
        .cloned()
        .filter(|c| c.carrier == Carrier::MEDIOAMBIENTE)
        .collect();
    let services: Vec<_> = envcomps.iter().map(|c| c.service).unique().collect();

    let mut balancecomps: Vec<Component> = services
        .iter()
        .map(|&service| {
            let ecomps = envcomps.iter().filter(|c| c.service == service);
            let consumed: Vec<_> = ecomps
                .clone()
                .filter(|c| c.ctype == CType::CONSUMO)
                .collect();
            if consumed.is_empty() {
                return None;
            }; // No hay consumo
            let mut unbalanced_values = veclistsum(
                consumed
                    .iter()
                    .map(|v| &v.values)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            let produced: Vec<_> = ecomps
                .clone()
                .filter(|c| c.ctype == CType::PRODUCCION)
                .collect();
            if !produced.is_empty() {
                let totproduced = veclistsum(
                    produced
                        .iter()
                        .map(|v| &v.values)
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                unbalanced_values = vecvecdif(&unbalanced_values, &totproduced)
                    .iter()
                    .map(|&v| if v > 0.0 { v } else { 0.0 })
                    .collect();
            }
            if unbalanced_values.iter().sum::<f32>() == 0.0 {
                return None;
            }; // Ya está equilibrado

            Some(Component {
                carrier: Carrier::MEDIOAMBIENTE,
                ctype: CType::PRODUCCION,
                csubtype: CSubtype::INSITU,
                service,
                values: unbalanced_values,
                comment:
                    "Equilibrado de energía térmica insitu consumida y sin producción declarada"
                        .into(),
            })
        })
        .filter(|v| v.is_some())
        .collect::<Option<Vec<_>>>()
        .unwrap_or_else(|| vec![]);
    components.cdata.append(&mut balancecomps);
}

/// Devuelve objetos CARRIER y META a partir de cadena, intentando asegurar los tipos.
pub fn parse_components(datastring: &str) -> Result<Components, Error> {
    let mut components: Components = datastring.parse()?;
    fix_components(&mut components);
    Ok(components)
}

// // ---------------------- Factores de paso -----------------------------------------------

/// Asegura consistencia de factores de paso definidos y deduce algunos de los que falten.
///
/// También elimina los destinados a exportación to_nEPB por defecto (pueden dejarse con opción a false)
pub fn fix_wfactors(
    mut wfactors: Factors,
    cogen: Option<RenNren>,
    cogennepb: Option<RenNren>,
    red1: Option<RenNren>,
    red2: Option<RenNren>,
    stripnepb: bool,
) -> Result<Factors, Error> {
    // Usa valores por defecto si no se definen los valores
    let cogen = cogen.unwrap_or(CTE_COGEN_DEFAULTS_TO_GRID);
    let cogennepb = cogennepb.unwrap_or(CTE_COGEN_DEFAULTS_TO_NEPB);
    let red1 = red1.unwrap_or(CTE_RED_DEFAULTS_RED1);
    let red2 = red2.unwrap_or(CTE_RED_DEFAULTS_RED2);

    // Actualiza metadatos con valores bien conocidos
    //let mut wfactors = wfactors;
    wfactors.update_meta("CTE_COGEN", &format!("{:.3}, {:.3}", cogen.ren, cogen.nren));
    wfactors.update_meta(
        "CTE_COGENNEPB",
        &format!("{:.3}, {:.3}", cogennepb.ren, cogennepb.nren),
    );
    wfactors.update_meta("CTE_RED1", &format!("{:.3}, {:.3}", red1.ren, red1.nren));
    wfactors.update_meta("CTE_RED2", &format!("{:.3}, {:.3}", red2.ren, red2.nren));

    // Vectores existentes
    let wf_carriers: Vec<_> = wfactors.wdata.iter().map(|f| f.carrier).unique().collect();

    // Asegura que existe MEDIOAMBIENTE, INSITU, input, A, 1.0, 0.0
    let has_ma_insitu_input_a = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::MEDIOAMBIENTE && f.source == Source::INSITU && f.dest == Dest::input
            && f.step == Step::A
    });
    if !has_ma_insitu_input_a {
        wfactors.wdata.push(Factor::new(
            Carrier::MEDIOAMBIENTE,
            Source::INSITU,
            Dest::input,
            Step::A,
            1.0,
            0.0,
            "Recursos usados para obtener energía térmica del medioambiente".to_string(),
        ));
    }
    // Asegura que existe MEDIOAMBIENTE, RED, input, A, 1.0, 0.0
    let has_ma_red_input_a = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::MEDIOAMBIENTE && f.source == Source::RED && f.dest == Dest::input
            && f.step == Step::A
    });
    if !has_ma_red_input_a {
        // MEDIOAMBIENTE, RED, input, A, ren, nren === MEDIOAMBIENTE, INSITU, input, A, ren, nren
        wfactors.wdata.push(Factor::new(
            Carrier::MEDIOAMBIENTE,
            Source::RED,
            Dest::input,
            Step::A,
            1.0,
            0.0,
            "Recursos usados para obtener energía térmica del medioambiente (red ficticia)"
                .to_string(),
        ));
    }
    // Asegura que existe ELECTRICIDAD, INSITU, input, A, 1.0, 0.0 si hay ELECTRICIDAD
    let has_elec_and_elec_insitu_input_a = wf_carriers.contains(&Carrier::ELECTRICIDAD)
        && !wfactors.wdata.iter().any(|f| {
            f.carrier == Carrier::ELECTRICIDAD && f.source == Source::INSITU
                && f.dest == Dest::input
        });
    if has_elec_and_elec_insitu_input_a {
        wfactors.wdata.push(Factor::new(
            Carrier::ELECTRICIDAD,
            Source::INSITU,
            Dest::input,
            Step::A,
            1.0,
            0.0,
            "Recursos usados para generar electricidad in situ".to_string(),
        ));
    }
    // Asegura definición de factores de red para todos los vectores energéticos
    let has_grid_factors_for_all_carriers = wf_carriers.iter().all(|&c| {
        wfactors.wdata.iter().any(|f| {
            f.carrier == c && f.source == Source::RED && f.dest == Dest::input && f.step == Step::A
        })
    });
    if !has_grid_factors_for_all_carriers {
        bail!("No se han definido los factores de paso de red de algún vector \"VECTOR, INSITU, input, A, fren?, fnren?\"");
    }
    // En paso A, el factor input de cogeneración es 0.0, 0.0 ya que el impacto se tiene en cuenta en el suministro del vector de generación
    let has_cogen_input = wfactors
        .wdata
        .iter()
        .any(|f| f.source == Source::COGENERACION && f.dest == Dest::input);
    if !has_cogen_input {
        wfactors.wdata.push(Factor::new(
            Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::input, Step::A, 0.0, 0.0,
            "Factor de paso generado (el impacto de la cogeneración se tiene en cuenta en el vector de suministro)".to_string()));
    }
    // Asegura que todos los vectores con exportación tienen factores de paso a la red y a usos no EPB
    let exp_carriers = [
        (Carrier::ELECTRICIDAD, Source::INSITU),
        (Carrier::ELECTRICIDAD, Source::COGENERACION),
        (Carrier::MEDIOAMBIENTE, Source::INSITU),
    ];
    for (c, s) in &exp_carriers {
        // Asegura que existe VECTOR, SRC, to_grid | to_nEPB, A, ren, nren
        let fp_a_input = wfactors
            .wdata
            .iter()
            .find(|f| {
                f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::input
            })
            .and_then(|f| Some(f.clone()));

        let has_to_grid = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::to_grid
        });
        if !has_to_grid {
            if *s != Source::COGENERACION {
                // VECTOR, SRC, to_grid, A, ren, nren === VECTOR, SRC, input, A, ren, nren
                if fp_a_input.is_some() {
                    let f = fp_a_input.as_ref().unwrap();
                    wfactors.wdata.push(Factor {
                        dest: Dest::to_grid,
                        step: Step::A,
                        comment: "Recursos usados para producir la energía exportada a la red"
                            .to_string(),
                        ..*f
                    });
                } else {
                    bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a la red en paso A", c);
                }
            } else {
                #[cfg_attr(clippy, allow(float_cmp))]
                // Valores por defecto para ELECTRICIDAD, COGENERACION, to_grid, A, ren, nren - ver 9.6.6.2.3
                let value_origin = if ((cogen.ren - CTE_COGEN_DEFAULTS_TO_GRID.ren).abs() < EPSILON)
                    && ((cogen.nren - CTE_COGEN_DEFAULTS_TO_GRID.nren).abs() < EPSILON)
                {
                    "(Valor predefinido)"
                } else {
                    "(Valor de usuario)"
                };
                wfactors.wdata.push(Factor::new(
                    Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::to_grid, Step::A, cogen.ren, cogen.nren,
                    format!("Recursos usados para producir la electricidad cogenerada y exportada a la red (ver EN ISO 52000-1 9.6.6.2.3) {}", value_origin)));
            }
        }
        let has_to_nepb = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::to_nEPB
        });
        if !has_to_nepb {
            if *s != Source::COGENERACION {
                // VECTOR, SRC, to_nEPB, A, ren, nren == VECTOR, SRC, input, A, ren, nren
                if fp_a_input.is_some() {
                    let f = fp_a_input.as_ref().unwrap();
                    wfactors.wdata.push(Factor {
                        dest: Dest::to_nEPB,
                        step: Step::A,
                        comment:
                            "Recursos usados para producir la energía exportada a usos no EPB"
                                .to_string(),
                        ..*f
                    });
                } else {
                    bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a usos no EPB en paso A", c);
                }
            } else {
                // TODO: Si está definido para to_grid (no por defecto) y no para to_nEPB, qué hacemos? usamos por defecto? usamos igual a to_grid?
                // Valores por defecto para ELECTRICIDAD, COGENERACION, to_nEPB, A, ren, nren - ver 9.6.6.2.3
                let value_origin = if ((cogennepb.ren - CTE_COGEN_DEFAULTS_TO_NEPB.ren).abs()
                    < EPSILON)
                    && ((cogennepb.nren - CTE_COGEN_DEFAULTS_TO_NEPB.nren).abs() < EPSILON)
                {
                    "(Valor predefinido)"
                } else {
                    "(Valor de usuario)"
                };
                wfactors.wdata.push(Factor::new(Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::to_nEPB, Step::A, cogennepb.ren, cogennepb.nren,
                    format!("Recursos usados para producir la electricidad cogenerada y exportada a usos no EPB (ver EN ISO 52000-1 9.6.6.2.3) {}", value_origin)
                    ));
            }
        }
        // Asegura que existe VECTOR, SRC, to_grid | to_nEPB, B, ren, nren
        let fp_a_red_input = wfactors
            .wdata
            .iter()
            .find(|f| {
                f.carrier == *c && f.source == Source::RED && f.dest == Dest::input
                    && f.step == Step::A
            })
            .and_then(|f| Some(f.clone()));
        let has_to_grid_b = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::to_grid
        });
        if !has_to_grid_b {
            // VECTOR, SRC, to_grid, B, ren, nren == VECTOR, RED, input, A, ren, nren
            if fp_a_red_input.is_some() {
                let f = fp_a_red_input.as_ref().unwrap();
                wfactors.wdata.push(Factor::new(f.carrier, *s, Dest::to_grid, Step::B, f.ren, f.nren,
                "Recursos ahorrados a la red por la energía producida in situ y exportada a la red".to_string()));
            } else {
                bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a la red en paso B", c);
            }
        }
        let has_to_nepb_b = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::to_nEPB
        });
        if !has_to_nepb_b {
            // VECTOR, SRC, to_nEPB, B, ren, nren == VECTOR, RED, input, A, ren, nren
            if fp_a_red_input.is_some() {
                let f = fp_a_red_input.as_ref().unwrap();
                wfactors.wdata.push(Factor::new(f.carrier, *s, Dest::to_nEPB, Step::B, f.ren, f.nren,
                "Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB".to_string()));
            } else {
                bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a usos no EPB en paso B", c);
            }
        }
    }
    // Asegura que existe RED1 | RED2, RED, input, A, ren, nren
    let has_red1_red_input = wfactors
        .wdata
        .iter()
        .any(|f| f.carrier == Carrier::RED1 && f.source == Source::RED && f.dest == Dest::input);
    if !has_red1_red_input {
        wfactors.wdata.push(Factor::new(Carrier::RED1, Source::RED, Dest::input, Step::A,
          red1.ren, red1.nren, "Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)".to_string()));
    }
    let has_red2_red_input = wfactors
        .wdata
        .iter()
        .any(|f| f.carrier == Carrier::RED2 && f.source == Source::RED && f.dest == Dest::input);
    if !has_red2_red_input {
        wfactors.wdata.push(Factor::new(Carrier::RED2, Source::RED, Dest::input, Step::A,
          red2.ren, red2.nren, "Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)".to_string()));
    }

    // Elimina destino nEPB si stripnepb es true
    if stripnepb {
        wfactors.wdata.retain(|e| e.dest != Dest::to_nEPB);
    }

    Ok(wfactors)
}

/// Lee factores de paso desde cadena y sanea los resultados.
pub fn parse_wfactors(
    wfactorsstring: &str,
    cogen: Option<RenNren>,
    cogennepb: Option<RenNren>,
    red1: Option<RenNren>,
    red2: Option<RenNren>,
    stripnepb: bool,
) -> Result<Factors, Error> {
    let wfactors: Factors = wfactorsstring.parse()?;
    fix_wfactors(wfactors, cogen, cogennepb, red1, red2, stripnepb)
}

/// Genera factores de paso a partir de localización.
///
/// Usa localización (PENINSULA, CANARIAS, BALEARES, CEUTAYMELILLA),
/// factores de paso de cogeneración, y factores de paso para RED1 y RED2
pub fn new_wfactors(
    loc: &str,
    cogen: Option<RenNren>,
    cogennepb: Option<RenNren>,
    red1: Option<RenNren>,
    red2: Option<RenNren>,
    stripnepb: bool,
) -> Result<Factors, Error> {
    // XXX: usar tipos en lugar de cadenas de texto
    let wfactorsstring = match &*loc {
        "PENINSULA" => CTE_FP_PENINSULA,
        "BALEARES" => CTE_FP_BALEARES,
        "CANARIAS" => CTE_FP_CANARIAS,
        "CEUTAMELILLA" => CTE_FP_CEUTAMELILLA,
        _ => bail!(
            "Localización \"{}\" desconocida al generar factores de paso",
            loc
        ),
    };
    let wfactors: Factors = wfactorsstring.parse()?;
    fix_wfactors(wfactors, cogen, cogennepb, red1, red2, stripnepb)
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
        .retain(|f| f.dest != Dest::to_nEPB || has_nepb);
    wfactors.wdata.retain(|f| {
        f.carrier != Carrier::ELECTRICIDAD || f.source != Source::INSITU || has_elec_insitu
    });
}

// Funcionalidad para generar RER para ACS en perímetro nearby -------------------------

/// Selecciona subconjunto de componentes relacionados con el servicio indicado.
#[allow(non_snake_case)]
pub fn components_by_service(components: &Components, service: Service) -> Components {
    // 1. Toma todos los consumos y producciones imputadas al servicio (p.e. ACS)
    // Nota: los consumos de MEDIOAMBIENTE de un servicio ya están equilibrados
    // Nota: por producciones asignadas a ese servicio (en parse_components)
    let mut cdata: Vec<_> = components
        .cdata
        .iter()
        .filter(|c| c.service == service)
        .cloned()
        .collect();

    // 2. Reparte la producción de electricidad INSITU asignada a NDEF
    // en la misma proporción del consumo de elec. del servicio en relación al del total de servicios
    let pr_el_ndef: Vec<_> = components
        .cdata
        .iter()
        .filter(|c| {
            c.carrier == Carrier::ELECTRICIDAD && c.ctype == CType::PRODUCCION
                && c.csubtype == CSubtype::INSITU && c.service == Service::NDEF
        })
        .collect();
    if !pr_el_ndef.is_empty() {
        // Hay producción de electricidad in situ de NDEF (no asignada a un servicio)
        let c_el = components
            .cdata
            .iter()
            .filter(|c| c.carrier == Carrier::ELECTRICIDAD && c.ctype == CType::CONSUMO);
        let c_el_tot = c_el.clone()
            .map(|c| c.values.iter().sum::<f32>())
            .sum::<f32>();
        let c_el_srv_tot = c_el.clone()
            .filter(|c| c.service == service)
            .map(|c| c.values.iter().sum::<f32>())
            .sum::<f32>();
        let F_pr_srv = if c_el_tot > 0.0 {
            c_el_srv_tot / c_el_tot
        } else {
            0.0
        };
        for c in &pr_el_ndef {
            cdata.push(Component {
                carrier: Carrier::ELECTRICIDAD,
                ctype: CType::PRODUCCION,
                csubtype: CSubtype::INSITU,
                service,
                values: veckmul(&c.values, F_pr_srv),
                comment: format!(
                    "{} Producción insitu proporcionalmente reasignada al servicio.",
                    c.comment
                ),
            })
        }
    }

    let cmeta = components.cmeta.clone();

    let mut newcomponents = Components { cdata, cmeta };

    newcomponents.update_meta("CTE_PERIMETRO", "NEARBY");
    newcomponents.update_meta("CTE_SERVICIO", &service.to_string());

    newcomponents
}

/// Vectores considerados dentro del perímetro NEARBY (a excepción de la ELECTRICIDAD in situ).
pub const CTE_NRBY: [Carrier; 5] = [
    Carrier::BIOMASA,
    Carrier::BIOMASADENSIFICADA,
    Carrier::RED1,
    Carrier::RED2,
    Carrier::MEDIOAMBIENTE,
]; // Ver B.23. Solo biomasa sólida

/// Convierte factores de paso con perímetro "distant" a factores de paso "nearby".
pub fn wfactors_to_nearby(wfactors: &Factors) -> Factors {
    // Los elementos que tiene origen en la RED (!= INSITU, != COGENERACION)
    // y no están en la lista CTE_NRBY cambian sus factores de paso
    // de forma que ren' = 0 y nren' = ren + nren.
    // ATENCIÓN: ¡¡La producción eléctrica de la cogeneración entra con (factores ren:0, nren:0)!!
    let mut wmeta = wfactors.wmeta.clone();
    let mut wdata: Vec<Factor> = Vec::new();

    for f in wfactors.wdata.iter().cloned() {
        if f.source == Source::INSITU || f.source == Source::COGENERACION
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
                format!("Perímetro nearby: {}", f.comment),
            ))
        }
    }
    wmeta.push(Meta {
        key: "CTE_PERIMETRO".to_string(),
        value: "NEARBY".to_string(),
    });
    Factors { wmeta, wdata }
}

// Métodos de salida -------------------------------------------------------------------

/// Muestra balance, paso B, de forma simplificada.
pub fn balance_to_plain(balance: &Balance) -> String {
    let Balance {
        k_exp,
        arearef,
        balance_m2,
        ..
    } = balance;
    let RenNren { ren, nren } = balance_m2.B;
    let tot = balance_m2.B.tot();
    let rer = balance_m2.B.rer();

    format!(
        "Area_ref = {:.2} [m2]
k_exp = {:.2}
C_ep [kWh/m2.an]: ren = {:.1}, nren = {:.1}, tot = {:.1}, RER = {:.2}",
        arearef, k_exp, ren, nren, tot, rer
    )
}

/// Sustituye símbolos reservados en XML.
pub fn escape_xml(unescaped: &str) -> String {
    unescaped
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "&apos;")
        .replace('"', "&quot;")
}

/// Muestra balance en formato XML.
pub fn balance_to_xml(balanceobj: &Balance) -> String {
    let Balance {
        components,
        wfactors,
        k_exp,
        arearef,
        balance_m2,
        ..
    } = balanceobj;
    let RenNren { ren, nren } = balance_m2.B;
    let cmeta = &components.cmeta;
    let cdata = &components.cdata;
    let wmeta = &wfactors.wmeta;
    let wdata = &wfactors.wdata;
    let wmetastring = wmeta
        .iter()
        .map(|m| {
            format!(
                "      <Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
                escape_xml(&m.key),
                escape_xml(&m.value)
            )
        })
        .join("\n");
    let wdatastring = wdata
        .iter()
        .map(|f| {
            let Factor {
                carrier,
                source,
                dest,
                step,
                ren,
                nren,
                comment,
            } = f;
            format!("      <Dato><Vector>{}</Vector><Origen>{}</Origen><Destino>{}</Destino><Paso>{}</Paso><ren>{:.3}</ren><nren>{:.3}</nren><Comentario>{}</Comentario></Dato>",
            carrier, source, dest, step, ren, nren, escape_xml(comment))
        })
        .join("\n");
    let cmetastring = cmeta
        .iter()
        .map(|m| {
            format!(
                "      <Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
                escape_xml(&m.key),
                escape_xml(&m.value)
            )
        })
        .join("\n");
    let cdatastring = cdata
        .iter()
        .map(|c| {
            let Component {
                carrier,
                ctype,
                csubtype,
                service,
                values,
                comment,
            } = c;
            let vals = values.iter().map(|v| format!("{:.2}", v)).join(",");
            format!(
                "      <Dato>
            <Vector>{}</Vector><Tipo>{}</Tipo><Subtipo>{}</Subtipo><Servicio>{}</Servicio>
            <Valores>{}</Valores>
            <Comentario>{}</Comentario>
        </Dato>",
                carrier,
                ctype,
                csubtype,
                service,
                vals,
                escape_xml(comment)
            )
        })
        .join("\n");

    format!(
        "<BalanceEPB>
    <FactoresDePaso>
        <Metadatos>
    {}
        </Metadatos>
        <Datos>
    {}
        </Datos>
    </FactoresDePaso>
    <Componentes>
        <Metadatos>
    {}
        </Metadatos>
        <Datos>
    {}
        </Datos>
    </Componentes>
    <kexp>{:.2}</kexp>
    <AreaRef>{:.2}</AreaRef><!-- área de referencia [m2] -->
    <Epm2><!-- ep [kWh/m2.a] -->
        <ren>{:.1}</ren>
        <nren>{:.1}</nren>
    </Epm2>
</BalanceEPB>",
        wmetastring, wdatastring, cmetastring, cdatastring, k_exp, arearef, ren, nren
    )
}
