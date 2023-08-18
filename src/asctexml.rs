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

use crate::types::*;
use crate::Components;
use crate::Factors;

// ==================== Conversión a XML de CTE y CEE

/// Muestra en formato XML de CTE y CEE
///
/// Esta función usa un formato compatible con el formato XML del certificado de eficiencia
/// energética del edificio definido en el documento de apoyo de la certificación energética
/// correspondiente.
pub trait AsCteXml {
    /// Get list of values
    fn to_xml(&self) -> String;

    /// Helper function -> XML escape symbols
    fn escape_xml(unescaped: &str) -> String {
        unescaped
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\\', "&apos;")
            .replace('"', "&quot;")
    }

    /// Convert list of numbers to string of comma separated values (2 decimal digits)
    fn format_values_2f(values: &[f32]) -> String {
        values
            .iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<String>>()
            .join(",")
    }
}

// ================= Implementaciones ====================


impl AsCteXml for EnergyPerformance {
    fn to_xml(&self) -> String {
        // Data
        let RenNrenCo2 { ren, nren, .. } = self.balance_m2.we.b;

        // Formatting
        let wfstring = self.wfactors.to_xml();
        let components_string = self.components.to_xml();

        // Final assembly
        format!(
            "<BalanceEPB>
        {}
        {}
        <kexp>{:.2}</kexp>
        <AreaRef>{:.2}</AreaRef><!-- área de referencia [m2] -->
        <Epm2><!-- C_ep [kWh/m2.an] -->
            <tot>{:.1}</tot>
            <nren>{:.1}</nren>
        </Epm2>
    </BalanceEPB>",
            wfstring,
            components_string,
            self.k_exp,
            self.arearef,
            ren + nren,
            nren
        )
    }
}

impl AsCteXml for Meta {
    fn to_xml(&self) -> String {
        format!(
            "<Metadato><Clave>{}</Clave><Valor>{}</Valor></Metadato>",
            <Self as AsCteXml>::escape_xml(&self.key),
            <Self as AsCteXml>::escape_xml(&self.value)
        )
    }
}

impl AsCteXml for Factor {
    fn to_xml(&self) -> String {
        let Factor {
            carrier,
            source,
            dest,
            step,
            ren,
            nren,
            co2,
            comment,
        } = self;
        let comentario = if comment.is_empty() {String::new()} else {
            format!("<Comentario>{}</Comentario>", <Self as AsCteXml>::escape_xml(comment))
        };
        format!(
            "<Factor><Vector>{}</Vector><Origen>{}</Origen><Destino>{}</Destino><Paso>{}</Paso><ren>{:.3}</ren><nren>{:.3}</nren><co2>{:.3}</co2>{}</Factor>",
            carrier, source, dest, step, ren, nren, co2, comentario
            
        )
    }
}

impl AsCteXml for Factors {
    fn to_xml(&self) -> String {
        let Factors { wmeta, wdata } = self;
        let wmetastring = wmeta
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        let wdatastring = wdata
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "<FactoresDePaso>
    {}
    {}
</FactoresDePaso>",
            wmetastring, wdatastring
        )
    }
}

impl AsCteXml for Components {
    fn to_xml(&self) -> String {
        let Components {
            meta,
            data,
            needs,
        } = self;
        let metastring = meta
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        let datastring = data
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        let needsdatastring = {
            let mut res = vec![];
            if let Some(nd) = &needs.ACS {
                res.push(format!("<Demanda><Servicio>ACS</Servicio><Valores>{}</Valores>", <Self as AsCteXml>::format_values_2f(nd)))
            };
            if let Some(nd) = &needs.CAL {
                res.push(format!("<Demanda><Servicio>CAL</Servicio><Valores>{}</Valores>", <Self as AsCteXml>::format_values_2f(nd)))
            };
            if let Some(nd) = &needs.REF {
                res.push(format!("<Demanda><Servicio>REF</Servicio><Valores>{}</Valores>", <Self as AsCteXml>::format_values_2f(nd)))
            };
            res.join("\n")
        };
        format!(
            "<Componentes>
        {}
        {}
        {}
    </Componentes>",
            metastring, datastring, needsdatastring
        )
    }
}

impl AsCteXml for Energy {
    fn to_xml(&self) -> String {
        match self {
            Energy::Used(e) => e.to_xml(),
            Energy::Prod(e) => e.to_xml(),
            Energy::Aux(e) => e.to_xml(),
            Energy::Out(e) => e.to_xml(),
        }
    }
}

impl AsCteXml for EProd {
    /// Convierte componente de energía producida a XML
    fn to_xml(&self) -> String {
        let Self {
            id,
            source,
            values,
            comment,
        } = self;
        let comentario = if comment.is_empty() {String::new()} else {
            format!("<Comentario>{}</Comentario>", <Self as AsCteXml>::escape_xml(comment))
        };
        format!(
            "<Produccion><Id>{}</Id><Origen>{}</Origen><Valores>{}</Valores>{}</Produccion>",
            id,
            source,
            <Self as AsCteXml>::format_values_2f(values),
            comentario
        )
    }
}

impl AsCteXml for EUsed {
    /// Convierte componente de energía consumida a XML
    fn to_xml(&self) -> String {
        let Self {
            id,
            carrier,
            service,
            values,
            comment,
        } = self;
        let comentario = if comment.is_empty() {String::new()} else {
            format!("<Comentario>{}</Comentario>", <Self as AsCteXml>::escape_xml(comment))
        };
        format!(
        "<Consumo><Id>{}</Id><Vector>{}</Vector><Servicio>{}</Servicio><Valores>{}</Valores>{}</Consumo>",
        id,
        carrier,
        service,
        <Self as AsCteXml>::format_values_2f(values),
        comentario
    )
    }
}

impl AsCteXml for EAux {
    /// Convierte componente de energía auxiliar a XML
    fn to_xml(&self) -> String {
        let Self {
            id,
            service,
            values,
            comment,
        } = self;
        let comentario = if comment.is_empty() {String::new()} else {
            format!("<Comentario>{}</Comentario>", <Self as AsCteXml>::escape_xml(comment))
        };
        format!(
        "<EAux><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores>{}</EAux>",
        id,
        service,
        <Self as AsCteXml>::format_values_2f(values),
        comentario
    )
    }
}

impl AsCteXml for EOut {
    /// Convierte componente de energía auxiliar a XML
    fn to_xml(&self) -> String {
        let Self {
            id,
            service,
            values,
            comment,
        } = self;
        let comentario = if comment.is_empty() {String::new()} else {
            format!("<Comentario>{}</Comentario>", <Self as AsCteXml>::escape_xml(comment))
        };
        format!(
        "<Salida><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores>{}</Salida>",
        id,
        service,
        <Self as AsCteXml>::format_values_2f(values),
        comentario
    )
    }
}

impl AsCteXml for Needs {
    /// Convierte elementos de demanda del edificio a XML
    fn to_xml(&self) -> String {
        let Self { service, values        } = self;
        format!(
            "<DemandaEdificio><Servicio>{}</Servicio><Valores>{}</Valores></DemandaEdificio>",
            service,
            <Self as AsCteXml>::format_values_2f(values)
        )
    }
}
