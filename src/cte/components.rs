/*! # Manejo de componentes energéticos (consumos o producciones de energía) para el CTE

Utilidades para la gestión de componentes energéticos para el CTE

*/

use failure::Error;
use itertools::Itertools;

use crate::types::{CSubtype, CType, Carrier, Service, MetaVec};
use crate::types::{Component, Components};
use crate::vecops::{veckmul, veclistsum, vecvecdif};

/// Devuelve objetos CARRIER y META a partir de cadena, intentando asegurar los tipos.
pub fn parse_components(datastring: &str) -> Result<Components, Error> {
    let mut components: Components = datastring.parse()?;
    fix_components(&mut components);
    Ok(components)
}

/// Asegura que la energía MEDIOAMBIENTE consumida está equilibrada por una producción in situ
///
/// Completa el balance de las producciones in situ de energía procedente del medioambiente
/// cuando el consumo de esos vectores supera la producción. Es solamente una comodidad, para no
/// tener que declarar las producciones de MEDIOAMBIENTE, solo los consumos.
///
/// Los metadatos, servicios y coherencia de los vectores se aseguran ya en el parsing
pub fn fix_components(components: &mut Components) {
    // Localiza componentes de energía procedente del medioambiente
    let envcomps: Vec<_> = components
        .cdata
        .iter()
        .cloned()
        .filter(|c| c.carrier == Carrier::MEDIOAMBIENTE)
        .collect();
    // Identifica servicios
    let services: Vec<_> = envcomps.iter().map(|c| c.service).unique().collect();

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
                comment:
                    "Equilibrado de energía térmica insitu consumida y sin producción declarada"
                        .into(),
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
#[allow(non_snake_case)]
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
            let F_pr_srv = c_el_srv_tot / c_el_tot;
            for c in &pr_el_ndef {
                cdata.push(Component {
                    carrier: Carrier::ELECTRICIDAD,
                    ctype: CType::PRODUCCION,
                    csubtype: CSubtype::INSITU,
                    service,
                    values: veckmul(&c.values, F_pr_srv),
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
