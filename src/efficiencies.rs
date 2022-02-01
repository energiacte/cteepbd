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

/*!
Cálculo de eficiencias energéticas de sistemas y subsistemas
============================================================

Subsistemas (Y): GEN+ALM, DIS, EM

- Pensar cómo encaja distribución y almacenamiento? DisIn, DisOut, DisAux, EmIn, EmOut, EmAux
    - GEN+ALM: CONSUMO (GenCrIn), PRODUCCION (GenProd), CARGA (GenOut), AUX (GenAux), PERDIDASNR (GenNrvdLs)
    - DIS: DisOut, DisAux, DisNrvdLs
    - EM: EmOut, EmAux, EmNrvdLs
    - NGEN? (DIS+EM)? Ver hoja de cálculo epb.center ISO_DIS_52000-1_SS_2015_05_13.xlsm
    - El consumo AUX se trata igual que el consumo eléctrico a efectos de balance pero no a efectos de cálculo de los rendimientos por subsistemas eff_i
    - Definimos las pérdidas por subsistema? y Las pérdidas recuperadas? O por sistema (id=0)?

Cálculo de las eficiencias de los sistemas por subsistema (generación+almacenamiento, distribución, emisión) y servicio (cal, ref?, acs)

Cálculo genérico de la eficiencia según EN 15613-1:2018 (33):
- eff_i = (Q_i_out + f_i · E_el_i_out) / (Q_i_in + f_i · W_i_aux);
    - A conventional weighting factor, f_i=2,5 is used to add up heat and electricity (for comparability reasons)
    - Q_i_in doesn't include heat captured in the evaporator from the heat source for heap pumps -> we get annual COP
    - i = gen+sto, dis, em

- Generic system balance (15613-1, (3)): Q_X_Y_in = Q_X_Y_out + Q_X_ls - Q_X_Y_ls_rvd
    - Q_X_Y_ls_rvd are recovered losses, Q_X_Y_nrvd = Q_X_ls - Q_X_Y_ls_rvd

- System balance for heat pumps (15613-4-2, (1), DWH (cal + acs)):
    - E_X_Y_in \* COP = Q_X_Y_out + Q_X_ls_tot - Q_X_Y_in (ambiente) - f_Y;aux;ls;rvd · W_X_Y_aux (EN 15316-4-2:2019, formula 1, DHW, HEATING)
    - Energy_use \* COP = Energy_out + Energy_losses - (Energy_from_heat_source_input + Energy_from_recovered_aux_energy_not_accounted_for_in_COP_input)
    - This is: electr. o combustible \* COP = energía entregada + pérdidas - energía aportada medioambiente (fuente de calor) - pérdidas recuperables
    - Q_X_Y_in = E_X_Y_in \* COP en términos de la EN 15316-1 (1). No considera energía ambiente capturada por la bomba.

- Para refrigeración ver UNE EN 14825=2016

TODO:
- Definir salidas y datos de entrada necesarios
    - Subsistemas: GEN+ALM, DIS, EM
    - Por servicios: CAL, REF, ACS, TODOS
- Precisar cálculo de agregaciones de sistemas
- Aclarar cálculo de eficiencias en refrigeración

*/

/// Compute system and or subsystem efficiencies
pub fn efficiencies () {
    todo!()
}