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
Energy performance
==================

Energy performance type as a tuple to represent energy or emission values.
*/

use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use serde::{Serialize, Deserialize};

use crate::error::EpbdError;

/// Tupla que representa los factores de energía primaria renovable, no renovable y de emisión
/// 
/// Energy pairs representing renewable and non renewable energy quantities or factors.
#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RenNrenCo2 {
    /// Renewable energy or factor
    #[serde(serialize_with = "round_serialize_3")]
    pub ren: f32,
    /// Non Renewable energy or factor
    #[serde(serialize_with = "round_serialize_3")]
    pub nren: f32,
    /// Non Renewable energy or factor
    #[serde(serialize_with = "round_serialize_3")]
    pub co2: f32,
}

fn round_serialize_3<S>(x: &f32, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_f32((x * 1000.0).round() / 1000.0)
}

impl RenNrenCo2 {
    /// Default constructor -> { ren: 0.0, nren: 0.0 }
    pub fn new(ren: f32, nren: f32, co2: f32) -> Self {
        Self { ren, nren, co2 }
    }

    /// Total renewable + non renewable energy
    pub fn tot(self) -> f32 {
        self.ren + self.nren
    }

    /// Renewable energy ratio
    pub fn rer(self) -> f32 {
        let tot = self.tot();
        if tot == 0.0 {
            0.0
        } else {
            self.ren / tot
        }
    }
}

// Conversión desde tupla a RenNrenCo2
impl std::convert::From<(f32, f32, f32)> for RenNrenCo2 {
    fn from((ren, nren, co2): (f32, f32, f32)) -> Self {
        Self { ren, nren, co2 }
    }
}

impl fmt::Display for RenNrenCo2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ ren: {:.3}, nren: {:.3}, co2: {:.3} }}",
            self.ren, self.nren, self.co2
        )
    }
}

impl std::str::FromStr for RenNrenCo2 {
    type Err = EpbdError;
    /// Get RenNrenCo2 from
    ///     (number, number, number)
    ///     number, number, number
    ///     { ren: number, nren: number, co2: number }
    fn from_str(s: &str) -> Result<RenNrenCo2, Self::Err> {
        let s = s.trim().trim_matches(|c| c == '(' || c == ')');
        if s.starts_with('{') {
            let mut res = RenNrenCo2::default();
            s.trim_matches(|c| c == '{' || c == '}')
                .split(',')
                .map(|s| s.splitn(2, ':').map(str::trim).collect::<Vec<&str>>())
                .for_each(|v| {
                    let mut it = v.iter();
                    let (key, val) = match (it.next(), it.next()) {
                        (Some(k), Some(v)) => (*k, *v),
                        _ => ("Error", "0.0"),
                    };
                    //let haskey = ["ren", "nren", "co2"].contains(&key);
                    match (key, f32::from_str(val)) {
                        ("ren", Ok(v)) => res.ren = v,
                        ("nren", Ok(v)) => res.nren = v,
                        ("co2", Ok(v)) => res.co2 = v,
                        _ => println!("Algo malo pasa con {}", key),
                    }
                });
            Ok(res)
        } else {
            let vals = s
                .split(',')
                .map(str::trim)
                .map(f32::from_str)
                .collect::<Result<Vec<f32>, _>>()
                .map_err(|_| EpbdError::ParseError(s.into()))?;

            match *vals.as_slice() {
                [ren, nren, co2] => Ok(RenNrenCo2 { ren, nren, co2 }),
                _ => Err(EpbdError::ParseError(s.into())),
            }
        }
    }
}

// The insane amount of boilerplate for ops would be simplified with the implementation
// of the Eye of Sauron in Rustc:
// - https://github.com/arielb1/rfcs/blob/df42b1df220d27876976b54dc93cdcb0b592cad3/text/0000-eye-of-sauron.md
// - https://github.com/rust-lang/rust/issues/44762

// Implement addition
impl Add for RenNrenCo2 {
    type Output = RenNrenCo2;

    fn add(self, other: RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren + other.ren,
            nren: self.nren + other.nren,
            co2: self.co2 + other.co2,
        }
    }
}

impl<'a> Add for &'a RenNrenCo2 {
    type Output = RenNrenCo2;

    fn add(self, other: &RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren + other.ren,
            nren: self.nren + other.nren,
            co2: self.co2 + other.co2,
        }
    }
}

// Implement +=
impl AddAssign for RenNrenCo2 {
    fn add_assign(&mut self, other: RenNrenCo2) {
        *self = RenNrenCo2 {
            ren: self.ren + other.ren,
            nren: self.nren + other.nren,
            co2: self.co2 + other.co2,
        };
    }
}

// Implement substraction
impl Sub for RenNrenCo2 {
    type Output = RenNrenCo2;

    fn sub(self, other: RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren - other.ren,
            nren: self.nren - other.nren,
            co2: self.co2 - other.co2,
        }
    }
}

impl<'a> Sub for &'a RenNrenCo2 {
    type Output = RenNrenCo2;

    fn sub(self, other: &RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren - other.ren,
            nren: self.nren - other.nren,
            co2: self.co2 - other.co2,
        }
    }
}

// Implement -=
impl SubAssign for RenNrenCo2 {
    fn sub_assign(&mut self, other: RenNrenCo2) {
        *self = RenNrenCo2 {
            ren: self.ren - other.ren,
            nren: self.nren - other.nren,
            co2: self.co2 - other.co2,
        };
    }
}

// Implement multiplication by a f32
// rennren * f32
impl Mul<f32> for RenNrenCo2 {
    type Output = RenNrenCo2;

    fn mul(self, rhs: f32) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
            co2: self.co2 * rhs,
        }
    }
}

// rennren * &f32
impl<'a> Mul<&'a f32> for RenNrenCo2 {
    type Output = RenNrenCo2;

    fn mul(self, rhs: &f32) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
            co2: self.co2 * rhs,
        }
    }
}

// &rennren * f32
impl<'a> Mul<f32> for &'a RenNrenCo2 {
    type Output = RenNrenCo2;

    fn mul(self, rhs: f32) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
            co2: self.co2 * rhs,
        }
    }
}

// TODO: &rennren * &f32 -> impl<'a, 'b> Mul<&'b f32> for &'a RenNRenPair

// f32 * rennren
impl Mul<RenNrenCo2> for f32 {
    type Output = RenNrenCo2;

    fn mul(self, rhs: RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self * rhs.ren,
            nren: self * rhs.nren,
            co2: self * rhs.co2,
        }
    }
}

// &f32 * rennren
impl<'a> Mul<RenNrenCo2> for &'a f32 {
    type Output = RenNrenCo2;

    fn mul(self, rhs: RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self * rhs.ren,
            nren: self * rhs.nren,
            co2: self * rhs.co2,
        }
    }
}

// f32 * &rennren
impl<'a> Mul<&'a RenNrenCo2> for f32 {
    type Output = RenNrenCo2;

    fn mul(self, rhs: &RenNrenCo2) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self * rhs.ren,
            nren: self * rhs.nren,
            co2: self * rhs.co2,
        }
    }
}

// TODO: &f32 * &rennren -> impl<'a, 'b> Mul<&'b RenNRenPair> for &'a f32

// Implement RenNren *= f32
impl MulAssign<f32> for RenNrenCo2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = RenNrenCo2 {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
            co2: self.co2 * rhs,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq};

    #[test]
    fn add() {
        assert_eq!(
            RenNrenCo2 {
                ren: 3.0,
                nren: 3.0,
                co2: 3.0
            },
            RenNrenCo2 {
                ren: 1.0,
                nren: 0.0,
                co2: 2.0
            } + RenNrenCo2 {
                ren: 2.0,
                nren: 3.0,
                co2: 1.0
            }
        );
        assert_eq!(
            RenNrenCo2 {
                ren: 3.0,
                nren: 3.0,
                co2: 3.0
            },
            {
                let mut a = RenNrenCo2 {
                    ren: 1.0,
                    nren: 0.0,
                    co2: 2.0,
                };
                a += RenNrenCo2 {
                    ren: 2.0,
                    nren: 3.0,
                    co2: 1.0,
                };
                a
            }
        );
    }
    #[test]
    fn sub() {
        assert_eq!(
            RenNrenCo2 {
                ren: -1.0,
                nren: -3.0,
                co2: 1.0
            },
            RenNrenCo2 {
                ren: 1.0,
                nren: 0.0,
                co2: 2.0
            } - RenNrenCo2 {
                ren: 2.0,
                nren: 3.0,
                co2: 1.0
            }
        );
        assert_eq!(
            RenNrenCo2 {
                ren: -1.0,
                nren: -3.0,
                co2: 1.0
            },
            {
                let mut a = RenNrenCo2 {
                    ren: 1.0,
                    nren: 0.0,
                    co2: 2.0,
                };
                a -= RenNrenCo2 {
                    ren: 2.0,
                    nren: 3.0,
                    co2: 1.0,
                };
                a
            }
        );
    }
    #[test]
    fn display() {
        assert_eq!(
            format!(
                "{}",
                RenNrenCo2 {
                    ren: 1.0,
                    nren: 0.0,
                    co2: 2.0
                }
            ),
            "{ ren: 1.000, nren: 0.000, co2: 2.000 }"
        );
    }

    #[test]
    fn parse() {
        let val = RenNrenCo2 {
            ren: 1.0,
            nren: 0.0,
            co2: 2.0,
        };

        assert_eq!("1.000, 0.000, 2.000".parse::<RenNrenCo2>().unwrap(), val);
        assert_eq!("(1.000, 0.000, 2.000)".parse::<RenNrenCo2>().unwrap(), val);
        assert_eq!(
            "{ ren: 1.000, nren: 0.000, co2: 2.000 }"
                .parse::<RenNrenCo2>()
                .unwrap(),
            val
        );
        assert_eq!(
            "{ co2: 2.000, nren: 0.000, ren: 1.000 }"
                .parse::<RenNrenCo2>()
                .unwrap(),
            val
        );
    }

    #[test]
    fn mul() {
        assert_eq!(
            RenNrenCo2 {
                ren: 2.2,
                nren: 4.4,
                co2: 2.0
            },
            2.0 * RenNrenCo2 {
                ren: 1.1,
                nren: 2.2,
                co2: 1.0
            }
        );
        assert_eq!(
            RenNrenCo2 {
                ren: 2.2,
                nren: 4.4,
                co2: 2.0
            },
            {
                let mut a = RenNrenCo2 {
                    ren: 1.1,
                    nren: 2.2,
                    co2: 1.0,
                };
                a *= 2.0;
                a
            }
        );
    }
}
