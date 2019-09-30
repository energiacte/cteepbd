// Copyright (c) 2018-2019  Ministerio de Fomento
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
Error handling
==============

Error handling types and helpers
*/

use std::fmt;

/// Custom Result
pub type Result<T> = std::result::Result<T, EpbdError>;

/// Errors defined for the cteepbd library and application
#[derive(Debug)]
pub enum EpbdError {
    /// Error when parsing a number
    NumberParseError(String),
    /// Error when parsing a Meta
    MetaParseError(String),
    /// Error when parsing a Component
    ComponentParseError(String),
    /// Error when parsing Components (Component list + metadata)
    ComponentsParseError(String),
    /// Error when parsing a Factor
    WFactorParseError(String),
    /// Error when parsing a RenNrenCo2 element
    RenNrenCo2ParseError(String),
    /// Error when a Carrier is not known
    CarrierUnknown(String),
    /// Error when a CType is not known
    CTypeUnknown(String),
    /// Error when a CSubtype is not known
    CSubtypeUnknown(String),
    /// Error when a Service is not known
    ServiceUnknown(String),
    /// Error when a Source is not known
    SourceUnknown(String),
    /// Error when a Dest is not known
    DestUnknown(String),
    /// Error when a Step is not known
    StepUnknown(String),
    /// Error when converting from CSubtype to Source
    SourceConversionError(String),
    /// Error for an invalid Area (wrong format or out of range)
    Area(String),
    /// Error forn an invalid Location (not known)
    Location(String),
    /// Error when a Factor is needed but not available
    FactorNotFound(String),
}

impl fmt::Display for EpbdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EpbdError::*;
        match self {
            NumberParseError(num) => write!(f, "Could not parse number value: {}", num),
            MetaParseError(input) => write!(f, "Could not parse Meta from \"{}\"", input),
            ComponentParseError(input) => write!(f, "Could not parse Component from \"{}\"", input),
            ComponentsParseError(input) => write!(f, "Could not parse Components from \"{}\"", input),
            WFactorParseError(input) => write!(f, "Could not parse Factor from \"{}\"", input),
            RenNrenCo2ParseError(input) => write!(f, "Could not parse RenNrenCo2 from \"{}\"", input),
            CarrierUnknown(input) => write!(f, "Unknown Carrier: \"{}\"", input),
            CTypeUnknown(input) => write!(f, "Unknown Ctype: \"{}\"", input),
            CSubtypeUnknown(input) => write!(f, "Unknown CSubtype: \"{}\"", input),
            ServiceUnknown(input) => write!(f, "Unknown Service: \"{}\"", input),
            SourceUnknown(input) => write!(f, "Unknown Source: \"{}\"", input),
            DestUnknown(input) => write!(f, "Unknown Dest: \"{}\"", input),
            StepUnknown(input) => write!(f, "Unknown Step: \"{}\"", input),
            SourceConversionError(input) => write!(f, "Could not convert to Source from CSubtype: {}", input),
            Area(area) => write!(f, "Unexpected reference area value: {}", area),
            Location(loc) => write!(f, "Unknown location; \"{}\"", loc),
            FactorNotFound(desc) => write!(f, "Conversion factor not found: {}", desc),
        }
    }
}

impl std::error::Error for EpbdError {}

impl From<std::num::ParseFloatError> for EpbdError {
    fn from(err: std::num::ParseFloatError) -> Self {
        EpbdError::NumberParseError(err.to_string())
    }
}
