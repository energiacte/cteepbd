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

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::{
    error::EpbdError,
    types::*,
    vecops::{vecvecmin, vecvecsum},
    Balance, Components, Factors, UserWF,
};

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

/// Factores de paso definibles por el usuario usados por defecto
pub const CTE_USERWF: UserWF<RenNrenCo2> = UserWF {
    red1: RenNrenCo2::new(0.0, 1.3, 0.3),
    red2: RenNrenCo2::new(0.0, 1.3, 0.3),
    cogen_to_grid: RenNrenCo2::new(0.0, 2.5, 0.3),
    cogen_to_nepb: RenNrenCo2::new(0.0, 2.5, 0.3),
};

/// Factores de paso reglamentarios según el documento reconocido del RITE (20/07/2014)
///
/// Estos factores son los usados en:
/// - DB-HE 2013
/// - DB-HE 2018
pub static CTE_LOCWF_RITE2014: Lazy<HashMap<&'static str, Factors>> = Lazy::new(|| {
    use Carrier::*;
    use Dest::*;
    use Source::*;
    use Step::A;
    let wf = Factors {
        wmeta: vec![
            Meta::new("CTE_FUENTE", "RITE2014"),
            // Meta::new("CTE_LOCALIZACION", loc),
            Meta::new("CTE_FUENTE_COMENTARIO", "Factores de paso (kWh/kWh_f,kWh/kWh_f,kg_CO2/kWh_f) del documento reconocido del RITE de 20/07/2014")
        ],
        wdata: vec![
            Factor::new(MEDIOAMBIENTE, RED, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)"),
            Factor::new(MEDIOAMBIENTE, INSITU, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)"),
            Factor::new(BIOCARBURANTE, RED, SUMINISTRO, A, (1.028, 0.085, 0.018).into(), "Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))"),
            Factor::new(BIOMASA, RED, SUMINISTRO, A, (1.003, 0.034, 0.018).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(BIOMASADENSIFICADA, RED, SUMINISTRO, A, (1.028, 0.085, 0.018).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(CARBON, RED, SUMINISTRO, A, (0.002, 1.082, 0.472).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(GASNATURAL, RED, SUMINISTRO, A, (0.005, 1.190, 0.252).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(GASOLEO, RED, SUMINISTRO, A, (0.003, 1.179, 0.311).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(GLP, RED, SUMINISTRO, A, (0.003, 1.201, 0.254).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(ELECTRICIDAD, INSITU, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para producir electricidad in situ"),
            Factor::new(ELECTRICIDAD, COGENERACION, SUMINISTRO, A, (0.000, 0.000, 0.000).into(), "Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)"),
            // Factor::new(ELECTRICIDAD, RED, SUMINISTRO, A, (ren, nren, co2), "Recursos usados para el suministro desde la red")
        ]};
    let mut wfpen = wf.clone();
    wfpen.set_meta("CTE_LOCALIZACION", "PENINSULA");
    wfpen.wdata.push(Factor::new(
        ELECTRICIDAD,
        RED,
        SUMINISTRO,
        A,
        (0.414, 1.954, 0.331).into(),
        "Recursos usados para el suministro desde la red",
    ));

    let mut wfbal = wf.clone();
    wfbal.set_meta("CTE_LOCALIZACION", "BALEARES");
    wfbal.wdata.push(Factor::new(
        ELECTRICIDAD,
        RED,
        SUMINISTRO,
        A,
        (0.082, 2.968, 0.932).into(),
        "Recursos usados para el suministro desde la red",
    ));

    let mut wfcan = wf.clone();
    wfcan.set_meta("CTE_LOCALIZACION", "CANARIAS");
    wfcan.wdata.push(Factor::new(
        ELECTRICIDAD,
        RED,
        SUMINISTRO,
        A,
        (0.070, 2.924, 0.776).into(),
        "Recursos usados para el suministro desde la red",
    ));

    let mut wfcym = wf;
    wfcym.set_meta("CTE_LOCALIZACION", "CEUTAMELILLA");
    #[allow(clippy::approx_constant)]
    wfcym.wdata.push(Factor::new(
        ELECTRICIDAD,
        RED,
        SUMINISTRO,
        A,
        (0.072, 2.718, 0.721).into(),
        "Recursos usados para el suministro desde la red",
    ));

    let mut m = HashMap::new();
    m.insert("PENINSULA", wfpen);
    m.insert("BALEARES", wfbal);
    m.insert("CANARIAS", wfcan);
    m.insert("CEUTAMELILLA", wfcym);
    m
});

/**
Manejo de factores de paso para el CTE
--------------------------------------

Factores de paso y utilidades para la gestión de factores de paso para el CTE
*/

/// Lee factores de paso desde cadena y sanea los resultados.
pub fn wfactors_from_str(
    wfactorsstring: &str,
    user: UserWF<Option<RenNrenCo2>>,
    userdefaults: UserWF<RenNrenCo2>,
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
    locmap: &HashMap<&'static str, Factors>,
    user: UserWF<Option<RenNrenCo2>>,
    userdefaults: UserWF<RenNrenCo2>,
) -> Result<Factors, EpbdError> {
    locmap
        .get(loc)
        .ok_or_else(|| EpbdError::ParseError(format!("Localizacion: {}", loc)))?
        .clone()
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

#[allow(non_snake_case)]
/// Parte renovable de la demanda de ACS considerando el perímetro próximo
///
/// Permite calcular el indicador de HE4 con las siguientes restricciones:
/// - no se usa más de un vector energético que no pertenezca al perímetro in situ
/// - no se permite consumo de electricidad cogenerada (solo la parte térmica).
///   Si se pudiese usar electricidad y existiese cogeneración tendríamos 2 vectores no insitu (BIOMASA, ELECTRICIDAD)
///   y, si no se usase la parte térmica, no sabríamos si tiene procedencia renovable o no.
/// Si no fuese así, necesitaríamos conocer la demanda satisfecha por cada equipo (algo que todavía no hacemos)
pub fn demanda_renovable_acs_nrb(
    components: &Components,
    wfactors: &Factors,
) -> Result<f32, EpbdError> {
    let get_fp_ren = |c: &Component| -> Result<f32, EpbdError> {
        // El origen es la red, salvo para la electricidad producida in situ
        let src = match c.carrier {
            Carrier::ELECTRICIDAD => Source::INSITU,
            _ => Source::RED,
        };
        wfactors
            .wdata
            .iter()
            .find(|f| f.carrier == c.carrier && f.source == src)
            .ok_or_else(|| {
                EpbdError::WrongInput(format!(
                    "No se encuentra el factor de paso para \"{}\"",
                    c.carrier
                ))
            })
            .and_then(|f| Ok(f.ren))
    };

    let components = &components.filter_by_epb_service(Service::ACS);
    let cr_list = &components.cdata;

    // Casos sin consumo (o producción) de ACS
    if cr_list.is_empty() {
        return Ok(0.0)
    };

    // Comprobaremos que las hipótesis para poder calcular la demanda renovable se cumplen:
    // - No puede haber consumo de más de un vector que no sea insitu (necesitaríamos rendimientos)
    let n_cr_nearby_distant = cr_list
        .iter()
        .filter(|c| c.carrier != Carrier::MEDIOAMBIENTE && c.ctype == CType::CONSUMO)
        .count();
    if n_cr_nearby_distant > 1 {
        return Err(EpbdError::WrongInput(
            "Más de un vector con origen no in situ".to_string(),
        ));
    };
    let exists_el_cgn = &cr_list
        .iter()
        .find(|c| c.ctype == CType::PRODUCCION && c.csubtype == CSubtype::COGENERACION);
    if exists_el_cgn.is_some() {
        return Err(EpbdError::WrongInput(
            "Uso de electricidad cogenerada".to_string(),
        ));
    };

    // 1. Consumo de electricidad "renovable"
    // No podemos considerar la electricidad producida por cogeneración con biomasa porque no sabemos qué parte de la demanda
    // se genera con cada equipo (biomasa, eléctrico)
    let num_steps = cr_list[0].values.len();

    // a. Total de consumo de electricidad para ACS, de cualquier origen
    let E_EPus_el_t = cr_list
        .iter()
        .filter(|c| c.carrier == Carrier::ELECTRICIDAD)
        .filter(|c| c.ctype == CType::CONSUMO && c.csubtype == CSubtype::EPB)
        .fold(vec![0.0; num_steps], |acc, c| vecvecsum(&acc, &c.values));
    // b. Total de producción de electricidad in situ asignada, en principio, a ACS
    // La producción de cogeneración no la podemos considerar,
    // puesto que si hay consumo eléctrico no sabemos qué parte de la demanda satisface la cogen y qué parte el sistema eléctrico
    // Para poder calcularlo necesitaríamos saber las demandas de cada equipo
    let E_pr_el_onsite_t = cr_list
        .iter()
        .filter(|c| c.carrier == Carrier::ELECTRICIDAD)
        .filter(|c| c.ctype == CType::PRODUCCION && c.csubtype == CSubtype::INSITU)
        .fold(vec![0.0; num_steps], |acc, c| vecvecsum(&acc, &c.values));
    // c. Consumo efectivo de electricidad renovable en ACS (Mínimo entre el consumo y la producción in situ)
    // Consideramos que la conversión es con rendimiento 1.0
    // XXX: La interacción con f_match_t, si se implementa, debería ser correcta, si se ha hecho bien el reparto de electr. por servicio
    let E_pr_el_onsite_used_EPus_an_ren: f32 =
        vecvecmin(&E_EPus_el_t, &E_pr_el_onsite_t).iter().sum();

    // 2. Otros consumos Nearby
    // Podemos obtener la parte renovable, con su factor de paso y
    // suponiendo que la conversión a calor es con rendimiento 1.0
    let E_EPus_nrb_an_ren = cr_list
        .iter()
        .filter(|c| c.ctype == CType::CONSUMO && CTE_NRBY.contains(&c.carrier))
        .map(|c| Ok(c.values.iter().sum::<f32>() * get_fp_ren(c)?))
        .collect::<Result<Vec<f32>, EpbdError>>()?
        .iter()
        .sum::<f32>();

    Ok(E_pr_el_onsite_used_EPus_an_ren + E_EPus_nrb_an_ren)
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

    let out = format!(
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
    );
    // Añade parámetros de demanda HE4 si existen
    if let Some(map) = &balance.misc {
        let demanda = map
            .get("demanda_anual_acs")
            .and_then(|v| v.parse::<f32>().and_then(|r| Ok(format!("{:.1}", r))).ok())
            .unwrap_or_else(|| "-".to_string());
        let pct_ren = map
            .get("fraccion_renovable_demanda_acs_nrb")
            .and_then(|v| {
                v.parse::<f32>()
                    .and_then(|r| Ok(format!("{:.1}", r * 100.0)))
                    .ok()
            })
            .unwrap_or_else(|| "-".to_string());
        format!(
            "{}
** Indicadores adicionales
Demanda total de ACS: {} [kWh]
Porcentaje renovable de la demanda de ACS (perímetro próximo): {} [%]
",
            out, demanda, pct_ren
        )
    } else {
        out
    }
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
