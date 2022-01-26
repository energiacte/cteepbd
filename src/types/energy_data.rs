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

use crate::types::{Carrier, HasValues, ProdOrigin, ProducedEnergy, Service, UsedEnergy};

/// Componentes de energía generada o consumida
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnergyData {
    /// Energía consumida
    UsedEnergy(UsedEnergy),
    /// Energía generada
    ProducedEnergy(ProducedEnergy),
}

impl EnergyData {
    /// Get id for this service
    pub fn id(&self) -> i32 {
        match self {
            EnergyData::UsedEnergy(e) => e.id,
            EnergyData::ProducedEnergy(e) => e.id,
        }
    }

    /// Get carrier for this component
    pub fn carrier(&self) -> Carrier {
        match self {
            EnergyData::UsedEnergy(e) => e.carrier,
            EnergyData::ProducedEnergy(e) => e.carrier,
        }
    }

    /// Get subtype (EPB / NEPB, INSITU / COGEN) for this component
    pub fn csubtype(&self) -> ProdOrigin {
        match self {
            // TODO: eliminar este método
            EnergyData::UsedEnergy(_) => unreachable!(),
            EnergyData::ProducedEnergy(e) => e.csubtype,
        }
    }

    /// Get service for this component
    pub fn service(&self) -> Service {
        match self {
            EnergyData::UsedEnergy(e) => e.service,
            EnergyData::ProducedEnergy(e) => e.service,
        }
    }

    /// Get comment for this component
    pub fn comment(&self) -> &str {
        match self {
            EnergyData::UsedEnergy(e) => &e.comment,
            EnergyData::ProducedEnergy(e) => &e.comment,
        }
    }

    /// Is this of kind UsedEnergy?
    pub fn is_used(&self) -> bool {
        match self {
            EnergyData::UsedEnergy(_) => true,
            EnergyData::ProducedEnergy(_) => false,
        }
    }

    /// Is this of kind UsedEnergy and destination is an EPB service?
    pub fn is_epb_use(&self) -> bool {
        match self {
            // TODO: tendría que tener también e.service != Service::COGEN
            EnergyData::UsedEnergy(e) => e.service != Service::NEPB,
            EnergyData::ProducedEnergy(_) => false,
        }
    }

    /// Is this ProducedEnergy of the onsite generated kind?
    pub fn is_onsite_pr(&self) -> bool {
        match self {
            EnergyData::UsedEnergy(_) => false,
            EnergyData::ProducedEnergy(e) => e.csubtype == ProdOrigin::INSITU,
        }
    }

    /// Is this ProducedEnergy of the cogeneration kind?
    pub fn is_cogen_pr(&self) -> bool {
        match self {
            EnergyData::UsedEnergy(_) => false,
            EnergyData::ProducedEnergy(e) => e.csubtype == ProdOrigin::COGENERACION,
        }
    }

    /// Is this of kind ProducedEnergy?
    pub fn is_generated(&self) -> bool {
        !self.is_used()
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
            EnergyData::UsedEnergy(e) => e.fmt(f),
            EnergyData::ProducedEnergy(e) => e.fmt(f),
        }
    }
}

impl HasValues for EnergyData {
    fn values(&self) -> &[f32] {
        match self {
            EnergyData::UsedEnergy(e) => e.values(),
            EnergyData::ProducedEnergy(e) => e.values(),
        }
    }
}
