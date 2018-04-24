// Copyright (c) 2018 Ministerio de Fomento
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

// Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>

use std::fmt;
use std::str;

use failure::Error;

// Common (carriers + weighting factors)

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Display, EnumString)]
pub enum carrierType {
    ELECTRICIDAD,
    MEDIOAMBIENTE,
    BIOCARBURANTE,
    BIOMASA,
    BIOMASADENSIFICADA,
    CARBON,
    FUELOIL,
    GASNATURAL,
    GASOLEO,
    GLP,
    RED1,
    RED2,
}

// Energy Components

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Display, EnumString)]
pub enum ctypeType {
    PRODUCCION,
    CONSUMO,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Display, EnumString)]
pub enum csubtypeType {
    INSITU,
    COGENERACION,
    EPB,
    NEPB,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Display)]
pub enum serviceType {
    ACS,
    CAL,
    REF,
    VEN,
    ILU,
    HU,
    DHU,
    NDEF,
}

impl str::FromStr for serviceType {
    type Err = Error;

    fn from_str(s: &str) -> Result<serviceType, Self::Err> {
        match s {
            "ACS" => Ok(serviceType::ACS),
            "WATERSYSTEMS" => Ok(serviceType::ACS),
            "CAL" => Ok(serviceType::CAL),
            "HEATING" => Ok(serviceType::CAL),
            "REF" => Ok(serviceType::REF),
            "COOLING" => Ok(serviceType::REF),
            "VEN" => Ok(serviceType::VEN),
            "FANS" => Ok(serviceType::VEN),
            "ILU" => Ok(serviceType::ACS),
            "HU" => Ok(serviceType::ACS),
            "DHU" => Ok(serviceType::ACS),
            "" => Ok(serviceType::NDEF),
            "NDEF" => Ok(serviceType::NDEF),
            _ => Err(format_err!("serviceType not found")),
        }
    }
}

// Weighting factors

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Display, EnumString)]
pub enum sourceType {
    RED,
    INSITU,
    COGENERACION,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Display, EnumString)]
pub enum destType {
    input,
    to_grid,
    to_nEPB,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Display, EnumString)]
pub enum stepType {
    A,
    B,
}

// General types

// Metadata Struct
// * objects of type 'META' represent metadata of components or weighting factors
//   - key is the metadata name
//   - value is the metadata value
pub struct TMeta {
    pub key: String,
    pub value: String,
}

impl fmt::Display for TMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#META {}: {}", self.key, self.value)
    }
}

impl str::FromStr for TMeta {
    type Err = Error;

    fn from_str(s: &str) -> Result<TMeta, Self::Err> {
        // Remove start of line with #META or #CTE_
        let items: Vec<&str> = s.trim()[5..].splitn(2, ':').map(|v| v.trim()).collect();
        if items.len() == 2 {
            let key = items[0].to_string();
            let value = items[1].to_string();
            Ok(TMeta { key, value })
        } else {
            Err(format_err!("Couldn't parse Metadata from string"))
        }
    }
}

// Energy Carrier Component Struct, representing an energy carrier component
//   - carrier is the carrier name
//   - ctype is either 'PRODUCCION' or 'CONSUMO' for produced or used energy
//   - csubtype defines:
//     - the energy origin for produced energy (INSITU or COGENERACION)
//     - the energy end use (EPB or NEPB) for delivered energy
//   - values is a list of energy values, one for each timestep
//   - comment is a comment string for the carrier
pub struct TComponent {
    pub carrier: carrierType,
    pub ctype: ctypeType,
    pub csubtype: csubtypeType,
    pub service: serviceType,
    pub values: Vec<f32>,
    pub comment: String,
}

impl fmt::Display for TComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let valuelist = self.values
            .iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
            .join(", ");
        let comment = if self.comment != "" {
            format!(" # {}", self.comment)
        } else {
            "".to_owned()
        };
        write!(
            f,
            "{}, {}, {}, {}, {}{}",
            self.carrier, self.ctype, self.csubtype, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for TComponent {
    type Err = Error;

    fn from_str(s: &str) -> Result<TComponent, Self::Err> {
        let items: Vec<&str> = s.trim().splitn(2, '#').map(|v| v.trim()).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(|v| v.trim()).collect();
        if items.len() < 4 {
            return Err(format_err!(
                "Couldn't parse Component (TComponent) from string"
            ));
        };
        let carrier: carrierType = items[0].parse()?;
        let ctype: ctypeType = items[1].parse()?;
        let csubtype: csubtypeType = items[2].parse()?;
        //This accounts for the legacy version, which may not have a service type
        let maybeservice: Result<serviceType, _> = items[3].parse();
        let (valuesidx, service) = match maybeservice {
            Ok(s) => (4, s),
            Err(_) => (3, serviceType::NDEF),
        };
        let values = items[valuesidx..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        Ok(TComponent {
            carrier,
            ctype,
            csubtype,
            service,
            values,
            comment,
        })
    }
}

// Weighting Factor Struct
pub struct TFactor {
    pub carrier: carrierType,
    pub source: sourceType,
    pub dest: destType,
    pub step: stepType,
    pub ren: f32,
    pub nren: f32,
    pub comment: String,
}

impl fmt::Display for TFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let comment = if self.comment != "" {
            format!(" # {}", self.comment)
        } else {
            "".to_owned()
        };
        write!(
            f,
            "{}, {}, {}, {}, {:.3}, {:.3}{}",
            self.carrier, self.source, self.dest, self.step, self.ren, self.nren, comment
        )
    }
}

impl str::FromStr for TFactor {
    type Err = Error;

    fn from_str(s: &str) -> Result<TFactor, Self::Err> {
        let items: Vec<&str> = s.trim().splitn(2, '#').map(|v| v.trim()).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(|v| v.trim()).collect();
        if items.len() < 6 {
            return Err(format_err!(
                "Couldn't parse Weighting Factor (TFactor) from string"
            ));
        };
        let carrier: carrierType = items[0].parse()?;
        let source: sourceType = items[1].parse()?;
        let dest: destType = items[2].parse()?;
        let step: stepType = items[3].parse()?;
        let ren: f32 = items[4].parse()?;
        let nren: f32 = items[5].parse()?;
        Ok(TFactor {
            carrier,
            source,
            dest,
            step,
            ren,
            nren,
            comment,
        })
    }
}

// List of Components with Metadata

// Components object with meta and carrier data
//
// #META CTE_AREAREF: 100.5
// ELECTRICIDAD,CONSUMO,EPB,16.39,13.11,8.20,7.38,4.10,4.92,6.56,5.74,4.10,6.56,9.84,13.11
// ELECTRICIDAD,PRODUCCION,INSITU,8.20,6.56,4.10,3.69,2.05,2.46,3.28,2.87,2.05,3.28,4.92,6.56
pub struct TComponents {
    pub cmeta: Vec<TMeta>,
    pub cdata: Vec<TComponent>,
}

impl fmt::Display for TComponents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let metalines = self.cmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self.cdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for TComponents {
    type Err = Error;

    fn from_str(s: &str) -> Result<TComponents, Self::Err> {
        let lines: Vec<&str> = s.lines().map(|v| v.trim()).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let cmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<TMeta>, _>>()?;
        let cdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<TComponent>, _>>()?;
        Ok(TComponents { cmeta, cdata })
    }
}

// List of Weighting Factors with Metadata
pub struct TFactors {
    pub wmeta: Vec<TMeta>,
    pub wdata: Vec<TFactor>,
}

impl fmt::Display for TFactors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let metalines = self.wmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self.wdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for TFactors {
    type Err = Error;

    fn from_str(s: &str) -> Result<TFactors, Self::Err> {
        let lines: Vec<&str> = s.lines().map(|v| v.trim()).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let wmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<TMeta>, _>>()?;
        let wdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<TFactor>, _>>()?;
        Ok(TFactors { wmeta, wdata })
    }
}

// Results Struct for Output
//TODO: implement Display to serialize and FromStr to deserialize? JSON?
#[allow(dead_code)]
pub struct TBalance {
    pub components: TComponents,
    pub wfactors: TFactors,
    pub k_exp: f32,
    pub arearef: f32,
    pub balance_cr_i: String, // TODO: era any
    pub balance: String,      // TODO: era any
    pub balance_m2: String,   // TODO: era any
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmeta() {
        let meta = TMeta {
            key: "CTE_FUENTE".to_string(),
            value: "CTE2013".to_string(),
        };
        let metastr = "#META CTE_FUENTE: CTE2013";
        assert_eq!(format!("{}", meta), metastr);
        assert_eq!(format!("{}", metastr.parse::<TMeta>().unwrap()), metastr);
    }

    #[test]
    fn tcomponent() {
        let component1 = TComponent {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "CONSUMO".parse().unwrap(),
            csubtype: "EPB".parse().unwrap(),
            service: "REF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0
            ],
            comment: "Comentario cons 1".into(),
        };
        let component1str = "ELECTRICIDAD, CONSUMO, EPB, REF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario cons 1";
        let component2 = TComponent {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "PRODUCCION".parse().unwrap(),
            csubtype: "INSITU".parse().unwrap(),
            service: "NDEF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0
            ],
            comment: "Comentario prod 1".into(),
        };
        let component2str = "ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        let component2strlegacy = "ELECTRICIDAD, PRODUCCION, INSITU, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(format!("{}", component1), component1str);

        // producer component
        assert_eq!(format!("{}", component2), component2str);

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", component2str.parse::<TComponent>().unwrap()),
            component2str
        );
        // roundtrip building from/to string for legacy format
        assert_eq!(
            format!("{}", component2strlegacy.parse::<TComponent>().unwrap()),
            component2str
        );
    }

    #[test]
    fn tfactor() {
        let factor1 = TFactor {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            source: "RED".parse().unwrap(),
            dest: "input".parse().unwrap(),
            step: "A".parse().unwrap(),
            ren: 0.414,
            nren: 1.954,
            comment: "Electricidad de red paso A".into(),
        };
        let factor1str = "ELECTRICIDAD, RED, input, A, 0.414, 1.954 # Electricidad de red paso A";
        let factor2str = "ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(format!("{}", factor1), factor1str);

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", factor2str.parse::<TComponent>().unwrap()),
            factor2str
        );
    }

    #[test]
    fn tcomponents() {
        let tcomponents1 = "#META CTE_AREAREF: 100.5
ELECTRICIDAD, CONSUMO, EPB, NDEF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tcomponents1.parse::<TComponents>().unwrap()),
            tcomponents1
        );
    }

    #[test]
    fn tfactors() {
        let tfactors1 = "#META CTE_FUENTE: CTE2013
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, input, A, 0.414, 1.954 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, input, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tfactors1.parse::<TFactors>().unwrap()),
            tfactors1
        );
    }
}
