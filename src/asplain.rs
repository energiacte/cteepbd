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

use crate::types::*;
use crate::Balance;
// use crate::Components;
// use crate::Factors;

// ==================== Conversión a formato simple

/// Muestra en formato simple
///
/// Esta función usa un formato simple y compacto para representar la información sobre
/// eficiencia energética del edificio, su balance y los resultados
pub trait AsCtePlain {
    /// Get in plan format
    fn to_plain(&self) -> String;
}

// ================= Implementaciones ====================

/// Convierte resultado RenNrenCo2 a String con2 decimales
fn rennren2string(v: &RenNrenCo2) -> String {
    format!(
        "ren {:.2}, nren {:.2}, tot: {:.2}, co2: {:.2}",
        v.ren,
        v.nren,
        v.tot(),
        v.co2
    )
}

impl AsCtePlain for Balance {
    /// Está mostrando únicamente los resultados
    fn to_plain(&self) -> String {
        let Balance {
            k_exp,
            arearef,
            balance_m2,
            misc,
            ..
        } = self;

        // Final, por servicios
        let mut used_by_service = balance_m2
            .used_EPB_by_service
            .iter()
            .map(|(k, v)| format!("- {}: {:.2}", k, v))
            .collect::<Vec<String>>();
        used_by_service.sort();

        // Producida, por origen
        let mut produced_by_source = balance_m2
            .produced_by_source
            .iter()
            .map(|(k, v)| format!("- {}: {:.2}", k, v))
            .collect::<Vec<String>>();
        produced_by_source.sort();

        // Producida, por vector
        let mut produced_by_carrier = balance_m2
            .produced_by_carrier
            .iter()
            .map(|(k, v)| format!("- {}: {:.2}", k, v))
            .collect::<Vec<String>>();
        produced_by_carrier.sort();

        // Ponderada por m2 (por uso)
        let mut a_by_service = balance_m2
            .A_by_service
            .iter()
            .map(|(k, v)| format!("- {}: {}", k, rennren2string(v)))
            .collect::<Vec<String>>();
        a_by_service.sort();

        let mut b_by_service = balance_m2
            .B_by_service
            .iter()
            .map(|(k, v)| format!("- {}: {}", k, rennren2string(v)))
            .collect::<Vec<String>>();
        b_by_service.sort();

        let out = format!(
            "** Balance energético

Area_ref = {:.2} [m2]
k_exp = {:.2}
C_ep [kWh/m2.an]: ren = {:.1}, nren = {:.1}, tot = {:.1}
E_CO2 [kg_CO2e/m2.an]: {:.2}
RER = {:.2}

** Energía final (todos los vectores) [kWh/m2.an]:

Energía consumida: {:.2}

Consumida en usos EPB: {:.2}

{}

Consumida en usos no EPB: {:.2}

Generada: {:.2}

Generada, por origen:

{}

Generada, por vector:

{}

Suministrada: {:.2}

Exportada: {:.2}

- a la red: {:.2}
- a usos no EPB: {:.2}

** Energía primaria (ren, nren) [kWh/m2.an] y emisiones [kg_CO2e/m2.an] por servicios:

Recursos utilizados (paso A): {}

{}

Incluyendo el efecto de la energía exportada (paso B): {}

{}
",
            arearef,
            k_exp,
            balance_m2.B.ren,
            balance_m2.B.nren,
            balance_m2.B.tot(),
            balance_m2.B.co2,
            balance_m2.B.rer(),
            balance_m2.used_EPB + balance_m2.used_nEPB,
            balance_m2.used_EPB,
            used_by_service.join("\n"),
            balance_m2.used_nEPB,
            balance_m2.produced,
            produced_by_source.join("\n"),
            produced_by_carrier.join("\n"),
            balance_m2.delivered,
            balance_m2.exported,
            balance_m2.exported_grid,
            balance_m2.exported_nEPB,
            rennren2string(&balance_m2.A),
            a_by_service.join("\n"),
            rennren2string(&balance_m2.B),
            b_by_service.join("\n")
        );
        // Añade parámetros de demanda HE4 si existen
        if let Some(map) = misc {
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
}
