// Copyright (c) 2018-2023  Ministerio de Fomento
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
*/

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

use crate::{error::EpbdError, types::*, Factors, UserWF};

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

/// Factores de paso definibles por el usuario usados por defecto
pub const CTE_USERWF: UserWF<RenNrenCo2> = UserWF {
    red1: RenNrenCo2::new(0.0, 1.3, 0.3),
    red2: RenNrenCo2::new(0.0, 1.3, 0.3),
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
            Factor::new(EAMBIENTE, RED, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para suministrar energía ambiente (red de suministro ficticia)"),
            Factor::new(EAMBIENTE, INSITU, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para generar in situ energía ambiente (vector renovable)"),
            Factor::new(TERMOSOLAR, RED, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para suministrar energía solar térmica (red de suministro ficticia)"),
            Factor::new(TERMOSOLAR, INSITU, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para generar in situ energía solar térmica (vector renovable)"),
            Factor::new(BIOCARBURANTE, RED, SUMINISTRO, A, (1.028, 0.085, 0.018).into(), "Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))"),
            Factor::new(BIOMASA, RED, SUMINISTRO, A, (1.003, 0.034, 0.018).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(BIOMASADENSIFICADA, RED, SUMINISTRO, A, (1.028, 0.085, 0.018).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(CARBON, RED, SUMINISTRO, A, (0.002, 1.082, 0.472).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(GASNATURAL, RED, SUMINISTRO, A, (0.005, 1.190, 0.252).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(GASOLEO, RED, SUMINISTRO, A, (0.003, 1.179, 0.311).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(GLP, RED, SUMINISTRO, A, (0.003, 1.201, 0.254).into(), "Recursos usados para suministrar el vector desde la red"),
            Factor::new(ELECTRICIDAD, INSITU, SUMINISTRO, A, (1.000, 0.000, 0.000).into(), "Recursos usados para producir electricidad in situ"),
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

/*
Porcentaje renovable de la demanda de ACS en el perímetro próximo
-----------------------------------------------------------------
*/

/// Devuelve eficiencia energética con datos de demanda renovable de ACS en perímetro próximo incorporados
pub fn incorpora_demanda_renovable_acs_nrb(mut ep: EnergyPerformance) -> EnergyPerformance {
    // Añadir a EnergyPerformance.misc un diccionario, si no existe, con datos:
    let mut map = ep.misc.take().unwrap_or_default();

    match fraccion_renovable_acs_nrb(&ep) {
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
    ep.misc = Some(map);
    ep
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
/// 3. el rendimiento térmico de la contribución renovable de vectores RED1, RED2 y EAMBIENTE es 1.0. (demanda == consumo)
/// 4. las únicas aportaciones nearby son biomasa (cualquiera), RED1, RED2, ELECTRICIDAD insitu y EAMBIENTE (insitu)
///
/// Se pueden excluir consumos eléctricos auxiliares con la etiqueta CTEEPBD_EXCLUYE_AUX_ACS o CTEEPBD_AUX en el comentario del componente de consumo y vector ELECTRICIDAD
/// Se pueden excluir producciones renovables para equipos con SCOP < 2,5 con la etiqueta CTEEPBD_EXCLUYE_SCOP_ACS en el comentario del componente de vector EAMBIENTE
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
pub fn fraccion_renovable_acs_nrb(ep: &EnergyPerformance) -> Result<f32, EpbdError> {
    use Carrier::{BIOMASA, BIOMASADENSIFICADA, EAMBIENTE, ELECTRICIDAD};

    let bal = &ep.balance;

    // Demanda anual de ACS
    let demanda_anual_acs = match bal.needs.ACS {
        // Sin demanda anual de ACS definida
        None => {
            return Err(EpbdError::WrongInput(
                "Demanda anual de ACS desconocida".to_string(),
            ));
        }
        Some(demanda) => demanda,
    };

    // Consumo de de ACS por vectores
    let dhw_used_by_cr = bal
        .used
        .epus_by_cr_by_srv
        .get(&Service::ACS)
        .cloned()
        .unwrap_or_default();

    // Calcula consumo de ACS por vectores descontando AUX y consumos de EAMBIENTE de bajo SCOP
    // Los consumos de EAMBIENTE excluidos son los marcados con CTEEPBD_EXCLUYE_SCOP_ACS
    let mut dhw_used_by_cr_no_aux_or_low_scop = dhw_used_by_cr.clone();
    let dhw_aux_use_an = ep
        .components
        .data
        .iter()
        .filter(|c| c.is_aux() && c.has_service(Service::ACS))
        .map(HasValues::values_sum)
        .sum::<f32>();
    dhw_used_by_cr_no_aux_or_low_scop
        .entry(Carrier::ELECTRICIDAD)
        .and_modify(|e| *e -= dhw_aux_use_an);
    if dhw_used_by_cr_no_aux_or_low_scop
        .get(&Carrier::ELECTRICIDAD)
        .map(|v| v.abs() < 0.01)
        .unwrap_or(false)
    {
        dhw_used_by_cr_no_aux_or_low_scop.remove(&ELECTRICIDAD);
    };
    let dhw_used_low_scop_an: f32 = ep
        .components
        .data
        .iter()
        .filter(|c| {
            c.is_used()
                && c.has_carrier(EAMBIENTE)
                && c.comment().contains("CTEEPBD_EXCLUYE_SCOP_ACS")
        })
        .map(HasValues::values_sum)
        .sum();
    dhw_used_by_cr_no_aux_or_low_scop
        .entry(EAMBIENTE)
        .and_modify(|e| *e -= dhw_used_low_scop_an);

    // Casos sin consumo de ACS
    if dhw_used_by_cr_no_aux_or_low_scop.is_empty() {
        return Ok(0.0);
    };
    if dhw_used_by_cr_no_aux_or_low_scop
        .get(&Carrier::EAMBIENTE)
        .map(|v| v.abs() < 0.01)
        .unwrap_or(false)
    {
        dhw_used_by_cr_no_aux_or_low_scop.remove(&EAMBIENTE);
    };

    // Demanda anual de ACS nula
    if demanda_anual_acs.abs() < f32::EPSILON {
        return Err(EpbdError::WrongInput(
            "Demanda anual de ACS nula o casi nula".to_string(),
        ));
    };

    // Comprobaremos las condiciones para poder calcular las aportaciones renovables a la demanda
    //
    // 1. Las aportaciones de redes de distrito RED1, RED2,TERMOSOLAR y EAMBIENTE son aportaciones renovables según sus factores de paso (fp_ren / fp_tot)
    // 2. La biomasa (o biomasa densificada)
    //  - si solo se consume uno de esos vectores o vectores insitu o de distrito, y se cubre el 100% de la demanda podemos calcular
    //  - si tenemos el porcentaje de demanda cubierto por la biomasa o biomasa in situ, podemos calcular la demanda renovable.
    //  - en ambos casos se usa también la proporción de los factores de paso
    // 3. La ELECTRICIDAD consumida en ACS y producida in situ se toma como renovable en un 100% (rendimiento térmico == 1 y demanda == consumo).
    // 4. ELECTRICIDAD cogenerada, se toma como renovable en la fracción que lo es su vector nearby

    // 1. == Energía ambiente y distrito ==
    // Demanda total y renovable de los consumos de ACS de RED1, RED2, TERMOSOLAR o EAMBIENTE (demanda == consumo)
    // En el caso de EAMBIENTE se excluyen los consumos con la etiqueta CTEEPBD_EXCLUYE_SCOP_ACS
    // Podemos obtener la parte renovable, con la fracción que supone su factor de paso ren respecto al total y
    // suponiendo que la conversión de consumo a demanda es con rendimiento 1.0 (de modo que demanda = consumo para estos vectores)
    // En el caso de la biomasa la conversión depende del rendimiento del sistema
    let (Q_nrb_non_biomass_an_tot, Q_nrb_non_biomass_an_ren) =
        Q_nrb_non_biomass_an(&dhw_used_by_cr_no_aux_or_low_scop, ep)?;

    // 2. == Biomasa ==
    // Vectores energéticos consumidos
    let has_biomass = dhw_used_by_cr_no_aux_or_low_scop.contains_key(&BIOMASA);
    let has_dens_biomass = dhw_used_by_cr_no_aux_or_low_scop.contains_key(&BIOMASADENSIFICADA);
    let has_any_biomass = has_biomass || has_dens_biomass;
    let has_only_one_type_of_biomass =
        (has_biomass || has_dens_biomass) && !(has_biomass && has_dens_biomass);
    let has_only_nearby = dhw_used_by_cr_no_aux_or_low_scop
        .keys()
        .all(|&c| c.is_nearby());

    let Q_biomass_an_ren = if has_only_one_type_of_biomass && has_only_nearby {
        // Solo hay un tipo de biomasa y no hay otros vectores que no sean de distrito o energía ambiente
        // entonces podemos calcular el % de la demanda de ACS abastecida por la biomasa
        // ya que es toda la no cubierta por el resto de vectores
        let Q_any_biomass_acs_an = demanda_anual_acs - Q_nrb_non_biomass_an_tot;
        // Parte renovable: Q_any_biomass_acs_an_ren
        if has_biomass {
            Q_any_biomass_acs_an * get_fpA_del_ren_fraction(BIOMASA, &ep.wfactors)?
        } else {
            Q_any_biomass_acs_an * get_fpA_del_ren_fraction(BIOMASADENSIFICADA, &ep.wfactors)?
        }
    } else if has_any_biomass {
        // Cuando además de biomasa hay otros vectores que no son de distrito o insitu
        // necesitamos saber qué cantidad de ACS produce la biomasa para poder calcular
        let Q_biomass_an_ren = if has_biomass {
            let fp_ren_fraction_biomass = get_fpA_del_ren_fraction(BIOMASA, &ep.wfactors)?;
            // Id de sistemas con uso de BIOMASA para ACS
            let idx_with_acs_use = Vec::from_iter(
                ep.components
                    .data
                    .iter()
                    .filter(|c| {
                        c.is_used() && c.has_service(Service::ACS) && c.has_carrier(BIOMASA)
                    })
                    .map(|c| c.id())
                    .collect::<HashSet<i32>>(),
            );
            // Comprobar que se ha definido la salida de ACS para equipos de BIOMASA
            for idx in &idx_with_acs_use {
                if !ep
                    .components
                    .data
                    .iter()
                    .any(|c| c.has_id(*idx) && c.is_out() && c.has_service(Service::ACS))
                {
                    return Err(EpbdError::WrongInput(
                        format!("Uso de biomasa en el sistema con id:{} sin definición de la energía entregada para el servicio de ACS.", idx),
                    ));
                }
            }
            // Suma de demandas de ACS salientes de equipos con consumo de BIOMASA
            let alt_tot_dhw_use: f32 = ep
                .components
                .data
                .iter()
                .filter(|c| {
                    idx_with_acs_use.contains(&c.id()) && c.is_out() && c.has_service(Service::ACS)
                })
                .map(HasValues::values_sum)
                .sum();
            alt_tot_dhw_use * fp_ren_fraction_biomass
        } else {
            0.0
        };
        let Q_dens_biomass_an_ren = if has_dens_biomass {
            let fp_ren_fraction_dens_biomass =
                get_fpA_del_ren_fraction(BIOMASADENSIFICADA, &ep.wfactors)?;
            // Id de sistemas con uso de BIOMASADENSIFICADA para ACS
            let idx_with_acs_use = Vec::from_iter(
                ep.components
                    .data
                    .iter()
                    .filter(|c| {
                        c.is_used()
                            && c.has_service(Service::ACS)
                            && c.has_carrier(BIOMASADENSIFICADA)
                    })
                    .map(|c| c.id())
                    .collect::<HashSet<i32>>(),
            );
            // Comprobar que se ha definido la salida de ACS para equipos de BIOMASADENSIFICADA
            for idx in &idx_with_acs_use {
                if !ep
                    .components
                    .data
                    .iter()
                    .any(|c| c.has_id(*idx) && c.is_out() && c.has_service(Service::ACS))
                {
                    return Err(EpbdError::WrongInput(
                        format!("Uso de biomasa en el sistema con id:{} sin definición de la energía entregada para el servicio de ACS.", idx),
                    ));
                }
            }
            // Suma de demandas de ACS salientes de equipos con consumo de BIOMASADENSIFICADA
            let alt_tot_dhw_use: f32 = ep
                .components
                .data
                .iter()
                .filter(|c| {
                    idx_with_acs_use.contains(&c.id()) && c.is_out() && c.has_service(Service::ACS)
                })
                .map(HasValues::values_sum)
                .sum();
            alt_tot_dhw_use * fp_ren_fraction_dens_biomass
        } else {
            0.0
        };
        Q_biomass_an_ren + Q_dens_biomass_an_ren
    } else {
        // No hay ningún tipo de biomasa
        0.0
    };

    // 3. === Electricidad producida in situ (EL_INSITU) ===
    // Consumo de electricidad "renovable" (consumo == demanda)
    // sin considerar consumos auxiliares de ACS, que no se convierten en demanda

    // a) Fracción del consumo eléctrico para ACS que suponen los auxiliares
    let frac_non_aux_el_use_dhw = {
        let dhw_el_used_an = dhw_used_by_cr.get(&ELECTRICIDAD).unwrap_or(&0.0);
        if dhw_el_used_an.abs() > f32::EPSILON {
            1.0 - (dhw_aux_use_an / dhw_el_used_an)
        } else {
            1.0
        }
    };
    // b) Producción in situ destinada a ACS, incluidos auxiliares de ACS
    let prod_el_onst_dhw = bal
        .prod
        .epus_by_srv_by_src
        .get(&ProdSource::EL_INSITU)
        .and_then(|by_src| by_src.get(&Service::ACS))
        .copied()
        .unwrap_or_default();
    // c) Producción insitu EL_INSITU destinada a ACS, excluidos auxiliares
    let Q_onst_el_an_ren = prod_el_onst_dhw * frac_non_aux_el_use_dhw;

    // 4. === Cogeneración ==
    // Consideramos la electricidad cogenerada con vectores nearby no usada para consumos auxiliares
    // XXX: Duda: ¿es la cogeneración una fuente nearby solo cuando el vector que lo alimenta es nearby o siempre?

    // 1. Hay producción de electricidad cogenerada que se usa en ACS
    let dhw_cogen_use = ep
        .balance
        .prod
        .epus_by_srv_by_src
        .get(&ProdSource::EL_COGEN)
        .and_then(|s| s.get(&Service::ACS))
        .cloned()
        .unwrap_or_default();
    // 2. La electricidad destinada a usos EPB va más allá de los auxiliares
    let dhw_el_use_no_aux_or_low_scop = dhw_used_by_cr_no_aux_or_low_scop
        .get(&ELECTRICIDAD)
        .cloned()
        .unwrap_or_default();
    // 3. La cogeneración se produce con algún vector del perímetro próximo
    let cogen_sources: Vec<_> = ep
        .components
        .data
        .iter()
        .filter(|c| c.is_cogen_use())
        .collect();
    let cogen_sources_has_nearby = cogen_sources.iter().any(|c| c.carrier().is_nearby());
    let Q_nrb_cogen_el_an_ren =
        if dhw_el_use_no_aux_or_low_scop > 0.0 && dhw_cogen_use > 0.0 && cogen_sources_has_nearby {
            // A diferencia de la generación in situ, la electricidad cogenerada se convierte en demanda
            // con un factor que depende del vector usado para generarla.
            // Tenemos que calcular el factor de paso para obtener
            //  f_ren_cgn_nrb = f_ren_nrb / f_tot
            // f_ren_nrb = suma (f_pA_cr_i.ren * consumo_cogen_cr_i) cuando cr_i es nrb
            // f_tot = suma(f_pA_cr_i.tot * consumo_cogen_cr_i)
            let f_ren_cgn_nrb = {
                let f_cgn_A = ep.wfactors.find(
                    Carrier::ELECTRICIDAD,
                    Source::COGEN,
                    Dest::SUMINISTRO,
                    Step::A,
                )?;
                let f_tot = f_cgn_A.ren + f_cgn_A.nren;
                if f_tot > 0.0 {
                    let f_cgn_ren_A = ep
                        .wfactors
                        .compute_cgn_exp_fP_A(&ep.components, true)?
                        .unwrap_or_default()
                        .ren;
                    println!("f_cgn_ren_A: {f_cgn_ren_A:.3}, f_tot: {f_tot:.3}");
                    f_cgn_ren_A / f_tot
                } else {
                    0.0
                }
            };
            // Fracción de la electricidad cogenerada que no va a auxiliares
            let dhw_non_aux_cogen_use = dhw_cogen_use * frac_non_aux_el_use_dhw;

            // fracción renovable de cada unidad cogenerada
            dhw_non_aux_cogen_use * f_ren_cgn_nrb
        } else {
            0.0
        };

    // 5. === Total de demanda renovable ==
    let Q_an_ren =
        Q_nrb_non_biomass_an_ren + Q_biomass_an_ren + Q_onst_el_an_ren + Q_nrb_cogen_el_an_ren;

    Ok(Q_an_ren / demanda_anual_acs)
}

// Funciones auxiliares ----------

/// Cálculo de la fracción que supone el factor de paso a energía primaria renovable respecto a la energía primaria total
#[allow(non_snake_case)]
fn get_fpA_del_ren_fraction(c: Carrier, wfactors: &Factors) -> Result<f32, EpbdError> {
    // El origen es la red, salvo para la electricidad producida in situ
    let src = match c {
        Carrier::ELECTRICIDAD => Source::INSITU,
        _ => Source::RED,
    };
    wfactors
        .wdata
        .iter()
        .find(|f| {
            f.carrier == c && f.source == src && f.dest == Dest::SUMINISTRO && f.step == Step::A
        })
        .ok_or_else(|| {
            EpbdError::WrongInput(format!("No se encuentra el factor de paso para \"{}\"", c))
        })
        .map(|f| f.ren / (f.ren + f.nren))
}

#[allow(non_snake_case)]
/// Demanda total y renovable de los consumos de ACS cubierto por vectores nearby que no sean biomasa
/// (EAMBIENTE, RED1, RED2 o TERMOSOLAR)
///
fn Q_nrb_non_biomass_an(
    dhw_used_by_cr_no_aux_or_low_scop: &HashMap<Carrier, f32>,
    ep: &EnergyPerformance,
) -> Result<(f32, f32), EpbdError> {
    use Carrier::{BIOMASA, BIOMASADENSIFICADA};

    let (mut tot, mut ren) = (0.0, 0.0);

    if !dhw_used_by_cr_no_aux_or_low_scop.is_empty() {
        // Energía usada en vectores nearby que no son biomasa
        for (carrier, us) in dhw_used_by_cr_no_aux_or_low_scop {
            if carrier.is_nearby() && *carrier != BIOMASA && *carrier != BIOMASADENSIFICADA {
                tot += us;
                ren += us * get_fpA_del_ren_fraction(*carrier, &ep.wfactors)?;
            }
        }
    }

    Ok((tot, ren))
}
