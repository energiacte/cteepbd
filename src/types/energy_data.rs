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

use crate::types::{Carrier, GenAux, GenCrIn, GenProd, HasValues, Service, Source};

/// Componentes de energía generada o consumida
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnergyData {
    /// Energía usada (consumida). E_X;Y;in;cr,j;t
    ///
    /// Representa el consumo de energía del vector energético j
    /// para el servicio X en el subsistema Y (e.g. generador i, id=i), para los distintos pasos de cálculo t,
    /// a lo largo del periodo de cálculo. Por ejemplo, E_X;gen,i;in;cr,j;t
    ///
    /// Las cantidades de energía de combustibles son en relación al poder calorífico superior.
    /// Subsistema: generación + almacenamiento
    GenCrIn(GenCrIn),
    /// Energía generada (producida). E_pr;cr,i;t
    ///
    /// Representa la producción de energía del vector energético j (con origen dado en el sistema j)
    /// para los pasos de cálculo t, a lo largo del periodo de cálculo. Por ejemplo, E_pr,j;cr,i;t
    /// Subsistema: generación + almacenamiento
    GenProd(GenProd),
    /// Energía auxiliar (consumida). W_X;Y;aux;t
    ///
    /// Representa el consumo de energía (eléctrica) para usos auxiliares
    /// del servicio X en el subsistema Y (gen, dis, em, alm), para los distintos
    /// pasos de cálculo. Por ejemplo, W_X;gen_i;aux;t
    /// Subsistema: generación + almacenamiento
    GenAux(GenAux),
    // TODO: Energía saliente (entregada o absorbida). Q_X;Y;out
    //
    // Representa la energía entregada o absorbida para el servicio X por los sistemas i
    // pertenecientes al subsistema Y del edificio. Por ejemplo, Q_X_gen_i_out
    // Subsistema: generación + almacenamiento
    // GenOut(xxx)

    // TODO: Pérdidas térmicas no recuperadas Q_X;Y;ls,nrvd (Q_X;Y;ls,nrvd = Q_X;Y;ls - Q_X;Y;ls,rvd)
    //
    // Permite calcular la energía entrante al sistema i para el servicio X en el subsistema Y
    // a partir de la energía saliente Q_X;Y;out
    // como (EN 15316-1, (3)) Q_X;Y;in = Q_X;Y;out + Q_X;Y;ls,nrvd
    // Subsistema: generación + almacenamiento
    // LsNrvd(xxx)

    // Subsistemas de distribución y emisión, sin identificación de sistema?

    // EmOut(xxx)
    // EmAux(xxx)
    // EmLsNrvd(xxx)
    // DisOut(xxx)
    // DisAux(xxx)
    // DisLsNrvd(xxx)
}

impl EnergyData {
    /// Get id for this service
    pub fn id(&self) -> i32 {
        match self {
            EnergyData::GenCrIn(e) => e.id,
            EnergyData::GenProd(e) => e.id,
            EnergyData::GenAux(e) => e.id,
        }
    }

    /// Get carrier for this component
    pub fn carrier(&self) -> Carrier {
        match self {
            EnergyData::GenCrIn(e) => e.carrier,
            EnergyData::GenProd(e) => e.carrier,
            EnergyData::GenAux(_) => Carrier::ELECTRICIDAD,
        }
    }

    /// Get production source (INSITU / COGEN) for this component
    pub fn source(&self) -> Source {
        match self {
            EnergyData::GenCrIn(_) | EnergyData::GenAux(_) => unreachable!(),
            EnergyData::GenProd(e) => e.source,
        }
    }

    /// Get service for this component
    pub fn service(&self) -> Service {
        match self {
            EnergyData::GenCrIn(e) => e.service,
            EnergyData::GenAux(e) => e.service,
            EnergyData::GenProd(_) => unreachable!(),
        }
    }

    /// Get comment for this component
    pub fn comment(&self) -> &str {
        match self {
            EnergyData::GenCrIn(e) => &e.comment,
            EnergyData::GenProd(e) => &e.comment,
            EnergyData::GenAux(e) => &e.comment,
        }
    }

    /// Is this of kind UsedEnergy?
    pub fn is_used(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => true,
            EnergyData::GenProd(_) => false,
            EnergyData::GenAux(_) => false,
        }
    }

    /// Is this energy of the produced energy kind?
    pub fn is_generated(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(_) => true,
            EnergyData::GenAux(_) => false,
        }
    }

    /// Is this energy of the auxiliary energy kind?
    pub fn is_aux(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(_) => false,
            EnergyData::GenAux(_) => true,
        }
    }

    /// Is this of kind UsedEnergy and destination is an EPB service?
    pub fn is_epb_use(&self) -> bool {
        match self {
            EnergyData::GenCrIn(e) => e.service.is_epb(),
            EnergyData::GenProd(_) => false,
            EnergyData::GenAux(e) => e.service.is_epb(),
        }
    }

    /// Is this of kind UsedEnergy and destination is a non EPB service (but not GEN)?
    pub fn is_nepb_use(&self) -> bool {
        match self {
            EnergyData::GenCrIn(e) => e.service.is_nepb(),
            EnergyData::GenProd(_) => false,
            EnergyData::GenAux(e) => e.service.is_nepb(),
        }
    }

    /// Is this energy of the onsite produced kind?
    pub fn is_onsite_pr(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(e) => e.source == Source::INSITU,
            EnergyData::GenAux(_) => false,
        }
    }

    /// Is this energy of the cogeneration produced kind?
    pub fn is_cogen_pr(&self) -> bool {
        match self {
            EnergyData::GenCrIn(_) => false,
            EnergyData::GenProd(e) => e.source == Source::COGEN,
            EnergyData::GenAux(_) => false,
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
            EnergyData::GenAux(e) => e.fmt(f),
        }
    }
}

impl HasValues for EnergyData {
    fn values(&self) -> &[f32] {
        match self {
            EnergyData::GenCrIn(e) => e.values(),
            EnergyData::GenProd(e) => e.values(),
            EnergyData::GenAux(e) => e.values(),
        }
    }
}
