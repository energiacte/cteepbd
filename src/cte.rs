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
Utilidades para el cumplimiento reglamentario (compliance utilities)
====================================================================

Utilidades para el manejo de balances energéticos para el CTE:

- valores reglamentarios
- generación y transformación de factores de paso
    - wfactors_from_str
    - wfactors_from_loc
    - wfactors_to_nearby
- salida/visualización de balances
    - balance_to_plain
    - balance_to_XML
*/

use crate::{error::EpbdError, types::*, Balance, Factors, UserWF};

/**
Constantes y valores generales
*/

/// Valor por defecto del área de referencia.
pub const AREAREF_DEFAULT: f32 = 1.0;
/// Valor predefinido del factor de exportación. Valor reglamentario.
pub const KEXP_DEFAULT: f32 = 0.0;
/// Localizaciones válidas para CTE
pub const CTE_LOCS: [&str; 4] = ["PENINSULA", "BALEARES", "CANARIAS", "CEUTAMELILLA"];

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

/// Valores por defecto de los factores de paso para cada localización
pub struct CteLocWF {
    /// Factores de paso reglamentarios para la Península
    pub loc_peninsula: &'static str,
    /// Factores de paso reglamentarios para Baleares.
    pub loc_baleares: &'static str,
    /// Factores de paso reglamentarios para Canarias.
    pub loc_canarias: &'static str,
    /// Factores de paso reglamentarios para Ceuta y Melilla.
    pub loc_ceutamelilla: &'static str,
}

/// Macro para la definición de los factores de paso de una localización
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

/// Factores de paso definibles por el usuario usados por defecto
pub const CTE_USERWF: UserWF<RenNrenCo2> = UserWF {
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
};

/// Factores de paso reglamentarios según el documento reconocido del RITE (20/07/2014)
/// 
/// Estos factores son los usados en:
/// - DB-HE 2013
/// - DB-HE 2018
pub const CTE_LOCWF_RITE2014: CteLocWF = CteLocWF {
    loc_peninsula: build_wf_2013!("PENINSULA", 0.414, 1.954, 0.331),
    loc_baleares: build_wf_2013!("BALEARES", 0.082, 2.968, 0.932),
    loc_canarias: build_wf_2013!("CANARIAS", 0.070, 2.924, 0.776),
    loc_ceutamelilla: build_wf_2013!("CEUTAMELILLA", 0.072, 2.718, 0.721),
};

/**
Manejo de factores de paso para el CTE
--------------------------------------

Factores de paso y utilidades para la gestión de factores de paso para el CTE
*/

/// Lee factores de paso desde cadena y sanea los resultados.
pub fn wfactors_from_str(
    wfactorsstring: &str,
    user: &UserWF<Option<RenNrenCo2>>,
    userdefaults: &UserWF<RenNrenCo2>,
) -> Result<Factors, EpbdError> {
    wfactorsstring
        .parse::<Factors>()?
        .set_user_wfactors(user)
        .normalize(&userdefaults)
}

/// Genera factores de paso a partir de localización.
///
/// Usa localización (PENINSULA, CANARIAS, BALEARES, CEUTAMELILLA),
/// factores de paso de cogeneración, y factores de paso para RED1 y RED2
pub fn wfactors_from_loc(
    loc: &str,
    locdefaults: &CteLocWF,
    user: &UserWF<Option<RenNrenCo2>>,
    userdefaults: &UserWF<RenNrenCo2>,
) -> Result<Factors, EpbdError> {
    let wfactorsstring = match &*loc {
        "PENINSULA" => locdefaults.loc_peninsula,
        "BALEARES" => locdefaults.loc_baleares,
        "CANARIAS" => locdefaults.loc_canarias,
        "CEUTAMELILLA" => locdefaults.loc_ceutamelilla,
        _ => return Err(EpbdError::ParseError(format!("Localizacion: {}", loc))),
    };
    wfactorsstring
        .parse::<Factors>()?
        .set_user_wfactors(user)
        .normalize(&userdefaults)
}

/// Convierte factores de paso con perímetro "distant" a factores de paso "nearby".
/// 
/// Los elementos que tiene origen en la RED (!= INSITU, != COGENERACION)
/// y no están en la lista CTE_NRBY cambian sus factores de paso
/// de forma que ren' = 0 y nren' = ren + nren.
/// **ATENCIÓN**: ¡¡La producción eléctrica de la cogeneración entra con (factores ren:0, nren:0)!!
pub fn wfactors_to_nearby(wfactors: &Factors) -> Factors {
    let wmeta = wfactors.wmeta.clone();
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
                RenNrenCo2::new(0.0, f.ren + f.nren, f.co2), // ¿Esto es lo que tiene más sentido?
                format!("Perímetro nearby: {}", f.comment),
            ))
        }
    }
    let mut factors = Factors { wmeta, wdata };
    factors.set_meta("CTE_PERIMETRO", "NEARBY");
    factors
}

/*
Utilidades para visualización del balance
-----------------------------------------
*/

/// Muestra el balance (paso B) en formato de texto simple.
pub fn balance_to_plain(balance: &Balance) -> String {
    let Balance {
        k_exp,
        arearef,
        balance_m2,
        ..
    } = balance;

    let RenNrenCo2 { ren, nren, co2 } = balance_m2.B;
    let tot = balance_m2.B.tot();
    let rer = balance_m2.B.rer();

    // Final
    let mut use_byuse = balance_m2
        .used_EPB_byuse
        .iter()
        .map(|(k, v)| format!("{}: {:.2}", k, v))
        .collect::<Vec<String>>();
    use_byuse.sort();

    // Ponderada por m2 (por uso)
    let mut b_byuse = balance_m2
        .B_byuse
        .iter()
        .map(|(k, v)| {
            format!(
                "{}: ren {:.2}, nren {:.2}, co2: {:.2}",
                k, v.ren, v.nren, v.co2
            )
        })
        .collect::<Vec<String>>();
    b_byuse.sort();

    format!(
        "Area_ref = {:.2} [m2]
k_exp = {:.2}
C_ep [kWh/m2.an]: ren = {:.1}, nren = {:.1}, tot = {:.1}, RER = {:.2}
E_CO2 [kg_CO2e/m2.an]: {:.2}

** Energía final (todos los vectores) [kWh/m2.an]:
{}

** Energía primaria (ren, nren) [kWh/m2.an] y emisiones [kg_CO2e/m2.an] por servicios:
{}
",
        arearef,
        k_exp,
        ren,
        nren,
        tot,
        rer,
        co2,
        use_byuse.join("\n"),
        b_byuse.join("\n")
    )
}

/// Muestra el balance (paso B) en formato XML
/// 
/// Esta función usa un formato compatible con el formato XML del certificado de eficiencia
/// energética del edificio definido en el documento de apoyo de la certificación energética
/// correspondiente.
pub fn balance_to_xml(balanceobj: &Balance) -> String {
    let Balance {
        components,
        wfactors,
        k_exp,
        arearef,
        balance_m2,
        ..
    } = balanceobj;

    // Data
    let RenNrenCo2 { ren, nren, .. } = balance_m2.B;
    let cmeta = &components.cmeta;
    let cdata = &components.cdata;
    let wmeta = &wfactors.wmeta;
    let wdata = &wfactors.wdata;

    /// Helper function -> XML escape symbols
    fn escape_xml(unescaped: &str) -> String {
        unescaped
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\\', "&apos;")
            .replace('"', "&quot;")
    }

    // Formatting
    let wmetastring = wmeta
        .iter()
        .map(|m| {
            format!(
                "      <Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
                escape_xml(&m.key),
                escape_xml(&m.value)
            )
        })
        .collect::<Vec<String>>()
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
                co2,
                comment,
            } = f;
            format!("      <Dato><Vector>{}</Vector><Origen>{}</Origen><Destino>{}</Destino><Paso>{}</Paso><ren>{:.3}</ren><nren>{:.3}</nren><CO2>{:.3}</CO2><Comentario>{}</Comentario></Dato>",
            carrier, source, dest, step, ren, nren, co2, escape_xml(comment))
        })
        .collect::<Vec<String>>()
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
        .collect::<Vec<String>>()
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
            let vals = values
                .iter()
                .map(|v| format!("{:.2}", v))
                .collect::<Vec<String>>()
                .join(",");
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
        .collect::<Vec<String>>()
        .join("\n");

    // Final assembly
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
    <Epm2><!-- C_ep [kWh/m2.an] -->
        <tot>{:.1}</tot>
        <nren>{:.1}</nren>
    </Epm2>
</BalanceEPB>",
        wmetastring,
        wdatastring,
        cmetastring,
        cdatastring,
        k_exp,
        arearef,
        ren + nren,
        nren
    )
}
