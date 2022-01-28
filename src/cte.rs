// Copyright (c) 2018-2022  Ministerio de Fomento
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

/*
Porcentaje renovable de la demanda de ACS en el perímetro próximo
-----------------------------------------------------------------
*/

// Funciones auxiliares ----------

/// Cálculo de la fracción que supone el factor de paso a energía primaria renovable respecto a la energía primaria total
fn get_fp_ren_fraction(c: Carrier, wfactors: &Factors) -> Result<f32, EpbdError> {
    // El origen es la red, salvo para la electricidad producida in situ
    let src = match c {
        Carrier::ELECTRICIDAD => Source::INSITU,
        _ => Source::RED,
    };
    wfactors
        .wdata
        .iter()
        .find(|f| f.carrier == c && f.source == src)
        .ok_or_else(|| {
            EpbdError::WrongInput(format!("No se encuentra el factor de paso para \"{}\"", c))
        })
        .map(|f| f.ren / (f.ren + f.nren))
}

#[allow(non_snake_case)]
/// Demanda total y renovable de los consumos de ACS de RED1, RED2 o MEDIOAMBIENTE
///
/// Podemos obtener la parte renovable, con la fracción que supone su factor de paso ren respecto al total y
/// suponiendo que la conversión de consumo a demanda es con rendimiento 1.0 (de modo que demanda = consumo para estos vectores)
fn Q_district_and_env_an(
    cr_list: &[&EnergyData],
    wfactors: &Factors,
) -> Result<(f32, f32), EpbdError> {
    use Carrier::{MEDIOAMBIENTE, RED1, RED2};

    let value = cr_list
        .iter()
        .filter(|c| {
            c.is_used()
                && (c.has_carrier(RED1) || c.has_carrier(RED2) || c.has_carrier(MEDIOAMBIENTE))
        })
        .map(|c| {
            let tot = c.values_sum();
            let ren = tot * get_fp_ren_fraction(c.carrier(), wfactors)?;
            Ok((tot, ren))
        })
        .collect::<Result<Vec<(f32, f32)>, EpbdError>>()?
        .iter()
        .fold((0.0, 0.0), |(ac_tot, ac_ren), &(elem_tot, elem_ren)| {
            (ac_tot + elem_tot, ac_ren + elem_ren)
        });
    Ok(value)
}

/// Vectores energéticos consumidos
fn get_used_carriers(cr_list: &[&EnergyData]) -> Vec<Carrier> {
    let mut used_carriers = cr_list
        .iter()
        .filter(|c| c.is_used())
        .map(|c| c.carrier())
        .collect::<Vec<_>>();
    used_carriers.sort_unstable();
    used_carriers.dedup();
    used_carriers
}

#[allow(non_snake_case)]
/// Fracción de la demanda de ACS con origen renovable, considerando el perímetro próximo
///
/// Permite calcular el indicador de HE4 con las siguientes restricciones:
///
/// 1. si hay biomasa (o biomasa densificada), esta y otros vectores insitu o de distrito cubren el 100% de la demanda
/// 2. no se permite el consumo de electricidad cogenerada para producir ACS (solo la parte térmica) aunque podría provenir de BIOMASA / BIOMASADENSIFICADA
///     Si se pudiese usar electricidad y existiese cogeneración tendríamos 2 vectores no insitu (BIOMASA, ELECTRICIDAD)
///     y, si no se usase la parte térmica, no sabríamos si tiene procedencia renovable o no.
/// 3. el rendimiento térmico de la contribución renovable de vectores RED1, RED2 y MEDIOAMBIENTE es 1.0. (demanda == consumo)
/// 4. las únicas aportaciones nearby son biomasa (cualquiera), RED1, RED2, ELECTRICIDAD insitu y MEDIOAMBIENTE (insitu)
///
/// Se pueden excluir consumos eléctricos auxiliares con la etiqueta CTEEPBD_EXCLUYE_AUX_ACS o CTEEPBD_AUX en el comentario del componente de consumo y vector ELECTRICIDAD
/// Se pueden excluir producciones renovables para equipos con SCOP < 2,5 con la etiqueta CTEEPBD_EXCLUYE_SCOP_ACS en el comentario del componente de vector MEDIOAMBIENTE
///
/// Casos que no podemos calcular:
/// - Cuando hay electricidad cogenerada
///     - En este caso sería necesario que la imputación del combustible fuese en función del destino final del consumo,
///       sea eléctrico o térmico. Alternativamente se podrían modificar los factores de paso, pero parece más complicado. Analizar.
///       Se podría estudiar hacer un reparto de la producción de combustible para generar electricidad en función del reparto de la
///       electricidad cogenerada por usos. Pensar qué ocurre con parte exportada
///
///       También se puede resolver si separamos el uso térmico del eléctrico en la cogeneración (y asignaríamos la poporción de electricidad cogenerada asignada a ACS).
/// - Cuando necesitaríamos conocer el % de la demanda anual de ACS satisfecha por el vector BIOMASA y BIOMASADENSIFICADA porque
///     - Hay BIOMASA o BIOMASADENSIFICADA y otro vector que no sea insitu o de distrito.
///      
///       Como esos son los únicos vectores para los que necesitamos saber el porcentaje de producción de ACS que suponen, nos bastaría para
///       hacer el cálculo (ahora lo obtenemos por sustracción de las aportaciones en las que consumo === demanda) aún en presencia
///       de más de un vector no in situ.
///
///       Podemos resolver esto también si se incluye la energía entregada o absorbida por los equipos (id, Q_OUT) y viendo la proporción
///       que supone sobre la demanda global del edificio (id=0, DEMANDA).
///
pub fn fraccion_renovable_acs_nrb(
    components: &Components,
    wfactors: &Factors,
    demanda_anual_acs: f32,
) -> Result<f32, EpbdError> {
    use Carrier::{BIOMASA, BIOMASADENSIFICADA, ELECTRICIDAD, MEDIOAMBIENTE, RED1, RED2};

    // Lista de componentes para ACS y filtrados excluidos de participar en el cálculo de la demanda renovable
    let components = &components.filter_by_epb_service(Service::ACS);
    let cr_list: &Vec<&EnergyData> = &components
        .cdata
        .iter()
        .filter(|c| {
            !((c.has_carrier(ELECTRICIDAD) && c.comment().contains("CTEEPBD_EXCLUYE_AUX_ACS"))
                || (c.has_carrier(MEDIOAMBIENTE)
                    && c.comment().contains("CTEEPBD_EXCLUYE_SCOP_ACS")))
        })
        .collect();

    // Casos sin consumo (o producción) de ACS
    if cr_list.is_empty() {
        return Ok(0.0);
    };

    // Demanda anual de ACS nula
    if demanda_anual_acs.abs() < f32::EPSILON {
        return Err(EpbdError::WrongInput(
            "Demanda anual de ACS nula o casi nula".to_string(),
        ));
    };

    // Existe cogeneración eléctrica -> caso no soportado -> ERROR
    //
    // Si tenemos electricidad cogenerada no sabemos con qué se ha cogenerado ni si se ha imputado todo el combustible correspondiente
    // ya que este podría ir a otros usos y no a ACS (y no tenemos los factores de paso de electricidad cogenerada)
    //
    // TODO: Para poder tenerlo en cuenta tendríamos dos opciones:
    // - Imputar correctamente los factores de paso de electricidad cogenerada, en lugar de 0.0 (y ver cómo se imputa el combustible en la parte térmica)
    // - Imputar el consumo de combustible en función del servicio de destino de la electricidad y de la parte térmica. Esto se podría hacer a la
    //   hora del reparto de electricidad, si se ha marcado el consumo de combustible como destinado a cogeneración eléctrica CTEEPBD_DESTINO_COGEN y uso NDEF.
    //   Habría que pensar qué ocurre si una parte no se consume y se exporta.
    // - Habría que ver cómo se imputa (prioridad) el consumo de electricidad in situ y cogenerada.
    let has_el_cgn = cr_list.iter().any(|c| c.is_cogen_pr());
    if has_el_cgn {
        return Err(EpbdError::WrongInput(
            "Uso de electricidad cogenerada".to_string(),
        ));
    };

    // Comprobaremos las condiciones para poder calcular las aportaciones renovables a la demanda
    //
    // 1. Las aportaciones de redes de distrito RED1 y RED2 y MEDIOAMBIENTE son aportaciones renovables según sus factores de paso (fp_ren / fp_tot)
    // 2. La biomasa (o biomasa densificada)
    //  - si solo se consume uno de esos vectores o vectores insitu o de distrito, y se cubre el 100% de la demanda podemos calcular
    //  - si tenemos el porcentaje de demanda cubierto por la biomasa o biomasa in situ, podemos calcular la demanda renovable.
    //  - en ambos casos se usa también la proporción de los factores de paso
    // 3. La ELECTRICIDAD consumida en ACS y producida in situ se toma como renovable en un 100% (rendimiento térmico == 1 y demanda == consumo).

    // 1. == Energía ambiente y distrito ==
    // Demanda total y renovable de los consumos de ACS de RED1, RED2 o MEDIOAMBIENTE (demanda == consumo)
    let (Q_district_and_env_an_tot, Q_district_and_env_acs_an_ren) =
        Q_district_and_env_an(cr_list, wfactors)?;

    // 2. == Biomasa ==
    // Vectores energéticos consumidos
    let used_carriers = get_used_carriers(cr_list);
    let has_biomass = used_carriers.contains(&BIOMASA);
    let has_dens_biomass = used_carriers.contains(&BIOMASADENSIFICADA);
    let has_any_biomass = has_biomass || has_dens_biomass;
    let has_only_one_type_of_biomass =
        (has_biomass || has_dens_biomass) && !(has_biomass && has_dens_biomass);
    let has_only_biomass_or_onsite_or_district = !used_carriers.iter().any(|&c| {
        c != MEDIOAMBIENTE && c != RED1 && c != RED2 && c != BIOMASA && c != BIOMASADENSIFICADA
    });

    let Q_biomass_an_ren = if has_only_one_type_of_biomass && has_only_biomass_or_onsite_or_district
    {
        // Solo hay un tipo de biomasa y no hay otros vectores que no sean de distrito o energía ambiente
        // entonces podemos calcular el % de la demanda de ACS abastecida por la biomasa
        let Q_any_biomass_acs_an = demanda_anual_acs - Q_district_and_env_an_tot;
        // Parte renovable: Q_any_biomass_acs_an_ren
        if has_biomass {
            Q_any_biomass_acs_an * get_fp_ren_fraction(BIOMASA, wfactors)?
        } else {
            Q_any_biomass_acs_an * get_fp_ren_fraction(BIOMASADENSIFICADA, wfactors)?
        }
    } else if has_any_biomass {
        // Además de biomasa hay otros vectores que no son de distrito o insitu y necesitamos saber qué cantidad de ACS produce la biomasa
        let Q_biomass_an_ren = if has_biomass {
            let fp_ren_fraction_biomass = get_fp_ren_fraction(BIOMASA, wfactors)?;
            let Q_biomass_an_pct = components
                .get_meta_f32("CTE_DEMANDA_ACS_PCT_BIOMASA")
                .ok_or_else(|| {
                    EpbdError::WrongInput(
                        "No se ha especificado el porcentaje de la demanda de ACS abastecida por BIOMASA en el metadato 'CTE_DEMANDA_ACS_PCT_BIOMASA'"
                            .to_string(),
                    )
                })?;
            demanda_anual_acs * Q_biomass_an_pct / 100.0 * fp_ren_fraction_biomass
        } else {
            0.0
        };
        let Q_dens_biomass_an_ren = if has_dens_biomass {
            let fp_ren_fraction_dens_biomass = get_fp_ren_fraction(BIOMASADENSIFICADA, wfactors)?;
            let Q_dens_biomass_an_pct = components
                .get_meta_f32("CTE_DEMANDA_ACS_PCT_BIOMASADENSIFICADA")
                .ok_or_else(|| {
                    EpbdError::WrongInput(
                        "No se ha especificado el porcentaje de la demanda de ACS abastecida por BIOMASADENSIFICADA en el metadato 'CTE_DEMANDA_ACS_PCT_BIOMASADENSIFICADA'"
                            .to_string(),
                    )
                })?;
            demanda_anual_acs * Q_dens_biomass_an_pct / 100.0 * fp_ren_fraction_dens_biomass
        } else {
            0.0
        };
        Q_biomass_an_ren + Q_dens_biomass_an_ren
    } else {
        0.0
    };

    // 3. === Electricidad producida in situ ===
    // Consumo de electricidad "renovable" (consumo == demanda)
    let num_steps = cr_list[0].num_steps();

    // a. Total de consumo de electricidad para ACS, de cualquier origen
    let E_EPus_el_t = cr_list
        .iter()
        .filter(|c| c.is_electricity() && c.is_epb_use())
        .fold(vec![0.0; num_steps], |acc, c| vecvecsum(&acc, c.values()));
    // b. Total de producción de electricidad in situ asignada, en principio, a ACS
    let E_pr_el_onsite_t = cr_list
        .iter()
        .filter(|c| c.is_electricity() && c.is_onsite_pr())
        .fold(vec![0.0; num_steps], |acc, c| vecvecsum(&acc, c.values()));
    // c. Consumo efectivo de electricidad renovable en ACS (Mínimo entre el consumo y la producción in situ) (consumo == demanda)
    let Q_el_an_ren: f32 = vecvecmin(&E_EPus_el_t, &E_pr_el_onsite_t).iter().sum();

    // === Total de demanda renovable ==
    let Q_an_ren = Q_district_and_env_acs_an_ren + Q_biomass_an_ren + Q_el_an_ren;

    Ok(Q_an_ren / demanda_anual_acs)
}

/// Devuelve balance con datos de demanda renovable de ACS en perímetro próximo incorporados
pub fn incorpora_demanda_renovable_acs_nrb(
    mut balance: Balance,
    demanda_anual_acs: Option<f32>,
) -> Balance {
    // Añadir a balance.misc un diccionario, si no existe, con datos:
    let mut map = balance.misc.unwrap_or_else(HashMap::<String, String>::new);
    match demanda_anual_acs {
        Some(demanda_anual_acs) => {
            map.insert(
                "demanda_anual_acs".to_string(),
                format!("{:.1}", demanda_anual_acs),
            );

            match fraccion_renovable_acs_nrb(
                &balance.components,
                &balance.wfactors,
                demanda_anual_acs,
            ) {
                Ok(fraccion_renovable_acs_nrb) => {
                    map.insert(
                        "fraccion_renovable_demanda_acs_nrb".to_string(),
                        format!("{:.3}", fraccion_renovable_acs_nrb),
                    );
                    map.remove("error_acs");
                }
                Err(e) => {
                    map.insert(
                        "error_acs".to_string(),
                        format!(
                            "ERROR: no se puede calcular la demanda renovable de ACS \"{}\"",
                            e
                        ),
                    );
                    map.remove("fraccion_renovable_demanda_acs_nrb");
                }
            }
        }
        _ => {
            map.insert(
                "error_acs".to_string(),
                "ERROR: demanda anual de ACS no definida".to_string(),
            );
        }
    }
    balance.misc = Some(map);
    balance
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
    let mut use_by_service = balance_m2
        .used_EPB_by_service
        .iter()
        .map(|(k, v)| format!("{}: {:.2}", k, v))
        .collect::<Vec<String>>();
    use_by_service.sort();

    // Ponderada por m2 (por uso)
    let mut b_by_service = balance_m2
        .B_by_service
        .iter()
        .map(|(k, v)| {
            format!(
                "{}: ren {:.2}, nren {:.2}, co2: {:.2}",
                k, v.ren, v.nren, v.co2
            )
        })
        .collect::<Vec<String>>();
    b_by_service.sort();

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
        use_by_service.join("\n"),
        b_by_service.join("\n")
    );
    // Añade parámetros de demanda HE4 si existen
    if let Some(map) = &balance.misc {
        let demanda = map
            .get("demanda_anual_acs")
            .and_then(|v| v.parse::<f32>().map(|r| format!("{:.1}", r)).ok())
            .unwrap_or_else(|| "-".to_string());
        let pct_ren = map
            .get("fraccion_renovable_demanda_acs_nrb")
            .and_then(|v| v.parse::<f32>().map(|r| format!("{:.1}", r * 100.0)).ok())
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

    // Formatting
    // TODO: add extra_padding to _to_xml functions
    let wfstring = wfactors_to_xml(wfactors);
    let components_string = components_to_xml(components);

    // Final assembly
    format!(
        "<BalanceEPB>
    {}
    {}
    <kexp>{:.2}</kexp>
    <AreaRef>{:.2}</AreaRef><!-- área de referencia [m2] -->
    <Epm2><!-- C_ep [kWh/m2.an] -->
        <tot>{:.1}</tot>
        <nren>{:.1}</nren>
    </Epm2>
</BalanceEPB>",
        wfstring,
        components_string,
        k_exp,
        arearef,
        ren + nren,
        nren
    )
}

/// Helper function -> XML escape symbols
fn escape_xml(unescaped: &str) -> String {
    unescaped
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "&apos;")
        .replace('"', "&quot;")
}

/// Convert list of numbers to string of comma separated values (2 decimal digits)
fn format_values_2f(values: &[f32]) -> String {
    values
        .iter()
        .map(|v| format!("{:.2}", v))
        .collect::<Vec<String>>()
        .join(",")
}

/// Converte Factor a XML
fn wfactors_to_xml(f: &Factors) -> String {
    let Factors { wmeta, wdata } = &f;
    let wmetastring = wmeta
        .iter()
        .map(meta_to_xml)
        .collect::<Vec<String>>()
        .join("\n");
    let wdatastring = wdata
        .iter()
        .map(factor_to_xml)
        .collect::<Vec<String>>()
        .join("\n");
    format!(
        "<FactoresDePaso>
    {}
    {}
</FactoresDePaso>",
        wmetastring, wdatastring
    )
}

/// Converte Factor a XML
fn components_to_xml(c: &Components) -> String {
    let Components {
        cmeta,
        cdata,
        zones,
        systems,
    } = &c;
    let cmetastring = cmeta
        .iter()
        .map(meta_to_xml)
        .collect::<Vec<String>>()
        .join("\n");
    let cdatastring = cdata
        .iter()
        .map(|c| match c {
            EnergyData::GenCrIn(e) => used_to_xml(e),
            EnergyData::GenProd(e) => produced_to_xml(e),
        })
        .collect::<Vec<String>>()
        .join("\n");
    let zonesdatastring = zones
        .iter()
        .map(zoneneeds_to_xml)
        .collect::<Vec<String>>()
        .join("\n");
    let systemsdatastring = systems
        .iter()
        .map(systemneeds_to_xml)
        .collect::<Vec<String>>()
        .join("\n");
    format!(
        "<Componentes>
    {}
    {}
    {}
    {}
</Componentes>",
        cmetastring, cdatastring, zonesdatastring, systemsdatastring
    )
}

/// Converte Factor a XML
fn factor_to_xml(f: &Factor) -> String {
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
    format!(
        "<Factor><Vector>{}</Vector><Origen>{}</Origen><Destino>{}</Destino><Paso>{}</Paso><ren>{:.3}</ren><nren>{:.3}</nren><co2>{:.3}</co2><Comentario>{}</Comentario></Factor>",
        carrier, source, dest, step, ren, nren, co2, escape_xml(comment)
    )
}

/// Convierte componente de energía producida a XML
fn produced_to_xml(e: &GenProd) -> String {
    let GenProd {
        id,
        carrier,
        source,
        values,
        comment,
    } = e;
    format!(
        "<Produccion><Id>{}</Id><Vector>{}</Vector><Origen>{}</Origen><Valores>{}</Valores><Comentario>{}</Comentario></Produccion>",
        id,
        carrier,
        source,
        format_values_2f(values),
        escape_xml(comment)
    )
}

/// Convierte componente de energía consumida a XML
fn used_to_xml(e: &GenCrIn) -> String {
    let GenCrIn {
        id,
        carrier,
        service,
        values,
        comment,
    } = e;
    format!(
        "<Consumo><Id>{}</Id><Vector>{}</Vector><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></Consumo>",
        id,
        carrier,
        service,
        format_values_2f(values),
        escape_xml(comment)
    )
}

/// Convierte metadato a XML
fn meta_to_xml(m: &Meta) -> String {
    format!(
        "<Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
        escape_xml(&m.key),
        escape_xml(&m.value)
    )
}

/// Convierte componente de demanda de zona a XML
fn zoneneeds_to_xml(e: &ZoneNeeds) -> String {
    let ZoneNeeds {
        id,
        service,
        values,
        comment,
    } = e;
    format!(
        "<DemandaZona><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></DemandaZona>",
        id,
        service,
        format_values_2f(values),
        escape_xml(comment)
    )
}

/// Convierte componente de energía consumida a XML
fn systemneeds_to_xml(e: &GenOut) -> String {
    let GenOut {
        id,
        service,
        values,
        comment,
    } = e;
    format!(
        "<DemandaSistema><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></DemandaSistema>",
        id,
        service,
        format_values_2f(values),
        escape_xml(comment)
    )
}
