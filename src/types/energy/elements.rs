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

use super::{EAux, EOut, EProd, EUsed};
use crate::types::{Carrier, HasValues, Service, Source, ProdSource};

/// Componentes de energía generada, consumida, auxiliar o saliente (entregada/absorbida)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Energy {
    /// Energía generada (producida). E_pr;cr,i;t
    ///
    /// Representa la producción de energía del vector energético j
    /// (con origen dado en el sistema j) para los pasos de cálculo t,
    /// a lo largo del periodo de cálculo. Ej. E_pr,j;cr,i;t
    Prod(EProd),
    /// Energía usada (consumida). E_X;Y;in;cr,j;t
    ///
    /// Representa el consumo de energía del vector energético j
    /// para el servicio X en el subsistema Y y sistema i, (id=i),
    /// para los distintos pasos de cálculo t,
    /// a lo largo del periodo de cálculo. Ej. E_X;gen,i;in;cr,j;t
    ///
    /// Las cantidades de energía de combustibles son en relación al poder calorífico superior.
    Used(EUsed),
    /// Energía auxiliar (consumida). W_X;Y;aux;t
    ///
    /// Representa el consumo de energía (eléctrica) para usos auxiliares
    /// del servicio X en el subsistema Y y sistema i (id=i),
    /// para los distintos pasos de cálculo. Ej. W_X;gen_i;aux;t
    Aux(EAux),
    /// Energía saliente (entregada o absorbida). Q_X;Y;out
    ///
    /// Representa la energía térmica entregada o absorbida para el servicio X por los sistemas i
    /// pertenecientes al subsistema Y  del edificio. Ej. Q_X;gen,i;out
    Out(EOut),
}

impl Energy {
    /// Get id for this service
    pub fn id(&self) -> i32 {
        match self {
            Energy::Prod(e) => e.id,
            Energy::Used(e) => e.id,
            Energy::Aux(e) => e.id,
            Energy::Out(e) => e.id,
        }
    }

    /// Get carrier for this component
    pub fn carrier(&self) -> Carrier {
        match self {
            Energy::Prod(e) => e.source.into(),
            Energy::Used(e) => e.carrier,
            Energy::Aux(_) => Carrier::ELECTRICIDAD,
            Energy::Out(_) => unreachable!(),
        }
    }

    /// Get production source (INSITU / COGEN) for this component
    pub fn source(&self) -> Source {
        match self {
            Energy::Prod(e) => e.source.into(),
            Energy::Used(_) | Energy::Aux(_) | Energy::Out(_) => {
                unreachable!()
            }
        }
    }

    /// Get service for this component
    pub fn service(&self) -> Service {
        match self {
            Energy::Prod(_) => unreachable!(),
            Energy::Used(e) => e.service,
            Energy::Aux(e) => e.service,
            Energy::Out(e) => e.service,
        }
    }

    /// Get comment for this component
    pub fn comment(&self) -> &str {
        match self {
            Energy::Prod(e) => &e.comment,
            Energy::Used(e) => &e.comment,
            Energy::Aux(e) => &e.comment,
            Energy::Out(e) => &e.comment,
        }
    }

    /// Is this of kind UsedEnergy?
    pub fn is_used(&self) -> bool {
        match self {
            Energy::Prod(_) => false,
            Energy::Used(_) => true,
            Energy::Aux(_) => false,
            Energy::Out(_) => false,
        }
    }

    /// Is this energy of the produced energy kind?
    pub fn is_generated(&self) -> bool {
        match self {
            Energy::Prod(_) => true,
            Energy::Used(_) => false,
            Energy::Aux(_) => false,
            Energy::Out(_) => false,
        }
    }

    /// Is this energy of the auxiliary energy kind?
    pub fn is_aux(&self) -> bool {
        match self {
            Energy::Prod(_) => false,
            Energy::Used(_) => false,
            Energy::Aux(_) => true,
            Energy::Out(_) => false,
        }
    }

    /// Is this energy of the output energy kind?
    pub fn is_out(&self) -> bool {
        match self {
            Energy::Prod(_) => false,
            Energy::Used(_) => false,
            Energy::Aux(_) => false,
            Energy::Out(_) => true,
        }
    }

    /// Is this of kind UsedEnergy and destination is an EPB service?
    pub fn is_epb_use(&self) -> bool {
        match self {
            Energy::Prod(_) => false,
            Energy::Used(e) => e.service.is_epb(),
            Energy::Aux(e) => e.service.is_epb(),
            Energy::Out(_) => false,
        }
    }

    /// Is this of kind UsedEnergy and destination is a non EPB service (but not GEN)?
    pub fn is_nepb_use(&self) -> bool {
        match self {
            Energy::Prod(_) => false,
            Energy::Used(e) => e.service.is_nepb(),
            Energy::Aux(e) => e.service.is_nepb(),
            Energy::Out(_) => false,
        }
    }

    /// Is this energy of the onsite produced kind?
    pub fn is_onsite_pr(&self) -> bool {
        match self {
            // TODO: Revisar esto...
            Energy::Prod(e) => e.source != ProdSource::EL_COGEN,
            Energy::Used(_) => false,
            Energy::Aux(_) => false,
            Energy::Out(_) => false,
        }
    }

    /// Is this energy of the cogeneration produced kind?
    pub fn is_cogen_pr(&self) -> bool {
        match self {
            Energy::Prod(e) => e.source == ProdSource::EL_COGEN,
            Energy::Used(_) => false,
            Energy::Aux(_) => false,
            Energy::Out(_) => false,
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

impl std::fmt::Display for Energy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Energy::Prod(e) => e.fmt(f),
            Energy::Used(e) => e.fmt(f),
            Energy::Aux(e) => e.fmt(f),
            Energy::Out(e) => e.fmt(f),
        }
    }
}

impl HasValues for Energy {
    fn values(&self) -> &[f32] {
        match self {
            Energy::Prod(e) => e.values(),
            Energy::Used(e) => e.values(),
            Energy::Aux(e) => e.values(),
            Energy::Out(e) => e.values(),
        }
    }
}
