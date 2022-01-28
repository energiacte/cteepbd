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

use std::{fmt, str};

use serde::{Deserialize, Serialize};

use crate::types::{Carrier, GenProd, GenCrIn, HasValues, Service, Source};

/// Componentes de energía generada o consumida
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnergyData {
    /// Energía consumida
    GenCrIn(GenCrIn),
    /// Energía generada
    GenProd(GenProd),
}

impl EnergyData {
    /// Get id for this service
    pub fn id(&self) -> i32 {
        match self {
            EnergyData::GenCrIn(e) => e.id,
            EnergyData::GenProd(e) => e.id,
        }
    }

    /// Get carrier for this component
    pub fn carrier(&self) -> Carrier {
        match self {
            EnergyData::GenCrIn(e) => e.carrier,
            EnergyData::GenProd(e) => e.carrier,
        }
    }

    /// Get production source (INSITU / COGEN) for this component
    pub fn source(&self) -> Source {
        match self {
            EnergyData::GenCrIn(_) => unreachable!(),
            EnergyData::GenProd(e) => e.source,
        }
    }

    /// Get service for this component
    pub fn service(&self) -> Service {
        match self {
            EnergyData::GenCrIn(e) => e.service,
            EnergyData::GenProd(_) => unreachable!(),
        }
    }

    /// Get comment for this component
    pub fn comment(&self) -> &str {
        match self {
            EnergyData::GenCrIn(e) => &e.comment,
            EnergyData::GenProd(e) => &e.comment,
        }
    }

    /// Is this of kind UsedEnergy?
    pub fn is_used(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => true,
            EnergyData::GenProd(_) => false,
        }
    }

    /// Is this energy of the produced energy kind?
    pub fn is_generated(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(_) => true,
        }
    }

    /// Is this of kind UsedEnergy and destination is an EPB service?
    pub fn is_epb_use(&self) -> bool {
        match self {
            EnergyData::GenCrIn(e) => e.service.is_epb(),
            EnergyData::GenProd(_) => false,
        }
    }

    /// Is this of kind UsedEnergy and destination is a non EPB service (but not GEN)?
    pub fn is_nepb_use(&self) -> bool {
        match self {
            EnergyData::GenCrIn(e) => e.service.is_nepb(),
            EnergyData::GenProd(_) => false,
        }
    }

    /// Is this energy of the onsite produced kind?
    pub fn is_onsite_pr(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(e) => e.source == Source::INSITU,
        }
    }

    /// Is this energy of the cogeneration produced kind?
    pub fn is_cogen_pr(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(e) => e.source == Source::COGENERACION,
        }
    }

    /// Is this a production or use of the electricity carrier?
    pub fn is_electricity(&self) -> bool {
        self.carrier() == Carrier::ELECTRICIDAD
    }

    /// Has this component this service?
    pub fn has_service(&self, srv: Service) -> bool {
        self.service() == srv
    }

    /// Has this component this carrier?
    pub fn has_carrier(&self, carrier: Carrier) -> bool {
        self.carrier() == carrier
    }

    /// Has this component this id?
    pub fn has_id(&self, id: i32) -> bool {
        self.id() == id
    }
}

impl std::fmt::Display for EnergyData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnergyData::GenCrIn(e) => e.fmt(f),
            EnergyData::GenProd(e) => e.fmt(f),
        }
    }
}

impl HasValues for EnergyData {
    fn values(&self) -> &[f32] {
        match self {
            EnergyData::GenCrIn(e) => e.values(),
            EnergyData::GenProd(e) => e.values(),
        }
    }
}
