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
CteEPBD
=======

This library is an implementation of the ISO EN 52000-1 standard

  Energy performance of buildings - Overarching EPB assessment - General framework and procedures
  This implementation has used the following assumptions:
  - weighting factors are constant for all timesteps
  - no priority is set for energy production (average step A weighting factor f_we_el_stepA)
  - all on-site produced energy from non cogeneration sources is considered as delivered
  - on-site produced energy is not compensated on a service by service basis, but on a by carrier basis
  - the load matching factor is constant and equal to 1.0
  TODO:
  - allow other values of the load matching factor (or usign functions) f_match_t (formula 32, B.32)

*/

#![deny(missing_docs)]

#[cfg(test)] // <-- not needed in examples + integration tests
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate serde_derive;

mod balance;
mod components;
pub mod cte;
pub mod error;
pub mod types;
mod vecops;
mod wfactors;

pub use balance::*;
pub use components::*;
pub use wfactors::*;

/// Version number
pub static VERSION: &str = env!("CARGO_PKG_VERSION");
