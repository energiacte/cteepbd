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


impl AsCteXml for Balance {
    fn to_xml(&self) -> String {
        let Balance {
            components,
            wfactors,
            k_exp,
            arearef,
            balance_m2,
            ..
        } = self;

        // Data
        let RenNrenCo2 { ren, nren, .. } = balance_m2.B;

        // Formatting
        let wfstring = wfactors.to_xml();
        let components_string = components.to_xml();

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
            k_exp,
            arearef,
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
        format!(
            "<Factor><Vector>{}</Vector><Origen>{}</Origen><Destino>{}</Destino><Paso>{}</Paso><ren>{:.3}</ren><nren>{:.3}</nren><co2>{:.3}</co2><Comentario>{}</Comentario></Factor>",
            carrier, source, dest, step, ren, nren, co2, 
            <Self as AsCteXml>::escape_xml(comment)
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
            cmeta,
            cdata,
            zones,
        } = self;
        let cmetastring = cmeta
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        let cdatastring = cdata
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        let zonesdatastring = zones
            .iter()
            .map(AsCteXml::to_xml)
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "<Componentes>
        {}
        {}
        {}
    </Componentes>",
            cmetastring, cdatastring, zonesdatastring
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
            carrier,
            source,
            values,
            comment,
        } = self;
        format!(
            "<Produccion><Id>{}</Id><Vector>{}</Vector><Origen>{}</Origen><Valores>{}</Valores><Comentario>{}</Comentario></Produccion>",
            id,
            carrier,
            source,
            <Self as AsCteXml>::format_values_2f(values),
            <Self as AsCteXml>::escape_xml(comment)
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
        format!(
        "<Consumo><Id>{}</Id><Vector>{}</Vector><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></Consumo>",
        id,
        carrier,
        service,
        <Self as AsCteXml>::format_values_2f(values),
        <Self as AsCteXml>::escape_xml(comment)
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
        format!(
        "<EAux><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></EAux>",
        id,
        service,
        <Self as AsCteXml>::format_values_2f(values),
        <Self as AsCteXml>::escape_xml(comment)
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
        format!(
        "<Salida><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></Salida>",
        id,
        service,
        <Self as AsCteXml>::format_values_2f(values),
        <Self as AsCteXml>::escape_xml(comment)
    )
    }
}

impl AsCteXml for ZoneNeeds {
    /// Convierte elementos de demanda a XML
    fn to_xml(&self) -> String {
        let Self {
            id,
            service,
            values,
            comment,
        } = self;
        format!(
            "<DemandaZona><Id>{}</Id><Servicio>{}</Servicio><Valores>{}</Valores><Comentario>{}</Comentario></DemandaZona>",
            id,
            service,
            <Self as AsCteXml>::format_values_2f(values),
            <Self as AsCteXml>::escape_xml(comment)
        )
    }
}
