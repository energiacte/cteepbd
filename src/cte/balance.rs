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
//            Daniel Jiménez González <dani@ietcc.csic.es>

/*! # Manejo de balances energéticos para el CTE

Utilidades para la gestión de balances energéticos para el CTE

*/

use itertools::Itertools; // join

use super::WFactorsMode;
use crate::types::{Balance, Component, Factor, MetaVec};
use crate::RenNrenCo2;


/// Muestra balance, paso B, de forma simplificada.
pub fn balance_to_plain(balance: &Balance) -> String {
    let Balance {
        k_exp,
        arearef,
        balance_m2,
        wfactors,
        ..
    } = balance;

    // XXX: TODO: Corregir
    let indicator_and_units =
        if wfactors.has_meta_value("CTE_FACTORES_TIPO", WFactorsMode::CO2.as_meta_value()) {
            "CO2_eq [kg_CO2e/m2.an]"
        } else {
            "C_ep [kWh/m2.an]"
        };
    let RenNrenCo2 { ren, nren, co2 } = balance_m2.B;
    let tot = balance_m2.B.tot();
    let rer = balance_m2.B.rer();

    // Final
    let mut use_byuse = balance_m2
        .used_EPB_byuse
        .iter()
        .map(|(k, v)| format!("{}: {:.2}", k, v))
        .collect::<Vec<String>>();
    use_byuse.sort();

    // Ponderada por m2 (por uso)
    let mut b_byuse = balance_m2
        .B_byuse
        .iter()
        .map(|(k, v)| format!("{}: ren {:.2}, nren {:.2}", k, v.ren, v.nren))
        .collect::<Vec<String>>();
    b_byuse.sort();

    format!(
        "Area_ref = {:.2} [m2]
k_exp = {:.2}
{}: ren = {:.1}, nren = {:.1}, tot = {:.1}, RER = {:.2}, co2 = {:.2}

** Energía final (todos los vectores) [kWh/m2.an]:
{}

** {} por servicios:
{}
",
        arearef,
        k_exp,
        indicator_and_units,
        ren,
        nren,
        tot,
        rer,
        co2,
        use_byuse.join("\n"),
        indicator_and_units,
        b_byuse.join("\n")
    )
}

/// Muestra balance en formato XML.
pub fn balance_to_xml(balanceobj: &Balance) -> String {
    let Balance {
        components,
        wfactors,
        k_exp,
        arearef,
        balance_m2,
        ..
    } = balanceobj;

    let indicator_and_units =
        if wfactors.has_meta_value("CTE_FACTORES_TIPO", WFactorsMode::CO2.as_meta_value()) {
            "CO2_eq [kg_CO2e/m2.an]"
        } else {
            "C_ep [kWh/m2.an]"
        };
    let RenNrenCo2 { ren, nren, co2: _ } = balance_m2.B;
    let cmeta = &components.cmeta;
    let cdata = &components.cdata;
    let wmeta = &wfactors.wmeta;
    let wdata = &wfactors.wdata;
    let wmetastring = wmeta
        .iter()
        .map(|m| {
            format!(
                "      <Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
                escape_xml(&m.key),
                escape_xml(&m.value)
            )
        })
        .join("\n");
    let wdatastring = wdata
        .iter()
        .map(|f| {
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
            format!("      <Dato><Vector>{}</Vector><Origen>{}</Origen><Destino>{}</Destino><Paso>{}</Paso><ren>{:.3}</ren><nren>{:.3}</nren><CO2>{:.3}</CO2><Comentario>{}</Comentario></Dato>",
            carrier, source, dest, step, ren, nren, co2, escape_xml(comment))
        })
        .join("\n");
    let cmetastring = cmeta
        .iter()
        .map(|m| {
            format!(
                "      <Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
                escape_xml(&m.key),
                escape_xml(&m.value)
            )
        })
        .join("\n");
    let cdatastring = cdata
        .iter()
        .map(|c| {
            let Component {
                carrier,
                ctype,
                csubtype,
                service,
                values,
                comment,
            } = c;
            let vals = values.iter().map(|v| format!("{:.2}", v)).join(",");
            format!(
                "      <Dato>
            <Vector>{}</Vector><Tipo>{}</Tipo><Subtipo>{}</Subtipo><Servicio>{}</Servicio>
            <Valores>{}</Valores>
            <Comentario>{}</Comentario>
        </Dato>",
                carrier,
                ctype,
                csubtype,
                service,
                vals,
                escape_xml(comment)
            )
        })
        .join("\n");

    format!(
        "<BalanceEPB>
    <FactoresDePaso>
        <Metadatos>
    {}
        </Metadatos>
        <Datos>
    {}
        </Datos>
    </FactoresDePaso>
    <Componentes>
        <Metadatos>
    {}
        </Metadatos>
        <Datos>
    {}
        </Datos>
    </Componentes>
    <kexp>{:.2}</kexp>
    <AreaRef>{:.2}</AreaRef><!-- área de referencia [m2] -->
    <Epm2><!-- {} -->
        <tot>{:.1}</tot>
        <nren>{:.1}</nren>
    </Epm2>
</BalanceEPB>",
        wmetastring,
        wdatastring,
        cmetastring,
        cdatastring,
        k_exp,
        arearef,
        indicator_and_units,
        ren + nren,
        nren
    )
}

/// Sustituye símbolos reservados en XML.
fn escape_xml(unescaped: &str) -> String {
    unescaped
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "&apos;")
        .replace('"', "&quot;")
}
