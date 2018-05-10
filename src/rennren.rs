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
use std::ops::{Add, Mul, Sub};

/// Energy pairs representing renewable and non renewable energy quantities or factors.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct RenNren {
    /// Renewable energy or factor
    pub ren: f32,
    /// Non Renewable energy or factor
    pub nren: f32,
}

impl RenNren {
    // Default constructor -> { ren: 0.0, nren: 0.0 }
    pub fn new() -> Self {
        Default::default()
    }

    /// Total renewable + non renewable energy
    pub fn tot(&self) -> f32 {
        self.ren + self.nren
    }

    /// Renewable energy ratio
    pub fn rer(&self) -> f32 {
        let tot = self.tot();
        if tot == 0.0 {
            0.0
        } else {
            self.ren / tot
        }
    }
}

impl fmt::Display for RenNren {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ ren: {:.3}, nren: {:.3} }}", self.ren, self.nren)
    }
}

// The insane amount of boilerplate for ops would be simplified with the implementation
// of the Eye of Sauron in Rustc:
// - https://github.com/arielb1/rfcs/blob/df42b1df220d27876976b54dc93cdcb0b592cad3/text/0000-eye-of-sauron.md
// - https://github.com/rust-lang/rust/issues/44762

// Implement addition
impl Add for RenNren {
    type Output = RenNren;

    fn add(self, other: RenNren) -> RenNren {
        RenNren {
            ren: self.ren + other.ren,
            nren: self.nren + other.nren,
        }
    }
}

impl<'a> Add for &'a RenNren {
    type Output = RenNren;

    fn add(self, other: &RenNren) -> RenNren {
        RenNren {
            ren: self.ren + other.ren,
            nren: self.nren + other.nren,
        }
    }
}

// Implement substraction
impl Sub for RenNren {
    type Output = RenNren;

    fn sub(self, other: RenNren) -> RenNren {
        RenNren {
            ren: self.ren - other.ren,
            nren: self.nren - other.nren,
        }
    }
}

impl<'a> Sub for &'a RenNren {
    type Output = RenNren;

    fn sub(self, other: &RenNren) -> RenNren {
        RenNren {
            ren: self.ren - other.ren,
            nren: self.nren - other.nren,
        }
    }
}

// Implement multiplication by a f32
// rennren * f32
impl Mul<f32> for RenNren {
    type Output = RenNren;

    fn mul(self, rhs: f32) -> RenNren {
        RenNren {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
        }
    }
}

// rennren * &f32
impl<'a> Mul<&'a f32> for RenNren {
    type Output = RenNren;

    fn mul(self, rhs: &f32) -> RenNren {
        RenNren {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
        }
    }
}

// &rennren * f32
impl<'a> Mul<f32> for &'a RenNren {
    type Output = RenNren;

    fn mul(self, rhs: f32) -> RenNren {
        RenNren {
            ren: self.ren * rhs,
            nren: self.nren * rhs,
        }
    }
}

// TODO: &rennren * &f32 -> impl<'a, 'b> Mul<&'b f32> for &'a RenNRenPair

// f32 * rennren
impl Mul<RenNren> for f32 {
    type Output = RenNren;

    fn mul(self, rhs: RenNren) -> RenNren {
        RenNren {
            ren: self * rhs.ren,
            nren: self * rhs.nren,
        }
    }
}

// &f32 * rennren
impl<'a> Mul<RenNren> for &'a f32 {
    type Output = RenNren;

    fn mul(self, rhs: RenNren) -> RenNren {
        RenNren {
            ren: self * rhs.ren,
            nren: self * rhs.nren,
        }
    }
}

// f32 * &rennren
impl<'a> Mul<&'a RenNren> for f32 {
    type Output = RenNren;

    fn mul(self, rhs: &RenNren) -> RenNren {
        RenNren {
            ren: self * rhs.ren,
            nren: self * rhs.nren,
        }
    }
}

// TODO: &f32 * &rennren -> impl<'a, 'b> Mul<&'b RenNRenPair> for &'a f32

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn RenNren() {
        assert_eq!(
            RenNren {
                ren: 3.0,
                nren: 3.0,
            },
            RenNren {
                ren: 1.0,
                nren: 0.0,
            } + RenNren {
                ren: 2.0,
                nren: 3.0,
            }
        );
        assert_eq!(
            RenNren {
                ren: -1.0,
                nren: -3.0,
            },
            RenNren {
                ren: 1.0,
                nren: 0.0,
            } - RenNren {
                ren: 2.0,
                nren: 3.0,
            }
        );
        assert_eq!(
            format!(
                "{}",
                RenNren {
                    ren: 1.0,
                    nren: 0.0,
                }
            ),
            "{ ren: 1.000, nren: 0.000 }"
        );
        assert_eq!(
            RenNren {
                ren: 2.2,
                nren: 4.4,
            },
            2.0 * RenNren {
                ren: 1.1,
                nren: 2.2,
            }
        );
    }
}
