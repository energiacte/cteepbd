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
Componentes energéticos
=======================

Define el tipo Components (lista de componentes + metadatos) y sus traits.

Los componentes modelizan el uso y producción de energía en el periodo de cálculo.

Hipótesis:

- Se completa automáticamente el consumo de energía procedente del medioambiente con una producción
- No se permite la producción de electricidad a usos concretos (se asume NDEF) (XXX: se podría eliminar)
*/

use std::collections::HashSet;
use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use crate::{
    error::EpbdError,
    types::{CSubtype, CType, Carrier, Component, Meta, MetaVec, Service},
    vecops::{veckmul, veclistsum, vecvecdif, vecvecmin, vecvecmul, vecvecsum},
};

/// Lista de datos de componentes con sus metadatos
///
/// List of component data bundled with its metadata
///
/// #META CTE_AREAREF: 100.5
/// ELECTRICIDAD,CONSUMO,EPB,16.39,13.11,8.20,7.38,4.10,4.92,6.56,5.74,4.10,6.56,9.84,13.11
/// ELECTRICIDAD,PRODUCCION,INSITU,8.20,6.56,4.10,3.69,2.05,2.46,3.28,2.87,2.05,3.28,4.92,6.56
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Component list
    pub cmeta: Vec<Meta>,
    /// Metadata
    pub cdata: Vec<Component>,
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
        let s_nobom = if s.starts_with("\u{feff}") {
            &s[3..]
        } else {
            s
        };
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
        let cdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Component>, _>>()?;
        {
            let cdata_lens: Vec<_> = cdata.iter().map(|e| e.values.len()).collect();
            if cdata_lens.iter().max().unwrap() != cdata_lens.iter().min().unwrap() {
                return Err(EpbdError::ParseError(s.into()));
            }
        }
        Ok(Components { cmeta, cdata })
    }
}

impl Components {
    /// Corrige los componentes de consumo y producción
    ///
    /// - Asegura que la energía MEDIOAMBIENTE consumida tiene su producción correspondiente
    /// - Asegura que la energía eléctrica producida no tiene un uso que no sea NDEF
    ///
    /// Los metadatos, servicios y coherencia de los vectores se aseguran ya en el parsing
    pub fn normalize(mut self) -> Self {
        self.force_ndef_use_for_electricity_production();
        self.compensate_env_use();
        self
    }

    /// Filtra Componentes relacionados con un servicio EPB
    ///
    /// 1. Se seleccionan todos los consumos y producciones asignados al servicio
    /// 2. Se toman las producciones eléctricas
    /// 3. Reparto de las producciones eléctricas en proporción al consumo del servicio respecto al consumo EPB
    ///
    /// *Nota*: los componentes deben estar normalizados (ver método normalize) para asegurar que:
    /// - los consumos de MEDIOAMBIENTE de un servicio ya están equilibrados
    /// - las producciones eléctricas no pueden ser asignadas a un servicio
    #[allow(non_snake_case)]
    pub fn filter_by_epb_service(&self, service: Service) -> Self {
        let num_steps = self.cdata[0].values.len(); // Pasos de cálculo
        let cdata = self.cdata.iter(); // Componentes

        // 1. Consumos y producciones del servicio, salvo la producción eléctrica
        let mut cdata_srv: Vec<_> = cdata
            .clone()
            .filter(|c| {
                c.service == service
                    && !(c.carrier == Carrier::ELECTRICIDAD && c.ctype == CType::PRODUCCION)
            })
            .cloned()
            .collect();

        // 2. Producción eléctrica
        let E_pr_el_t = cdata
            .clone()
            .filter(|c| c.carrier == Carrier::ELECTRICIDAD && c.ctype == CType::PRODUCCION);
        let E_pr_el_an: f32 = E_pr_el_t.clone().flat_map(|c| c.values.iter()).sum();

        // 3. Reparto de la producción electrica en proporción al consumo de usos EPB
        // Energía eléctrica consumida en usos EPB
        let E_EPus_el_t = cdata.clone().filter(|c| {
            c.carrier == Carrier::ELECTRICIDAD
                && c.ctype == CType::CONSUMO
                && c.csubtype == CSubtype::EPB
        });

        // Energía eléctrica consumida en el servicio srv
        let E_srv_el_an: f32 = E_EPus_el_t
            .clone()
            .filter(|c| c.service == service)
            .flat_map(|c| c.values.iter())
            .sum();

        // Si hay consumo y producción de electricidad, se reparte el consumo
        if E_srv_el_an > 0.0 && E_pr_el_an > 0.0 {
            // Energía eléctrica consumida en usos EPB
            let E_EPus_el_t_tot = E_EPus_el_t
                .clone()
                .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, &e.values));
            let E_pr_el_t_tot = E_pr_el_t
                .clone()
                .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, &e.values));

            // Energía eléctrica producida y consumida en usos EPB
            // let f_match_t = vec![1.0; num_steps]; // TODO: implementar f_match_t
            
            // let E_pr_el_used_EPus_t =
            //     vecvecmul(&f_match_t, &vecvecmin(&E_EPus_el_t_tot, &E_pr_el_t_tot));

            // Fracción del consumo EPB que representa el servicio srv
            // FIXME: esto debe ser paso a paso y no anual
            let f_srv: f32 = E_srv_el_an / E_EPus_el_t_tot.iter().sum::<f32>();

            // Repartimos la producción eléctrica proporcionalemente
            // FIXME: Aquí podría haber un exceso de producción, por encima del consumo. Ver.
            for mut E_pr_el_i in E_pr_el_t.cloned() {
                // Fracción de la producción total que corresponde al generador i
                let f_pr_el_i: f32 = E_pr_el_i.values.iter().sum::<f32>() / E_pr_el_an;

                E_pr_el_i.values = veckmul(&E_pr_el_i.values, f_pr_el_i * f_srv);
                E_pr_el_i.service = service;
                E_pr_el_i.comment =
                    format!("{} Producción eléctrica reasignada al servicio", E_pr_el_i.comment);
                cdata_srv.push(E_pr_el_i);
            }
        }

        let cmeta = self.cmeta.clone();
        let mut newcomponents = Self {
            cdata: cdata_srv,
            cmeta,
        };
        newcomponents.set_meta("CTE_SERVICIO", &service.to_string());

        newcomponents
    }

    /// Asegura que la energía eléctrica producida no tiene un uso que no sea NDEF
    ///
    /// Esta restricción es propia de la implementación y de cómo hace el reparto de la producción,
    /// solamente en base al consumo de cada servicio y sin tener en cuenta si se define un destino
    ///XXX: *Esta restricción debería eliminarse*
    fn force_ndef_use_for_electricity_production(&mut self) {
        // Localiza componentes de energía procedente del medioambiente
        for component in &mut self.cdata {
            if component.carrier == Carrier::ELECTRICIDAD && component.ctype == CType::PRODUCCION {
                component.service = Service::NDEF
            }
        }
    }

    /// Asegura que la energía MEDIOAMBIENTE consumida está equilibrada por una producción in situ
    ///
    /// Completa el balance de las producciones in situ de energía procedente del medioambiente
    /// cuando el consumo de esos vectores supera la producción. Es solamente una comodidad, para no
    /// tener que declarar las producciones de MEDIOAMBIENTE, solo los consumos.
    fn compensate_env_use(&mut self) {
        // Localiza componentes de energía procedente del medioambiente
        let envcomps: Vec<_> = self
            .cdata
            .iter()
            .cloned()
            .filter(|c| c.carrier == Carrier::MEDIOAMBIENTE)
            .collect();
        // Identifica servicios
        let services: HashSet<_> = envcomps.iter().map(|c| c.service).collect();

        // Asegura que la producción eléctrica no tiene un uso definido (es NDEF)

        // Genera componentes de consumo no compensados con producción
        let mut balancecomps: Vec<Component> = services
            .iter()
            .map(|&service| {
                // Componentes para el servicio
                let ecomps = envcomps.iter().filter(|c| c.service == service);
                // Componentes de consumo del servicio
                let consumed: Vec<_> = ecomps
                    .clone()
                    .filter(|c| c.ctype == CType::CONSUMO)
                    .collect();
                // Si no hay consumo que compensar con producción retornamos None
                if consumed.is_empty() {
                    return None;
                };
                // Consumos no compensados con producción
                let mut unbalanced_values = veclistsum(
                    &consumed
                        .iter()
                        .map(|&v| v.values.as_slice())
                        .collect::<Vec<_>>(),
                );
                // Componentes de producción del servicio
                let produced: Vec<_> = ecomps
                    .clone()
                    .filter(|c| c.ctype == CType::PRODUCCION)
                    .collect();
                // Descontamos la producción existente de los consumos
                if !produced.is_empty() {
                    let totproduced = veclistsum(
                        &produced
                            .iter()
                            .map(|&v| v.values.as_slice())
                            .collect::<Vec<_>>(),
                    );
                    unbalanced_values = vecvecdif(&unbalanced_values, &totproduced)
                        .iter()
                        .map(|&v| if v > 0.0 { v } else { 0.0 })
                        .collect();
                }
                // Si no hay desequilibrio retornamos None
                if unbalanced_values.iter().sum::<f32>() == 0.0 {
                    return None;
                };

                // Si hay desequilibrio agregamos un componente de producción
                Some(Component {
                    carrier: Carrier::MEDIOAMBIENTE,
                    ctype: CType::PRODUCCION,
                    csubtype: CSubtype::INSITU,
                    service,
                    values: unbalanced_values,
                    comment: "Equilibrado de consumo sin producción declarada".into(),
                })
            })
            .filter(std::option::Option::is_some)
            .collect::<Option<Vec<_>>>()
            .unwrap_or_else(|| vec![]);
        // Agrega componentes no compensados
        self.cdata.append(&mut balancecomps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TCOMPS1: &str = "#META CTE_AREAREF: 100.5
ELECTRICIDAD, PRODUCCION, INSITU, CAL, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56
ELECTRICIDAD, CONSUMO, EPB, REF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
ELECTRICIDAD, CONSUMO, EPB, CAL, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
MEDIOAMBIENTE, CONSUMO, EPB, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11";

    // Se han puesto las producciones eléctricas a servicio NDEF y compensado consumos de MEDIOAMBIENTE
    const TCOMPSRES1: &str = "#META CTE_AREAREF: 100.5
ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56
ELECTRICIDAD, CONSUMO, EPB, REF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
ELECTRICIDAD, CONSUMO, EPB, CAL, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
MEDIOAMBIENTE, CONSUMO, EPB, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11
MEDIOAMBIENTE, PRODUCCION, INSITU, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11 # Equilibrado de consumo sin producción declarada";

    // La producción se debe repartir al 50% entre los usos EPB
    const TCOMPSRES2: &str = "#META CTE_AREAREF: 100.5
#META CTE_SERVICIO: CAL
ELECTRICIDAD, CONSUMO, EPB, CAL, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
MEDIOAMBIENTE, CONSUMO, EPB, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11
MEDIOAMBIENTE, PRODUCCION, INSITU, CAL, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11 # Equilibrado de consumo sin producción declarada
ELECTRICIDAD, PRODUCCION, INSITU, CAL, 4.10, 3.28, 2.05, 1.85, 1.02, 1.23, 1.64, 1.43, 1.02, 1.64, 2.46, 3.28 #  Producción eléctrica reasignada al servicio";

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
}
