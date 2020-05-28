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
Gestión de errores (error handling)
===================================

Tipos y funciones para la gestión de errores
*/

use std::fmt;

/// Resultado que usa el tipo de error personalizado
pub type Result<T> = std::result::Result<T, EpbdError>;

/// Errores definidos para la librería y aplicación cteepbd
#[derive(Debug)]
pub enum EpbdError {
    /// Error al interpretar un valor
    ParseError(String),
    /// Error para un valor de entrada incorrecto (formato o rango incorrecto)
    WrongInput(String),
    /// Error cuando falta un factor de conversión
    MissingFactor(String),
}

impl fmt::Display for EpbdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EpbdError::*;
        match self {
            ParseError(v) => write!(f, "No se ha podido interpretar {}", v),
            WrongInput(v) => write!(f, "Valor de entrada incorrecto: {}", v),
            MissingFactor(v) => write!(f, "Factor de paso no encontrado: {}", v),
        }
    }
}

impl std::error::Error for EpbdError {}

impl From<std::num::ParseFloatError> for EpbdError {
    fn from(err: std::num::ParseFloatError) -> Self {
        EpbdError::ParseError(err.to_string())
    }
}
