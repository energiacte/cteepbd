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
use crate::BalanceTotal;
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
        // Datos generales
        let Balance {
            k_exp,
            arearef,
            balance_m2,
            misc,
            ..
        } = self;

        let BalanceTotal {
            A: bal_a,
            B: bal_b,
            used_EPB: used_epb,
            used_nEPB: used_nepb,
            prod,
            del,
            exp,
            exp_grid,
            exp_nEPB: exp_nepb,
            ..
        } = balance_m2;

        let RenNrenCo2 { ren, nren, co2, .. } = bal_b;
        let tot = bal_b.tot();
        let rer = bal_b.rer();

        let used = used_epb + used_nepb;

        // Consumos
        let used_by_srv = list_entries_f32(&balance_m2.used_EPB_by_srv);
        // Generada
        let prod_by_src = list_entries_f32(&balance_m2.prod_by_src);
        // Producida, por vector
        let prod_by_cr = list_entries_f32(&balance_m2.prod_by_cr);
        let balance_m2_a = rennren2string(bal_a);
        // Ponderada por m2 (por uso)
        let a_by_srv = list_entries_rennrenco2(&balance_m2.A_by_srv);
        let balance_m2_b = rennren2string(bal_b);
        let b_by_srv = list_entries_rennrenco2(&balance_m2.B_by_srv);
        // Parámetros de demanda HE4
        let misc_out = if let Some(map) = misc {
            let demanda = map.get_str_1d("demanda_anual_acs");
            let pct_ren = map.get_str_pct1d("fraccion_renovable_demanda_acs_nrb");
            format!("\n** Indicadores adicionales\n
Demanda total de ACS: {demanda} [kWh]\nPorcentaje renovable de la demanda de ACS (perímetro próximo): {pct_ren} [%]
"            )
        } else {
            String::new()
        };

        format!(
            "** Balance energético

Area_ref = {arearef:.2} [m2]
k_exp = {k_exp:.2}
C_ep [kWh/m2.an]: ren = {ren:.1}, nren = {nren:.1}, tot = {tot:.1}
E_CO2 [kg_CO2e/m2.an]: {co2:.2}
RER = {rer:.2}

** Energía final (todos los vectores) [kWh/m2.an]:

Energía consumida: {used:.2}

Consumida en usos EPB: {used_epb:.2}

{used_by_srv}

Consumida en usos no EPB: {used_nepb:.2}

Generada: {prod:.2}

Generada, por origen:

{prod_by_src}

Generada, por vector:

{prod_by_cr}

Suministrada: {del:.2}

Exportada: {exp:.2}

- a la red: {exp_grid:.2}
- a usos no EPB: {exp_nepb:.2}

** Energía primaria (ren, nren) [kWh/m2.an] y emisiones [kg_CO2e/m2.an] por servicios:

Recursos utilizados (paso A): {balance_m2_a}

{a_by_srv}

Incluyendo el efecto de la energía exportada (paso B): {balance_m2_b}

{b_by_srv}{misc_out}
"
        )
    }
}

fn list_entries_f32<T: std::fmt::Display>(map: &std::collections::HashMap<T, f32>) -> String {
    let mut entries = map
        .iter()
        .map(|(k, v)| format!("- {}: {:.2}", k, v))
        .collect::<Vec<String>>();
    entries.sort();
    entries.join("\n")
}

fn list_entries_rennrenco2<T: std::fmt::Display>(
    map: &std::collections::HashMap<T, RenNrenCo2>,
) -> String {
    let mut entries = map
        .iter()
        .map(|(k, v)| format!("- {}: {}", k, rennren2string(v)))
        .collect::<Vec<String>>();
    entries.sort();
    entries.join("\n")
}
