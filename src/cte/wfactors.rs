// Copyright (c) 2018-2019  Ministerio de Fomento
//                          Instituto de Ciencias de la Construcción Eduardo Torroja (IETcc-CSIC)

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
//            Daniel Jiménez González <dani@ietcc.csic.es>,
//            Marta Sorribes Gil <msorribes@ietcc.csic.es>

/*!
Weighting factors (CTE)
=======================

Manejo de factores de paso para el CTE
--------------------------------------

Factores de paso y utilidades para la gestión de factores de paso para el CTE

*/

use crate::{
    error::{EpbdError, Result},
    fix_wfactors, set_user_wfactors,
    types::{Carrier, Factor, Meta, RenNrenCo2, Source},
    Factors, UserWF,
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

/// Valores por defecto para factores de paso
pub struct CteDefaultsWF {
    /// Factores de paso de usuario
    pub user: UserWF<RenNrenCo2>,
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
    user: UserWF {
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
    user: &UserWF<Option<RenNrenCo2>>,
    defaults: &CteDefaultsWF,
) -> Result<Factors> {
    let mut wfactors: Factors = wfactorsstring.parse()?;
    set_user_wfactors(&mut wfactors, user);
    fix_wfactors(wfactors, &defaults.user)
}

/// Genera factores de paso a partir de localización.
///
/// Usa localización (PENINSULA, CANARIAS, BALEARES, CEUTAMELILLA),
/// factores de paso de cogeneración, y factores de paso para RED1 y RED2
pub fn wfactors_from_loc(
    loc: &str,
    user: &UserWF<Option<RenNrenCo2>>,
    defaults: &CteDefaultsWF,
) -> Result<Factors> {
    let wfactorsstring = match &*loc {
        "PENINSULA" => defaults.loc_peninsula,
        "BALEARES" => defaults.loc_baleares,
        "CANARIAS" => defaults.loc_canarias,
        "CEUTAMELILLA" => defaults.loc_ceutamelilla,
        _ => return Err(EpbdError::Location(loc.to_string())),
    };
    let mut wfactors: Factors = wfactorsstring.parse()?;
    set_user_wfactors(&mut wfactors, user);
    fix_wfactors(wfactors, &defaults.user)
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
