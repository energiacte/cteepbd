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
//            Daniel Jiménez González <dani@ietcc.csic.es>,
//            Marta Sorribes Gil <msorribes@ietcc.csic.es>

/*! 
Energy Components (CTE)
=======================

Manejo de componentes energéticos (consumos o producciones de energía) para el CTE

Utilidades para la gestión de componentes energéticos para el CTE

Hipótesis:
- Se completa automáticamente el consumo de energía procedente del medioambiente con una producción
- No se permite la producción de electricidad a usos concretos (se asume NDEF)
*/

use itertools::Itertools;

use crate::vecops::{veckmul, veclistsum, vecvecdif};
use crate::{CSubtype, CType, Carrier, Component, Components, EpbdError, MetaVec, Service};

/// Devuelve objetos CARRIER y META a partir de cadena, intentando asegurar los tipos.
pub fn parse_components(datastring: &str) -> Result<Components, EpbdError> {
    let mut components: Components = datastring.parse()?;
    fix_components(&mut components);
    Ok(components)
}

/// Corrige los componentes de consumo y producción
///
/// - Asegura que la energía MEDIOAMBIENTE consumida tiene su producción correspondiente
/// - Asegura que la energía eléctrica producida no tiene un uso que no sea NDEF
///
/// Los metadatos, servicios y coherencia de los vectores se aseguran ya en el parsing
pub fn fix_components(components: &mut Components) {
    force_ndef_use_for_electricity_production(components);
    compensate_env_use(components);
}

/// Asegura que la energía eléctrica producida no tiene un uso que no sea NDEF
///
/// Esta restricción es propia de la implementación y de cómo hace el reparto de la producción,
/// solamente en base al consumo de cada servicio y sin tener en cuenta si se define un destino
///XXX: *Esta restricción debería eliminarse*
pub fn force_ndef_use_for_electricity_production(components: &mut Components) {
    // Localiza componentes de energía procedente del medioambiente
    for component in &mut components.cdata {
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
pub fn compensate_env_use(components: &mut Components) {
    // Localiza componentes de energía procedente del medioambiente
    let envcomps: Vec<_> = components
        .cdata
        .iter()
        .cloned()
        .filter(|c| c.carrier == Carrier::MEDIOAMBIENTE)
        .collect();
    // Identifica servicios
    let services: Vec<_> = envcomps.iter().map(|c| c.service).unique().collect();

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
    components.cdata.append(&mut balancecomps);
}

// Funcionalidad para generar RER para ACS en perímetro nearby -------------------------

/// Selecciona subconjunto de componentes relacionados con el servicio indicado.
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
    // proporcionalmente al consumo de elec. del servicio respecto al de todos los servicios
    let pr_el_ndef: Vec<_> = components
        .cdata
        .iter()
        .filter(|c| {
            c.carrier == Carrier::ELECTRICIDAD
                && c.ctype == CType::PRODUCCION
                && c.csubtype == CSubtype::INSITU
                && c.service == Service::NDEF
        })
        .collect();

    if !pr_el_ndef.is_empty() {
        let c_el = components
            .cdata
            .iter()
            .filter(|c| c.carrier == Carrier::ELECTRICIDAD && c.ctype == CType::CONSUMO);
        let c_el_tot = c_el
            .clone()
            .map(|c| c.values.iter().sum::<f32>())
            .sum::<f32>();
        let c_el_srv_tot = c_el
            .clone()
            .filter(|c| c.service == service)
            .map(|c| c.values.iter().sum::<f32>())
            .sum::<f32>();

        if c_el_tot > 0.0 && c_el_srv_tot > 0.0 {
            let fraction_pr_srv = c_el_srv_tot / c_el_tot;
            for c in &pr_el_ndef {
                cdata.push(Component {
                    carrier: Carrier::ELECTRICIDAD,
                    ctype: CType::PRODUCCION,
                    csubtype: CSubtype::INSITU,
                    service,
                    values: veckmul(&c.values, fraction_pr_srv),
                    comment: format!(
                        "{} Producción insitu proporcionalmente reasignada al servicio.",
                        c.comment
                    ),
                })
            }
        }
    }

    let cmeta = components.cmeta.clone();

    let mut newcomponents = Components { cdata, cmeta };

    newcomponents.update_meta("CTE_PERIMETRO", "NEARBY");
    newcomponents.update_meta("CTE_SERVICIO", &service.to_string());

    newcomponents
}
