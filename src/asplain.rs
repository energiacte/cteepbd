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
// use crate::Components;
// use crate::Factors;

// ==================== Conversión a formato simple

/// Muestra en formato simple
///
/// Esta función usa un formato simple y compacto para representar la información sobre
/// eficiencia energética del edificio, datos y balances
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

/// Muestra un valor opcional con la precisión deseada o como un guion si no está presente
fn value_or_dash(v: Option<f32>, precision: usize) -> String {
    match v {
        Some(v) => format!("{:.*}", precision, v),
        None => "-".to_string(),
    }
}

impl AsCtePlain for EnergyPerformance {
    /// Está mostrando únicamente los resultados
    fn to_plain(&self) -> String {
        // Datos generales
        let bal = &self.balance_m2;
        let k_exp = self.k_exp;
        let arearef = self.arearef;

        // Demanda
        let dhw_needs = value_or_dash(bal.needs.ACS, 1);
        let heating_needs = value_or_dash(bal.needs.CAL, 1);
        let cooling_needs = value_or_dash(bal.needs.REF, 1);

        // Consumos
        let epus = bal.used.epus;
        let nepus = bal.used.nepus;
        let cgnus = bal.used.cgnus;
        let used = epus + nepus + cgnus;

        let used_by_srv = to_key_value_list(&bal.used.epus_by_srv);
        let used_epus_by_cr = to_key_value_list(&bal.used.epus_by_cr);
        // Generada
        let prod_an = bal.prod.an;
        let prod_by_src = to_key_value_list(&bal.prod.by_src);
        let prod_by_cr = to_key_value_list(&bal.prod.by_cr);
        let prod_epus_by_src = to_key_value_list(&bal.prod.epus_by_src);
        // Suministrada
        let del_an = bal.del.an;
        let del_grid = bal.del.grid;
        let del_onsite = bal.del.onst;
        // let del_cgn = bal.del.cgn;
        // Exportada
        let exp_an = bal.exp.an;
        let exp_grid = bal.exp.grid;
        let exp_nepus = bal.exp.nepus;
        // Ponderada por m2 (por uso)
        let we_a = bal.we.a;
        let we_b = bal.we.b;
        let RenNrenCo2 { ren, nren, co2, .. } = we_b;
        let tot = we_b.tot();
        let rer = self.rer;
        let rer_nrb = self.rer_nrb;
        let balance_m2_a = rennren2string(&we_a);
        let a_by_srv = to_key_rennrenco2_value_list(&bal.we.a_by_srv);
        let balance_m2_b = rennren2string(&we_b);
        let b_by_srv = to_key_rennrenco2_value_list(&bal.we.b_by_srv);
        // Parámetros de demanda HE4
        let misc_out = if let Some(map) = &self.misc {
            let pct_ren = map.get_str_pct1d("fraccion_renovable_demanda_acs_nrb");
            format!("\n\n** Indicadores adicionales\nPorcentaje renovable de la demanda de ACS (perímetro próximo): {pct_ren} [%]")
        } else {
            String::new()
        };

        format!(
            "** Eficiencia energética

Area_ref = {arearef:.2} [m2]
k_exp = {k_exp:.2}
C_ep [kWh/m2.an]: ren = {ren:.1}, nren = {nren:.1}, tot = {tot:.1}
E_CO2 [kg_CO2e/m2.an]: {co2:.2}
RER = {rer:.2}
RER_nrb = {rer_nrb:.2}

** Demanda [kWh/m2.an]:

- ACS: {dhw_needs}
- CAL: {heating_needs}
- REF: {cooling_needs}

** Energía final (todos los vectores) [kWh/m2.an]:

Energía consumida: {used:.2}

+ Consumida en usos EPB: {epus:.2}

* por servicio:
{used_by_srv}

* por vector:
{used_epus_by_cr}

+ Consumida en usos no EPB: {nepus:.2}

+ Consumida en cogeneración: {cgnus:.2}

Generada: {prod_an:.2}

* por vector:
{prod_by_cr}

* por origen:
{prod_by_src}

* generada y usada en servicios EPB, por origen:
{prod_epus_by_src}

Suministrada {del_an:.2}:

- de red: {del_grid:.2}
- in situ: {del_onsite:.2}

Exportada: {exp_an:.2}

- a la red: {exp_grid:.2}
- a usos no EPB: {exp_nepus:.2}

** Energía primaria (ren, nren) [kWh/m2.an] y emisiones [kg_CO2e/m2.an]:

Recursos utilizados (paso A): {balance_m2_a}

* por servicio:
{a_by_srv}

Incluyendo el efecto de la energía exportada (paso B): {balance_m2_b}

* por servicio:
{b_by_srv}{misc_out}
"
        )
    }
}

fn to_key_value_list<T: std::fmt::Display>(map: &std::collections::HashMap<T, f32>) -> String {
    let mut entries = map
        .iter()
        .map(|(k, v)| format!("- {}: {:.2}", k, v))
        .collect::<Vec<String>>();
    entries.sort();
    entries.join("\n")
}

fn to_key_rennrenco2_value_list<T: std::fmt::Display>(
    map: &std::collections::HashMap<T, RenNrenCo2>,
) -> String {
    let mut entries = map
        .iter()
        .map(|(k, v)| format!("- {}: {}", k, rennren2string(v)))
        .collect::<Vec<String>>();
    entries.sort();
    entries.join("\n")
}
