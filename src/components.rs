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
Componentes energéticos
=======================

Define el tipo Components (lista de componentes + metadatos) y sus traits.

Los componentes modelizan el uso y producción de energía en el periodo de cálculo.

Hipótesis:

- Se completa automáticamente el consumo de energía procedente del medioambiente con una producción
- No se permite la producción de electricidad a usos concretos (se asume NDEF) (XXX: se podría eliminar)
*/

use std::{collections::HashSet, fmt, str};

use serde::{Deserialize, Serialize};

use crate::{
    error::EpbdError,
    types::{
        Carrier, EnergyData, HasValues, Meta, MetaVec, Source, GenProd, Service,
        GenOut, GenCrIn, ZoneNeeds,
    },
    vecops::{veclistsum, vecvecdif, vecvecmin, vecvecmul, vecvecsum},
};

/// Lista de datos de componentes con sus metadatos
///
/// List of component data bundled with its metadata
///
/// #META CTE_AREAREF: 100.5
/// 0, ELECTRICIDAD,CONSUMO,EPB,16.39,13.11,8.20,7.38,4.10,4.92,6.56,5.74,4.10,6.56,9.84,13.11
/// 0, ELECTRICIDAD,PRODUCCION,INSITU,8.20,6.56,4.10,3.69,2.05,2.46,3.28,2.87,2.05,3.28,4.92,6.56
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Metadata
    pub cmeta: Vec<Meta>,
    /// Used or produced energy data
    pub cdata: Vec<EnergyData>,
    /// Zone data
    pub zones: Vec<ZoneNeeds>,
    /// System data
    pub systems: Vec<GenOut>,
}

impl MetaVec for Components {
    fn get_metavec(&self) -> &Vec<Meta> {
        &self.cmeta
    }
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta> {
        &mut self.cmeta
    }
}

impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metalines = self
            .cmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self
            .cdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for Components {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Components, Self::Err> {
        let s_nobom = s.strip_prefix('\u{feff}').or(Some(s)).unwrap();
        let lines: Vec<&str> = s_nobom.lines().map(str::trim).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let cmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Meta>, _>>()?;

        let mut cdata = Vec::new();
        let mut zones = Vec::new();
        let mut systems = Vec::new();

        // Tipos disponibles
        let ctypes_tag_list = ["CONSUMO", "PRODUCCION", "ZONA", "GEN"];

        for line in datalines {
            let tags: Vec<_> = line.splitn(4, ',').map(str::trim).skip(1).take(2).collect();
            let tag1 = tags.get(0).unwrap_or(&"");
            let tag2 = tags.get(1).unwrap_or(&"");
            let tag = if ctypes_tag_list.contains(tag1) {
                tag1
            } else {
                tag2
            };
            match *tag {
                "CONSUMO" => cdata.push(EnergyData::GenCrIn(line.parse::<GenCrIn>()?)),
                "PRODUCCION" => cdata.push(EnergyData::GenProd(line.parse()?)),
                "ZONA" => zones.push(line.parse()?),
                "GEN" => systems.push(line.parse()?),
                _ => {
                    return Err(EpbdError::ParseError(format!(
                        "ERROR: No se reconoce el componente de la línea: {}",
                        line
                    )))
                }
            }
        }

        // Check that all used or produced energy components have an equal number of steps (data lengths)
        {
            let cdata_lengths: Vec<_> = cdata.iter().map(|e| e.num_steps()).collect();
            let start_num_steps = *cdata_lengths.get(0).unwrap_or(&12);
            if cdata_lengths.iter().any(|&clen| clen != start_num_steps) {
                return Err(EpbdError::ParseError(
                    "Componentes con distinto número de pasos de cálculo".into(),
                ));
            }
        }
        Ok(Components {
            cmeta,
            cdata,
            zones,
            systems,
        })
    }
}

impl Components {
    /// Conjunto de vectores energéticos disponibles en componentes de energía consumida o producida
    pub fn available_carriers(&self) -> HashSet<Carrier> {
        self.cdata
            .iter()
            .filter(|c| c.is_used() || c.is_generated())
            .map(|e| e.carrier())
            .collect()
    }

    /// Corrige los componentes de consumo y producción
    ///
    /// - Asegura que la energía MEDIOAMBIENTE consumida tiene su producción correspondiente
    /// - Asegura que la energía eléctrica producida no tiene un uso que no sea NDEF
    ///
    /// Los metadatos, servicios y coherencia de los vectores se aseguran ya en el parsing
    pub fn normalize(mut self) -> Self {
        self.compensate_env_use();
        self
    }

    /// Filtra Componentes relacionados con un servicio EPB
    ///
    /// 1. Selecciona todos los consumos y producciones asignados al servicio elegido (se excluyen componentes de zona y sistema)
    /// 2. Toma las producciones eléctricas
    /// 3. Reparte las producciones eléctricas en proporción al consumo del servicio elegido respecto al consumo EPB
    ///
    /// *Nota*: los componentes deben estar normalizados (ver método normalize) para asegurar que:
    /// - los consumos de MEDIOAMBIENTE de un servicio ya están equilibrados
    /// - las producciones eléctricas no pueden ser asignadas a un servicio (siempre son a NDEF)
    /// - la producción eléctrica o de energía ambiente no distingue entre sistemas y
    ///   se considera que siempre forman un pool con reparto según consumos.
    #[allow(non_snake_case)]
    pub fn filter_by_epb_service(&self, service: Service) -> Self {
        let cdata = self.cdata.iter(); // Componentes

        // 1. Consumos y producciones del servicio, salvo la producción eléctrica
        // Se excluyen los componentes de zona y sistema
        // La electricidad generada se reparte más abajo entre los distintos servicios
        let mut cdata_srv: Vec<_> = cdata
            .clone()
            .filter(|c| {
                (c.is_used() && c.has_service(service)) || (c.is_generated() && !c.is_electricity())
            })
            .cloned()
            .collect();

        // 2. Producción eléctrica
        let E_pr_el_t = cdata
            .clone()
            .filter(|c| c.is_electricity() && c.is_generated());
        let E_pr_el_an: f32 = E_pr_el_t.clone().flat_map(|c| c.values().iter()).sum();

        // 3. Reparto de la producción electrica en proporción al consumo de usos EPB
        // Energía eléctrica consumida en usos EPB
        let E_EPus_el_t = cdata
            .clone()
            .filter(|c| c.is_electricity() && c.is_epb_use());

        // Energía eléctrica consumida en el servicio srv
        let E_srv_el_t = E_EPus_el_t.clone().filter(|c| c.has_service(service));
        let E_srv_el_an: f32 = E_srv_el_t.clone().flat_map(|c| c.values().iter()).sum();

        // Si hay consumo y producción de electricidad, se reparte el consumo
        if E_srv_el_an > 0.0 && E_pr_el_an > 0.0 {
            // Pasos de cálculo. Sabemos que cdata.len() > 1 porque si no no se podría cumplir el condicional
            let num_steps = self.cdata[0].num_steps();

            // Energía eléctrica consumida en usos EPB
            let E_EPus_el_t_tot = E_EPus_el_t
                .clone()
                .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, e.values()));

            // Fracción del consumo EPB que representa el servicio srv
            let E_srv_el_t_tot = E_srv_el_t
                .clone()
                .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, e.values()));
            let f_srv_t = E_srv_el_t_tot
                .iter()
                .zip(&E_EPus_el_t_tot)
                .map(|(v, t)| if v.abs() < f32::EPSILON { 0.0 } else { v / t })
                .collect::<Vec<_>>();

            // Repartimos la producción eléctrica

            // Energía eléctrica producida y consumida en usos EPB, corregida por f_match_t
            let f_match_t = vec![1.0; num_steps]; // TODO: implementar f_match_t
            let E_pr_el_t_tot = E_pr_el_t
                .clone()
                .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, e.values()));
            let E_pr_el_used_EPus_t =
                vecvecmul(&f_match_t, &vecvecmin(&E_EPus_el_t_tot, &E_pr_el_t_tot));

            // Para cada producción de electricidad i
            // Repartimos la electricidad generada en la parte que corresponde al servicio
            // ya que la habíamos excluido en el filtrado incial
            for mut E_pr_el_i in E_pr_el_t.cloned() {
                let pr_component = match E_pr_el_i {
                    EnergyData::GenProd(ref mut c) => c,
                    _ => continue,
                };

                // Fracción de la producción total que corresponde al generador i
                let f_pr_el_i: f32 = pr_component.values_sum() / E_pr_el_an;

                // Reparto proporcional a la producción del generador i y al consumo del servicio srv
                pr_component.values = E_pr_el_used_EPus_t
                    .iter()
                    .zip(&f_srv_t)
                    .map(|(v, f_srv)| v * f_pr_el_i * f_srv)
                    .collect();
                pr_component.comment = format!(
                    "{} Producción eléctrica reasignada al servicio",
                    pr_component.comment
                );

                cdata_srv.push(E_pr_el_i);
            }
        }

        let cmeta = self.cmeta.clone();
        let mut newcomponents = Self {
            cmeta,
            cdata: cdata_srv,
            zones: self.zones.clone(),
            systems: self.systems.clone(),
        };
        newcomponents.set_meta("CTE_SERVICIO", &service.to_string());

        newcomponents
    }

    /// Asegura que la energía MEDIOAMBIENTE consumida está equilibrada por una producción in situ
    ///
    /// Completa el balance de las producciones in situ de energía procedente del medioambiente
    /// cuando el consumo de esos vectores supera la producción.
    /// Evita tener que declarar las producciones de MEDIOAMBIENTE, basta con los consumos.
    /// La compensación se hace sistema a sistema y servicio a servicio, sin trasvases de producción entre sistemas.
    ///
    /// Esto significa que, para cada sistema (j=id) y servicio a servicio:
    /// 1) se calculan las cantidades descompensadas
    /// 2) se reparte la producción existente para ese sistema
    /// 3) se genera una producción que completa las cantidades no satisfechas para el sistema
    ///
    /// Las producciones declaradas para un sistema, que no se consuman, no se trasvasan a otros.
    fn compensate_env_use(&mut self) {
        // Localiza componentes de energía procedente del medioambiente
        let envcomps: Vec<_> = self
            .cdata
            .iter()
            .cloned()
            .filter(|c| c.has_carrier(Carrier::MEDIOAMBIENTE))
            .collect();

        // Componentes de MEDIOAMBIENTE que vamos a añadir
        let mut balancecomps = Vec::new();

        let ids: HashSet<_> = envcomps.iter().map(|c| c.id()).collect();

        // Identifica servicios en componentes de consumo
        let services: HashSet<_> = envcomps
            .iter()
            .filter(|c| c.is_used())
            .map(|c| c.service())
            .collect();

        for id in ids {
            // Componentes para el sistema dado
            let components_for_id = envcomps.iter().filter(|c| c.has_id(id));

            for service in &services {
                // Componentes de consumo del servicio
                let consumed: Vec<_> = components_for_id
                    .clone()
                    .filter(|c| c.is_used() && c.has_service(*service))
                    .collect();
                // Si no hay consumo que compensar con producción retornamos None
                if consumed.is_empty() {
                    continue;
                };

                // Consumos no compensados con producción
                let mut unbalanced_values =
                    veclistsum(&consumed.iter().map(|&v| v.values()).collect::<Vec<_>>());

                // Componentes de producción del servicio
                let produced: Vec<_> = components_for_id
                    .clone()
                    .filter(|c| c.is_generated())
                    .collect();
                // Descontamos la producción existente de los consumos
                if !produced.is_empty() {
                    let totproduced =
                        veclistsum(&produced.iter().map(|&v| v.values()).collect::<Vec<_>>());
                    unbalanced_values = vecvecdif(&unbalanced_values, &totproduced)
                        .iter()
                        .map(|&v| if v > 0.0 { v } else { 0.0 })
                        .collect();
                }
                // Si no hay desequilibrio continuamos
                if unbalanced_values.iter().sum::<f32>() == 0.0 {
                    continue;
                };

                // Si hay desequilibrio agregamos un componente de producción
                balancecomps.push(EnergyData::GenProd(GenProd {
                    id,
                    carrier: Carrier::MEDIOAMBIENTE,
                    source: Source::INSITU,
                    values: unbalanced_values,
                    comment: "Equilibrado de consumo sin producción declarada".into(),
                }));
            }
        }

        // Agrega componentes no compensados
        self.cdata.append(&mut balancecomps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TCOMPS1: &str = "#META CTE_AREAREF: 100.5
0, ELECTRICIDAD, PRODUCCION, INSITU, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56
0, ELECTRICIDAD, CONSUMO, REF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, ELECTRICIDAD, CONSUMO, CAL, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, MEDIOAMBIENTE, CONSUMO, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11";

    // Se han puesto las producciones eléctricas a servicio NDEF y compensado consumos de MEDIOAMBIENTE
    const TCOMPSRES1: &str = "#META CTE_AREAREF: 100.5
0, ELECTRICIDAD, PRODUCCION, INSITU, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56
0, ELECTRICIDAD, CONSUMO, REF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, ELECTRICIDAD, CONSUMO, CAL, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, MEDIOAMBIENTE, CONSUMO, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11
0, MEDIOAMBIENTE, PRODUCCION, INSITU, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11 # Equilibrado de consumo sin producción declarada";

    // La producción se debe repartir al 50% entre los usos EPB
    const TCOMPSRES2: &str = "#META CTE_AREAREF: 100.5
#META CTE_SERVICIO: CAL
0, ELECTRICIDAD, CONSUMO, CAL, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, MEDIOAMBIENTE, CONSUMO, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11
0, MEDIOAMBIENTE, PRODUCCION, INSITU, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11 # Equilibrado de consumo sin producción declarada
0, ELECTRICIDAD, PRODUCCION, INSITU, 4.10, 3.28, 2.05, 1.85, 1.02, 1.23, 1.64, 1.43, 1.02, 1.64, 2.46, 3.28 #  Producción eléctrica reasignada al servicio";

    // La producción se debe repartir al 50% entre los usos EPB y sin excesos
    const TCOMPS2: &str = "#META CTE_AREAREF: 1.0
0, ELECTRICIDAD, PRODUCCION, INSITU, 2.00, 6.00, 2.00
0, ELECTRICIDAD, CONSUMO, REF, 1.00, 1.00, 1.00
0, ELECTRICIDAD, CONSUMO, CAL, 1.00, 2.00, 1.00
0, MEDIOAMBIENTE, CONSUMO, CAL, 2.00, 2.00, 2.00";

    const TCOMPSRES3: &str = "#META CTE_AREAREF: 1.0
#META CTE_SERVICIO: CAL
0, ELECTRICIDAD, CONSUMO, CAL, 1.00, 2.00, 1.00
0, MEDIOAMBIENTE, CONSUMO, CAL, 2.00, 2.00, 2.00
0, MEDIOAMBIENTE, PRODUCCION, INSITU, 2.00, 2.00, 2.00 # Equilibrado de consumo sin producción declarada
0, ELECTRICIDAD, PRODUCCION, INSITU, 1.00, 2.00, 1.00 #  Producción eléctrica reasignada al servicio";

    #[test]
    fn tcomponents_parse() {
        let tcomps = TCOMPS1.parse::<Components>().unwrap();
        // roundtrip building from/to string
        assert_eq!(tcomps.to_string(), TCOMPS1);
    }

    #[test]
    fn tcomponents_normalize() {
        let tcompsnorm = TCOMPS1.parse::<Components>().unwrap().normalize();
        assert_eq!(tcompsnorm.to_string(), TCOMPSRES1);
    }

    #[test]
    fn tcomponents_filter_by_epb_service() {
        let tcompsnormfilt = TCOMPS1
            .parse::<Components>()
            .unwrap()
            .normalize()
            .filter_by_epb_service(Service::CAL);
        assert_eq!(tcompsnormfilt.to_string(), TCOMPSRES2);
    }

    #[test]
    fn tcomponents_filter_by_epb_service_prod_excess() {
        let tcompsnormfilt = TCOMPS2
            .parse::<Components>()
            .unwrap()
            .normalize()
            .filter_by_epb_service(Service::CAL);
        assert_eq!(tcompsnormfilt.to_string(), TCOMPSRES3);
    }

    /// Componentes con id de sistema diferenciados
    /// e imputación de producción no compensada de MEDIOAMBIENTE a los id correspondientes
    #[test]
    fn normalize() {
        let comps = "# Bomba de calor 1
            1,ELECTRICIDAD,CONSUMO,ACS,100 # BdC 1
            1,MEDIOAMBIENTE,CONSUMO,ACS,150 # BdC 1
            # Bomba de calor 2
            2,ELECTRICIDAD,CONSUMO,CAL,200 # BdC 2
            2,MEDIOAMBIENTE,CONSUMO,CAL,300 # BdC 2
            # Producción fotovoltaica in situ
            1,ELECTRICIDAD,PRODUCCION,INSITU,50 # PV
            2,ELECTRICIDAD,PRODUCCION,INSITU,100 # PV
            # Producción de energía ambiente dada por el usuario
            0,MEDIOAMBIENTE,PRODUCCION,INSITU,100 # Producción declarada de sistema sin consumo (no reduce energía a compensar)
            1,MEDIOAMBIENTE,PRODUCCION,INSITU,100 # Producción declarada de sistema con consumo (reduce energía a compensar)
            2,MEDIOAMBIENTE,PRODUCCION,INSITU,100 # Producción declarada de sistema sin ese servicio consumo (no reduce energía a compensar)
            # Compensación de energía ambiente a completar por CteEPBD"
            .parse::<Components>()
            .unwrap()
            .normalize();
        let ma_prod = comps
            .cdata
            .iter()
            .filter(|c| c.is_generated() && c.has_carrier(Carrier::MEDIOAMBIENTE));

        // Se añaden 50kWh a los 100kWh declarados para compensar consumo en ACS (150kWh)
        let ma_prod_1: f32 = ma_prod
            .clone()
            .filter(|c| c.has_id(1))
            .map(EnergyData::values_sum)
            .sum();
        assert_eq!(format!("{:.1}", ma_prod_1), "150.0");

        // Se añaden 200kWh a los 100kWh declarados para compensar consumo en CAL (300kWh)
        let ma_prod_2: f32 = ma_prod
            .clone()
            .filter(|c| c.has_id(2))
            .map(EnergyData::values_sum)
            .sum();
        assert_eq!(format!("{:.1}", ma_prod_2), "300.0");
        // En total, se añaden 200 + 50 a los 300kWh declarados, para un total de 550kWh
        // Hay 100kWh declarados para sistema 0 que no se consumen
        let ma_prod_tot: f32 = ma_prod.clone().map(EnergyData::values_sum).sum();
        assert_eq!(format!("{:.1}", ma_prod_tot), "550.0");
    }

    /// Prueba del formato con componentes de zona y sistema para declarar
    /// demanda del edificio y energía entregada o absorbida por los sistemas
    #[test]
    fn tcomponents_extended_parse() {
        "#META CTE_AREAREF: 1.0
            0, ZONA, DEMANDA, REF, -3.0 # Demanda ref. edificio
            0, ZONA, DEMANDA, CAL, 3.0 # Demanda cal. edificio
            1, GEN, CARGA, REF, -3.0 # Demanda ref. EER 3
            2, GEN, CARGA, CAL, 3.0 # Demanda cal. COP 3
            1, ELECTRICIDAD, PRODUCCION, INSITU, 2.00 # Producción PV
            2, ELECTRICIDAD, CONSUMO, REF, 1.00 # BdC modo refrigeración
            2, ELECTRICIDAD, CONSUMO, CAL, 1.00 # BdC modo calefacción
            2, MEDIOAMBIENTE, CONSUMO, CAL, 2.00 # BdC modo calefacción
            "
        .parse::<Components>()
        .unwrap();
    }
}
