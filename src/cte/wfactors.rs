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

use failure::Error;
use itertools::Itertools;
use std::f32::EPSILON;

use crate::rennren::RenNren;
use crate::types::{CSubtype, Carrier, Dest, Source, Step};
use crate::types::{Components, Factor, Factors, Meta, MetaVec};

// Localizaciones válidas para CTE
// const CTE_LOCS: [&str; 4] = ["PENINSULA", "BALEARES", "CANARIAS", "CEUTAMELILLA"];

// Valores bien conocidos de metadatos:
// CTE_AREAREF -> num
// CTE_KEXP -> num
// CTE_LOCALIZACION -> str
// CTE_FACTORES_TIPO: FINAL_A_PRIMARIA | FINAL_A_CO2
// CTE_COGEN -> num, num
// CTE_RED1 -> num, num
// CTE_RED2 -> num, num

/// Vectores considerados dentro del perímetro NEARBY (a excepción de la ELECTRICIDAD in situ).
pub const CTE_NRBY: [Carrier; 5] = [
    Carrier::BIOMASA,
    Carrier::BIOMASADENSIFICADA,
    Carrier::RED1,
    Carrier::RED2,
    Carrier::MEDIOAMBIENTE,
]; // Ver B.23. Solo biomasa sólida

/// Tipo de factores de conversión de energía final empleados:
/// - a energía primaria
/// - a emisiones de CO2
#[derive(Clone, Copy, Debug, Display, PartialEq)]
pub enum WFactorsMode {
    /// Cálculo de energía primaria
    EP,
    /// Cálculo de emisiones de CO2
    CO2,
}

impl WFactorsMode {
    pub fn as_meta_value(&self) -> &'static str {
        match self {
            WFactorsMode::CO2 => "FINAL_A_EMISIONES",
            _ => "FINAL_A_PRIMARIA",
        }
    }
}

// ---------------- Valores por defecto y definibles por el usuario -----------------------
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
    /// Tipo de factores de conversión usados (p.e., a energía primaria o a emisiones)
    pub mode: WFactorsMode,
    pub user: CteUserWF<RenNren>,
    pub loc_peninsula: &'static str,
    /// Factores de paso reglamentarios (RITE 20/07/2014) para Baleares.
    pub loc_baleares: &'static str,
    /// Factores de paso reglamentarios (RITE 20/07/2014) para Canarias.
    pub loc_canarias: &'static str,
    /// Factores de paso reglamentarios (RITE 20/07/2014) para Ceuta y Melilla.
    pub loc_ceutamelilla: &'static str,
}

/// Valores por defecto para energía primaria
pub const CTE_DEFAULTS_WF_EP: CteDefaultsWF = CteDefaultsWF {
    mode: WFactorsMode::EP,
    user: CteUserWF {
        red1: RenNren {
            ren: 0.0,
            nren: 1.3,
        },
        red2: RenNren {
            ren: 0.0,
            nren: 1.3,
        },
        cogen_to_grid: RenNren {
            ren: 0.0,
            nren: 2.5,
        },
        cogen_to_nepb: RenNren {
            ren: 0.0,
            nren: 2.5,
        }
    },
    loc_peninsula: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: PENINSULA
#META CTE_FACTORES_TIPO: FINAL_A_PRIMARIA
#META CTE_FUENTE_COMENTARIO: Factores de paso (kWh/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954 # Recursos usados para suministrar electricidad (PENINSULA) desde la red
",
    loc_baleares: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: BALEARES
#META CTE_FACTORES_TIPO: FINAL_A_PRIMARIA
#META CTE_FUENTE_COMENTARIO: Factores de paso (kWh/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.082, 2.968 # Recursos usados para suministrar electricidad (BALEARES) desde la red
",
    loc_canarias: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CANARIAS
#META CTE_FACTORES_TIPO: FINAL_A_PRIMARIA
#META CTE_FUENTE_COMENTARIO: Factores de paso (kWh/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.070, 2.924 # Recursos usados para suministrar electricidad (CANARIAS) desde la red
",
    loc_ceutamelilla: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CEUTAMELILLA
#META CTE_FACTORES_TIPO: FINAL_A_PRIMARIA
#META CTE_FUENTE_COMENTARIO: Factores de paso (kWh/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.072, 2.718 # Recursos usados para suministrar electricidad (CEUTA Y MELILLA) desde la red
",
};

/// Valores por defecto para emisiones de CO2. El valor final se lee en la fracción no renovable
pub const CTE_DEFAULTS_WF_CO2: CteDefaultsWF = CteDefaultsWF {
    mode: WFactorsMode::CO2,
    user: CteUserWF {
        red1: RenNren {
            ren: 0.0,
            nren: 0.300,
        },
        red2: RenNren {
            ren: 0.0,
            nren: 0.300,
        },
        cogen_to_grid: RenNren {
            ren: 0.0,
            nren: 0.300,
        },
        cogen_to_nepb: RenNren {
            ren: 0.0,
            nren: 0.300,
        }
    },
    loc_peninsula: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: PENINSULA
#META CTE_FACTORES_TIPO: FINAL_A_CO2
#META CTE_FUENTE_COMENTARIO: Factores de paso de emisiones (kg_CO2/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.0, 0.472 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.0, 0.252 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.0, 0.311 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.0, 0.254 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.0, 0.331 # Recursos usados para suministrar electricidad (PENINSULA) desde la red
",
    loc_baleares: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: BALEARES
#META CTE_FACTORES_TIPO: FINAL_A_CO2
#META CTE_FUENTE_COMENTARIO: Factores de paso de emisiones (kg_CO2/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.0, 0.472 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.0, 0.252 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.0, 0.311 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.0, 0.254 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.0, 0.932 # Recursos usados para suministrar electricidad (BALEARES) desde la red
",
    loc_canarias: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CANARIAS
#META CTE_FACTORES_TIPO: FINAL_A_CO2
#META CTE_FUENTE_COMENTARIO: Factores de paso de emisiones (kg_CO2/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.0, 0.472 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.0, 0.252 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.0, 0.311 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.0, 0.254 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.0, 0.776 # Recursos usados para suministrar electricidad (CANARIAS) desde la red
",
    loc_ceutamelilla: "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CEUTAMELILLA
#META CTE_FACTORES_TIPO: FINAL_A_CO2
#META CTE_FUENTE_COMENTARIO: Factores de paso de emisiones (kg_CO2/kWh_f) del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 0.0, 0.018 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.0, 0.472 # Recursos usados para suministrar el vector desde la red
GASNATURAL, RED, SUMINISTRO, A, 0.0, 0.252 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.0, 0.311 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.0, 0.254 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.0, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.0, 0.721 # Recursos usados para suministrar electricidad (CEUTA Y MELILLA) desde la red
",
};

// --------------------- Utilidades E/S ------------------------

/// Lee factores de paso desde cadena y sanea los resultados.
pub fn parse_wfactors(
    wfactorsstring: &str,
    user: &CteUserWF<Option<RenNren>>,
    defaults: &CteDefaultsWF,
    stripnepb: bool,
) -> Result<Factors, Error> {
    let mut wfactors: Factors = wfactorsstring.parse()?;

    if wfactors.has_meta_value("CTE_FACTORES_TIPO", WFactorsMode::CO2.as_meta_value())
        && defaults.mode != WFactorsMode::CO2
    {
        bail!("Factores de emisión incoherentes para el cálculo de emisiones");
    }
    set_user_wfactors_and_mode(
        &mut wfactors,
        user,
        defaults,
    );
    fix_wfactors(wfactors, stripnepb)
}

/// Genera factores de paso a partir de localización.
///
/// Usa localización (PENINSULA, CANARIAS, BALEARES, CEUTAMELILLA),
/// factores de paso de cogeneración, y factores de paso para RED1 y RED2
pub fn new_wfactors(
    loc: &str,
    user: &CteUserWF<Option<RenNren>>,
    defaults: &CteDefaultsWF,
    stripnepb: bool,
) -> Result<Factors, Error> {
    // XXX: usar tipos en lugar de cadenas de texto
    let wfactorsstring = match &*loc {
        "PENINSULA" => defaults.loc_peninsula,
        "BALEARES" => defaults.loc_baleares,
        "CANARIAS" => defaults.loc_canarias,
        "CEUTAMELILLA" => defaults.loc_ceutamelilla,
        _ => bail!(
            "Localización \"{}\" desconocida al generar factores de paso",
            loc
        ),
    };
    let mut wfactors: Factors = wfactorsstring.parse()?;
    set_user_wfactors_and_mode(
        &mut wfactors,
        user,
        defaults,
    );
    fix_wfactors(wfactors, stripnepb)
}

/// Actualiza factores definidos por el usuario en los metadatos (cogen_to_grid, cogen_to_nepb, red1 y red2)
///
/// Utiliza:
/// 1. el factor si está definido en los argumentos (es Some)
/// 2. el factor de wfactors en los metadatos
/// 3. el factor por defecto
///
pub fn set_user_wfactors_and_mode(
    wfactors: &mut Factors,
    user: &CteUserWF<Option<RenNren>>,
    defaults: &CteDefaultsWF,
) {
    let cogen_to_grid = user.cogen_to_grid
        .or_else(|| wfactors.get_meta_rennren("CTE_COGEN"))
        .or_else(|| {
            wfactors
                .wdata
                .iter()
                .find(|f| {
                    f.source == Source::COGENERACION && f.step == Step::A && f.dest == Dest::A_RED
                })
                .and_then(|f| Some(f.factors()))
        })
        .unwrap_or(defaults.user.cogen_to_grid);

    let cogen_to_nepb = user.cogen_to_nepb
        .or_else(|| wfactors.get_meta_rennren("CTE_COGENNEPB"))
        .or_else(|| {
            wfactors
                .wdata
                .iter()
                .find(|f| {
                    f.source == Source::COGENERACION && f.step == Step::A && f.dest == Dest::A_NEPB
                })
                .and_then(|f| Some(f.factors()))
        })
        .unwrap_or(defaults.user.cogen_to_nepb);

    let red1 = user.red1
        .or_else(|| wfactors.get_meta_rennren("CTE_RED1"))
        .or_else(|| {
            wfactors
                .wdata
                .iter()
                .find(|f| {
                    f.carrier == Carrier::RED1 && f.step == Step::A && f.dest == Dest::SUMINISTRO
                })
                .and_then(|f| Some(f.factors()))
        })
        .unwrap_or(defaults.user.red1);

    let red2 = user.red2
        .or_else(|| wfactors.get_meta_rennren("CTE_RED2"))
        .or_else(|| {
            wfactors
                .wdata
                .iter()
                .find(|f| {
                    f.carrier == Carrier::RED2 && f.step == Step::A && f.dest == Dest::SUMINISTRO
                })
                .and_then(|f| Some(f.factors()))
        })
        .unwrap_or(defaults.user.red2);

    // Actualiza factores de usuario en metadatos
    wfactors.update_meta(
        "CTE_COGEN",
        &format!("{:.3}, {:.3}", cogen_to_grid.ren, cogen_to_grid.nren),
    );
    wfactors.update_meta(
        "CTE_COGENNEPB",
        &format!("{:.3}, {:.3}", cogen_to_nepb.ren, cogen_to_nepb.nren),
    );
    wfactors.update_meta("CTE_RED1", &format!("{:.3}, {:.3}", red1.ren, red1.nren));
    wfactors.update_meta("CTE_RED2", &format!("{:.3}, {:.3}", red2.ren, red2.nren));

    // Actualiza modo de cálculo
    wfactors.update_meta("CTE_FACTORES_TIPO", defaults.mode.as_meta_value());
}

/// Asegura consistencia de factores de paso definidos y deduce algunos de los que falten.
///
/// Realiza los siguientes pasos:
/// - asegura definición de factores de producción in situ
/// - asegura definición de factores desde la red para todos los vectores
/// - asegura que factor paso A para suministro de cogeneración es 0.0 (se considera en vector original)
/// - asegura definición de factores a la red para vectores con exportación
/// - asegura que existe RED1 | RED2 en suministro
/// - elimina factores con destino nEPB si stripnepb es true
///
/// Los factores destinados a exportación A_NEPB se eliminan por defecto (pueden dejarse con opción a false)
///
/// TODO: se deberían separar algunos de estos pasos como métodos de Factors
pub fn fix_wfactors(mut wfactors: Factors, stripnepb: bool) -> Result<Factors, Error> {
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
            "Recursos usados para obtener energía térmica del medioambiente".to_string(),
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
            "Recursos usados para obtener energía térmica del medioambiente (red ficticia)"
                .to_string(),
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
            "Recursos usados para generar electricidad in situ".to_string(),
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
        bail!("No se han definido los factores de paso de red de algún vector \"VECTOR, INSITU, SUMINISTRO, A, fren?, fnren?\"");
    }
    // En paso A, el factor SUMINISTRO de cogeneración es 0.0, 0.0 ya que el impacto se tiene en cuenta en el suministro del vector de generación
    let has_cogen_input = wfactors
        .wdata
        .iter()
        .any(|f| f.source == Source::COGENERACION && f.dest == Dest::SUMINISTRO);
    if !has_cogen_input {
        wfactors.wdata.push(Factor::new(
            Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::SUMINISTRO, Step::A, 0.0, 0.0,
            "Factor de paso generado (el impacto de la cogeneración se tiene en cuenta en el vector de suministro)".to_string()));
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
                    bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a la red en paso A", c);
                }
            } else {
                // Valores por defecto para ELECTRICIDAD, COGENERACION, A_RED, A, ren, nren - ver 9.6.6.2.3
                let cogen = wfactors
                    .get_meta_rennren("CTE_COGEN")
                    .unwrap_or(CTE_DEFAULTS_WF_EP.user.cogen_to_grid);
                let value_origin = if ((cogen.ren - CTE_DEFAULTS_WF_EP.user.cogen_to_grid.ren)
                    .abs()
                    < EPSILON)
                    && ((cogen.nren - CTE_DEFAULTS_WF_EP.user.cogen_to_grid.nren).abs() < EPSILON)
                {
                    "(Valor predefinido)"
                } else {
                    "(Valor de usuario)"
                };
                wfactors.wdata.push(Factor::new(
                    Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::A_RED, Step::A, cogen.ren, cogen.nren,
                    format!("Recursos usados para producir la electricidad cogenerada y exportada a la red (ver EN ISO 52000-1 9.6.6.2.3) {}", value_origin)));
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
                    bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a usos no EPB en paso A", c);
                }
            } else {
                // TODO: Si está definido para A_RED (no por defecto) y no para A_NEPB, qué hacemos? usamos por defecto? usamos igual a A_RED?
                // Valores por defecto para ELECTRICIDAD, COGENERACION, A_NEPB, A, ren, nren - ver 9.6.6.2.3
                let cogennepb = wfactors
                    .get_meta_rennren("CTE_COGENNEPB")
                    .unwrap_or(CTE_DEFAULTS_WF_EP.user.cogen_to_nepb);
                let value_origin = if ((cogennepb.ren - CTE_DEFAULTS_WF_EP.user.cogen_to_nepb.ren)
                    .abs()
                    < EPSILON)
                    && ((cogennepb.nren - CTE_DEFAULTS_WF_EP.user.cogen_to_nepb.nren).abs()
                        < EPSILON)
                {
                    "(Valor predefinido)"
                } else {
                    "(Valor de usuario)"
                };
                wfactors.wdata.push(Factor::new(Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::A_NEPB, Step::A, cogennepb.ren, cogennepb.nren,
                    format!("Recursos usados para producir la electricidad cogenerada y exportada a usos no EPB (ver EN ISO 52000-1 9.6.6.2.3) {}", value_origin)
                    ));
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
                wfactors.wdata.push(Factor::new(f.carrier, *s, Dest::A_RED, Step::B, f.ren, f.nren,
                "Recursos ahorrados a la red por la energía producida in situ y exportada a la red".to_string()));
            } else {
                bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a la red en paso B", c);
            }
        }
        let has_to_nepb_b = wfactors.wdata.iter().any(|f| {
            f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::A_NEPB
        });
        if !has_to_nepb_b {
            // VECTOR, SRC, A_NEPB, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
            if fp_a_red_input.is_some() {
                let f = fp_a_red_input.as_ref().unwrap();
                wfactors.wdata.push(Factor::new(f.carrier, *s, Dest::A_NEPB, Step::B, f.ren, f.nren,
                "Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB".to_string()));
            } else {
                bail!("No se ha definido el factor de paso de suministro del vector {} y es necesario para definir el factor de exportación a usos no EPB en paso B", c);
            }
        }
    }
    // Asegura que existe RED1 | RED2, RED, SUMINISTRO, A, ren, nren
    let has_red1_red_input = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::RED1 && f.source == Source::RED && f.dest == Dest::SUMINISTRO
    });
    if !has_red1_red_input {
        let red1 = wfactors
            .get_meta_rennren("CTE_RED1")
            .unwrap_or(CTE_DEFAULTS_WF_EP.user.red1);
        wfactors.wdata.push(Factor::new(Carrier::RED1, Source::RED, Dest::SUMINISTRO, Step::A,
          red1.ren, red1.nren, "Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)".to_string()));
    }
    let has_red2_red_input = wfactors.wdata.iter().any(|f| {
        f.carrier == Carrier::RED2 && f.source == Source::RED && f.dest == Dest::SUMINISTRO
    });
    if !has_red2_red_input {
        let red2 = wfactors
            .get_meta_rennren("CTE_RED2")
            .unwrap_or(CTE_DEFAULTS_WF_EP.user.red2);
        wfactors.wdata.push(Factor::new(Carrier::RED2, Source::RED, Dest::SUMINISTRO, Step::A,
          red2.ren, red2.nren, "Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)".to_string()));
    }

    // Elimina destino nEPB si stripnepb es true
    if stripnepb {
        wfactors.wdata.retain(|e| e.dest != Dest::A_NEPB);
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
