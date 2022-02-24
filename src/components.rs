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

- Se completa automáticamente el consumo de energía procedente del medioambiente o termosolar con una producción
- El reparto de la electricidad generada es proporcional a los consumos eléctricos
*/

use std::{
    collections::{HashMap, HashSet},
    fmt, str,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::EpbdError,
    types::{
        BuildingNeeds, Carrier, EProd, Energy, HasValues, Meta, MetaVec, ProdSource, Service,
        ZoneNeeds,
    },
    vecops::{veclistsum, vecvecdif, vecvecsum},
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
    /// EUsed or produced energy data
    pub cdata: Vec<Energy>,
    /// Building data (energy needs, ...)
    pub building: Vec<BuildingNeeds>,
    /// Zone data (energy needs, ...)
    pub zones: Vec<ZoneNeeds>,
    // System data
    // pub systems: Vec<SystemData>,
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
        let mut building = Vec::new();
        // let mut systems = None;

        // Tipos disponibles
        let ctypes_tag_list = [
            "CONSUMO",
            "PRODUCCION",
            "AUX",
            "SALIDA",
            "EDIFICIO",
            "ZONA",
            "SISTEMA",
        ];

        for line in datalines {
            let tags: Vec<_> = line.splitn(3, ',').map(str::trim).take(2).collect();
            let tag1 = tags.get(0).unwrap_or(&"");
            let tag2 = tags.get(1).unwrap_or(&"");
            let tag = if ctypes_tag_list.contains(tag1) {
                tag1
            } else {
                tag2
            };
            match *tag {
                "CONSUMO" => cdata.push(Energy::Used(line.parse()?)),
                "PRODUCCION" => cdata.push(Energy::Prod(line.parse()?)),
                "AUX" => cdata.push(Energy::Aux(line.parse()?)),
                "SALIDA" => cdata.push(Energy::Out(line.parse()?)),
                "EDIFICIO" => building.push(line.parse()?),
                "ZONA" => zones.push(line.parse()?),
                "SISTEMA" => unimplemented!(),
                _ => {
                    return Err(EpbdError::ParseError(format!(
                        "ERROR: No se reconoce el componente de la línea: {} {}",
                        line, tag
                    )))
                }
            }
        }

        // Check that all used or produced energy components have an equal number of steps (data lengths)
        // TODO: Additional checks
        // - Move to check_components
        // - There are, at most, 3 building needs definitions (CAL, REF, ACS)
        // - Q_out (SALIDA) services include, at least, those included in E_in (CONSUMO). Think about interactive building of components and transient states
        // - AUX components for systems with more than 1 service output need Q_out (SALIDA) components
        {
            let cdata_lengths: Vec<_> = cdata.iter().map(|e| e.num_steps()).collect();
            let start_num_steps = *cdata_lengths.get(0).unwrap_or(&12);
            if cdata_lengths.iter().any(|&clen| clen != start_num_steps) {
                return Err(EpbdError::ParseError(
                    "Componentes con distinto número de pasos de cálculo".into(),
                ));
            }
        }

        Components {
            cmeta,
            cdata,
            building,
            zones,
        }
        .normalize()
    }
}

impl Components {
    /// Number of steps of the first component
    pub fn num_steps(&self) -> usize {
        self.cdata.get(0).map(|v| v.num_steps()).unwrap_or(0)
    }

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
    /// - Asegura que la energía EAMBIENTE consumida tiene su producción correspondiente
    /// - Asegura que la energía TERMOSOLAR consumida tiene su producción correspondiente
    /// - Reparte los consumos auxliares proporcionalmente a los servicios
    ///
    /// Los metadatos, servicios y coherencia de los vectores se aseguran ya en el parsing
    pub fn normalize(mut self) -> Result<Self, EpbdError> {
        // Compensa consumos no respaldados por producción
        self.complete_produced_for_onsite_generated_use(Carrier::EAMBIENTE);
        self.complete_produced_for_onsite_generated_use(Carrier::TERMOSOLAR);
        self.assign_aux_nepb_to_epb_services()?;
        self.sort_by_id();
        Ok(self)
    }

    /// Compensa los consumos declarados de energía insitu no equilibrada por producción
    ///
    /// Afecta a los vectores EAMBIENTE y TERMOSOLAR
    ///
    /// cuando el consumo de esos vectores supera la producción.
    /// Evita tener que declarar las producciones de EAMBIENTE y TERMOSOLAR, basta con los consumos.
    /// La compensación se hace sistema a sistema, sin trasvases de producción entre sistemas.
    ///
    /// Esto significa que, para cada sistema (j=id):
    /// 1) se calcula el consumo del vector en todos los servicios
    /// 2) se calculan las cantidades produccidas del vector
    /// 2) se reparte la producción existente para ese sistema
    /// 3) se genera una producción que completa las cantidades no cubiertas por la producción definida
    ///
    /// Las producciones declaradas para un sistema, que no se consuman, no se trasvasan a otros.
    fn complete_produced_for_onsite_generated_use(&mut self, carrier: Carrier) {
        let source = match carrier {
            Carrier::EAMBIENTE => ProdSource::EAMBIENTE,
            Carrier::TERMOSOLAR => ProdSource::TERMOSOLAR,
            _ => {
                panic!("Intento de compensación de vector distinto de EAMBIENTE o TERMOSOLAR")
            }
        };

        // Localiza componentes pertenecientes al vector
        let envcomps: Vec<_> = self
            .cdata
            .iter()
            .cloned()
            .filter(|c| c.has_carrier(carrier))
            .collect();
        if envcomps.is_empty() {
            return;
        };

        let ids: HashSet<_> = envcomps.iter().map(|c| c.id()).collect();
        for id in ids {
            // Componentes para el sistema dado
            let components_for_id = envcomps.iter().filter(|c| c.has_id(id));
            // Componentes de producción del servicio
            let prod: Vec<_> = components_for_id
                .clone()
                .filter(|c| c.is_generated())
                .collect();

            // Componentes de consumo
            let used: Vec<_> = components_for_id.clone().filter(|c| c.is_used()).collect();
            // Si no hay consumo que compensar con producción retornamos None
            if used.is_empty() {
                continue;
            };
            // Consumos no compensados con producción
            let total_use = veclistsum(&used.iter().map(|&v| v.values()).collect::<Vec<_>>());

            // Usos no compensados con la producción existente
            let unbalanced_use = if prod.is_empty() {
                total_use
            } else {
                let avail_prod = veclistsum(&prod.iter().map(|&v| v.values()).collect::<Vec<_>>());
                vecvecdif(&total_use, &avail_prod)
                    .iter()
                    .map(|&v| if v > 0.0 { v } else { 0.0 })
                    .collect()
            };

            // Si no hay desequilibrio continuamos
            if unbalanced_use.iter().sum::<f32>() == 0.0 {
                continue;
            };

            // Si hay desequilibrio agregamos un componente de producción
            self.cdata.push(Energy::Prod(EProd {
                id,
                source,
                values: unbalanced_use,
                comment: "Equilibrado de consumo sin producción declarada".into(),
            }));
        }
    }

    /// Asigna servicios EPB a los componentes de energía auxiliar
    ///
    /// Los componentes de consumos auxiliares se cargan incialmente con el servicio NEPB
    /// pero representan solo servicios EPB y debemos asignarlos.
    ///
    /// Para hacer esta asignación se actúa sistema a sistema:
    /// 1) si solamente hay un servicio EPB se asigna el consumo Aux a ese servicio
    /// 2) si hay más de un servicio EPB se genera un consumo Aux para cada servicio
    ///    disponible y se asigna a cada servicio un consumo proporcional
    ///    a la energía saliente de cada servicio en relación a la total saliente
    ///    para todos los servicios EPB.
    fn assign_aux_nepb_to_epb_services(&mut self) -> Result<(), EpbdError> {
        // ids with aux energy use
        let ids: HashSet<_> = self
            .cdata
            .iter()
            .filter(|c| c.is_aux())
            .map(Energy::id)
            .collect();
        for id in ids {
            let services_for_uses_with_id = self
                .cdata
                .iter()
                .filter_map(|c| match c {
                    Energy::Used(e) if e.id == id => Some(e.service),
                    _ => None,
                })
                .collect::<HashSet<_>>();

            // Con un solo servicio en los consumos usamos ese para los auxiliares
            // sin necesidad de consultar la energía entregada o absorbida
            if services_for_uses_with_id.len() == 1 {
                let service = *services_for_uses_with_id.iter().next().unwrap();
                for c in &mut self.cdata {
                    if let Energy::Aux(e) = c {
                        if e.id == id {
                            e.service = service
                        }
                    }
                }
                continue;
            }

            // Con más de un servicio necesitamos repartir la energía auxiliar de forma proporcional
            // a la energía saliente de cada servicio en relación al total de servicios EPB
            let aux_tot = veclistsum(
                &self
                    .cdata
                    .iter()
                    .filter_map(|c| match c {
                        Energy::Aux(e) if e.id == id => Some(e.values()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            );

            let mut q_out_by_srv: HashMap<Service, Vec<f32>> = HashMap::new();
            for component in &self.cdata {
                if let Energy::Out(e) = component {
                    if e.id == id {
                        q_out_by_srv
                            .entry(e.service)
                            .or_insert_with(|| vec![0.0; self.num_steps()]);
                        q_out_by_srv
                            .insert(e.service, vecvecsum(&q_out_by_srv[&e.service], &e.values));
                    }
                };
            }

            let mut q_out_tot = vec![0.0; self.num_steps()];
            for q_out in q_out_by_srv.values() {
                q_out_tot = vecvecsum(&*q_out_tot, q_out);
            }

            if aux_tot.iter().sum::<f32>() > 0.0 && q_out_tot.iter().sum::<f32>() == 0.0 {
                return Err(EpbdError::WrongInput(format!("Sin datos de energía saliente para hacer el reparto de los consumos auxiliares del sistema {}", id)));
            };

            // Calculamos la fracción de cada servicio sobre el total
            let mut q_out_frac_by_srv = q_out_by_srv;
            let out_services: Vec<Service> = q_out_frac_by_srv.keys().cloned().collect();
            for service in &out_services {
                let values = q_out_frac_by_srv[service]
                    .iter()
                    .zip(q_out_tot.iter())
                    .map(|(val, tot)| if tot > &0.0 { val / tot } else { 0.0 })
                    .collect();
                q_out_frac_by_srv.insert(*service, values);
            }

            // Elimina componentes de auxiliares existentes
            self.cdata.retain(|c| !c.is_aux());

            // Incorpora nuevos auxiliares con reparto calculado por servicios
            for service in &out_services {
                let values = q_out_frac_by_srv[service]
                    .iter()
                    .zip(aux_tot.iter())
                    .map(|(q_out_frac, aux_tot_i)| q_out_frac * aux_tot_i)
                    .collect();
                self.cdata.push(Energy::Aux(crate::types::EAux {
                    id,
                    service: *service,
                    values,
                    comment: "Reasignación automática de consumos auxiliares".into(),
                }));
            }
        }
        Ok(())
    }

    /// Ordena componentes según el id del sistema
    fn sort_by_id(&mut self) {
        self.cdata.sort_by_key(|e| e.id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TCOMPS1: &str = "#META CTE_AREAREF: 100.5
0, PRODUCCION, EL_INSITU, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56
0, CONSUMO, REF, ELECTRICIDAD, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, CONSUMO, CAL, ELECTRICIDAD, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, CONSUMO, CAL, EAMBIENTE, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11
0, PRODUCCION, EAMBIENTE, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11 # Equilibrado de consumo sin producción declarada";

    // Reparto de producciones eléctricas y compensación de consumos de EAMBIENTE
    const TCOMPSRES1: &str = "#META CTE_AREAREF: 100.5
0, PRODUCCION, EL_INSITU, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56
0, CONSUMO, REF, ELECTRICIDAD, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, CONSUMO, CAL, ELECTRICIDAD, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
0, CONSUMO, CAL, EAMBIENTE, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11
0, PRODUCCION, EAMBIENTE, 6.39, 3.11, 8.20, 17.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 3.11 # Equilibrado de consumo sin producción declarada";

    #[test]
    fn tcomponents_parse() {
        let tcomps = TCOMPS1.parse::<Components>().unwrap();
        // roundtrip building from/to string
        assert_eq!(tcomps.to_string(), TCOMPS1);
    }

    #[test]
    fn tcomponents_normalize() {
        let tcompsnorm = TCOMPS1.parse::<Components>().unwrap();
        assert_eq!(tcompsnorm.to_string(), TCOMPSRES1);
    }

    /// Componentes con id de sistema diferenciados
    /// e imputación de producción no compensada de EAMBIENTE a los id correspondientes
    #[test]
    fn check_normalized_components() {
        let comps = "# Bomba de calor 1
            1,CONSUMO,ACS,ELECTRICIDAD,100 # BdC 1
            1,CONSUMO,ACS,EAMBIENTE,150 # BdC 1
            # Bomba de calor 2
            2,CONSUMO,CAL,ELECTRICIDAD,200 # BdC 2
            2,CONSUMO,CAL,EAMBIENTE,300 # BdC 2
            # Producción fotovoltaica in situ
            1,PRODUCCION,EL_INSITU,50 # PV
            2,PRODUCCION,EL_INSITU,100 # PV
            # Producción de energía ambiente dada por el usuario
            0,PRODUCCION,EAMBIENTE,100 # Producción declarada de sistema sin consumo (no reduce energía a compensar)
            1,PRODUCCION,EAMBIENTE,100 # Producción declarada de sistema con consumo (reduce energía a compensar)
            2,PRODUCCION,EAMBIENTE,100 # Producción declarada de sistema sin ese servicio consumo (no reduce energía a compensar)
            # Compensación de energía ambiente a completar por CteEPBD"
            .parse::<Components>()
            .unwrap();
        let ma_prod = comps
            .cdata
            .iter()
            .filter(|c| c.is_generated() && c.has_carrier(Carrier::EAMBIENTE));

        // Se añaden 50kWh a los 100kWh declarados para compensar consumo en ACS (150kWh)
        let ma_prod_1: f32 = ma_prod
            .clone()
            .filter(|c| c.has_id(1))
            .map(Energy::values_sum)
            .sum();
        assert_eq!(format!("{:.1}", ma_prod_1), "150.0");

        // Se añaden 200kWh a los 100kWh declarados para compensar consumo en CAL (300kWh)
        let ma_prod_2: f32 = ma_prod
            .clone()
            .filter(|c| c.has_id(2))
            .map(Energy::values_sum)
            .sum();
        assert_eq!(format!("{:.1}", ma_prod_2), "300.0");
        // En total, se añaden 200 + 50 a los 300kWh declarados, para un total de 550kWh
        // Hay 100kWh declarados para sistema 0 que no se consumen
        let ma_prod_tot: f32 = ma_prod.clone().map(Energy::values_sum).sum();
        assert_eq!(format!("{:.1}", ma_prod_tot), "550.0");
    }

    /// Prueba del formato con componentes de zona y sistema para declarar
    /// demanda del edificio y energía entregada o absorbida por los sistemas
    #[test]
    fn tcomponents_extended_parse() {
        "#META CTE_AREAREF: 1.0
            0, ZONA, DEMANDA, REF, -3.0 # Demanda ref. edificio
            0, ZONA, DEMANDA, CAL, 3.0 # Demanda cal. edificio
            1, PRODUCCION, EL_INSITU, 2.00 # Producción PV
            2, CONSUMO, CAL, ELECTRICIDAD, 1.00 # BdC modo calefacción
            2, CONSUMO, CAL, EAMBIENTE, 2.00 # BdC modo calefacción
            2, SALIDA, CAL, 3.0 # Energía entregada por el equipo de calefacción con COP 3
            2, CONSUMO, ACS, ELECTRICIDAD, 1.0 # BdC modo ACS
            2, CONSUMO, ACS, EAMBIENTE, 2.0 # BdC modo ACS
            2, SALIDA, ACS, 3.0 # Energía entregada por el equipo de acs con COP_dhw 3
            2, AUX, 0.5 # Auxiliares ACS BdC
            3, CONSUMO, REF, ELECTRICIDAD, 1.00 # BdC modo refrigeración
            3, SALIDA, REF, -3.0 # Energía absorbida por el equipo de refrigeración con EER 3
            "
        .parse::<Components>()
        .unwrap();
    }
}
