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

// Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>,
//            Daniel Jiménez González <dani@ietcc.csic.es>

// -----------------------------------------------------------------------------------
// Vector utilities
// -----------------------------------------------------------------------------------

use num::{Float, Num, Zero};
use std::iter::{FromIterator, Sum};
use std::ops::Deref;

/// Elementwise sum res[i] = vec1[i] + vec2[i] + ... + vecj[i]
pub fn veclistsum<T, U>(veclist: &[&U]) -> Vec<T>
where
    T: Num + Copy,
    U: FromIterator<T> + Deref<Target = [T]>,
{
    let maxlen: usize = veclist.iter().map(|lst| lst.len()).max().unwrap_or(0_usize);
    veclist.iter().fold(vec![Zero::zero()], |acc, ref x| {
        (0..maxlen)
            .map(|idx| {
                *acc.get(idx).unwrap_or(&Zero::zero()) + *x.get(idx).unwrap_or(&Zero::zero())
            })
            .collect()
    })
}

/// Elementwise minimum min res[i] = min(vec1[i], vec2[i])
pub fn vecvecmin<T, U>(vec1: &U, vec2: &U) -> Vec<T>
where
    T: Float,
    U: Deref<Target = [T]>,
{
    vec1.iter()
        .enumerate()
        .map(|(ii, el)| el.min(*vec2.get(ii).unwrap_or(&Zero::zero())))
        .collect()
}

/// Elementwise sum of arrays
pub fn vecvecsum<T, U>(vec1: &U, vec2: &U) -> Vec<T>
where
    T: Float,
    U: Deref<Target = [T]>,
{
    vec1.iter()
        .enumerate()
        .map(|(ii, el)| *el + *vec2.get(ii).unwrap_or(&Zero::zero()))
        .collect()
}

/// Elementwise difference res[i] = vec1[i] - vec2[i]
pub fn vecvecdif<T, U>(vec1: &U, vec2: &U) -> Vec<T>
where
    T: Float,
    U: Deref<Target = [T]>,
{
    vec1.iter()
        .enumerate()
        .map(|(ii, el)| *el - *vec2.get(ii).unwrap_or(&Zero::zero()))
        .collect()
}

/// Elementwise multiplication res[i] = vec1[i] * vec2[i]
pub fn vecvecmul<T, U>(vec1: &U, vec2: &U) -> Vec<T>
where
    T: Float,
    U: Deref<Target = [T]>,
{
    vec1.iter()
        .enumerate()
        .map(|(ii, el)| *el * *vec2.get(ii).unwrap_or(&Zero::zero()))
        .collect()
}

// TODO: Elementwise division res[i] = vec1[i] / vec2[i] for each vec1[i] != 0 if vec2 != 0

/// Multiply vector by scalar
pub fn veckmul<T, U>(vec1: &U, k: T) -> Vec<T>
where
    T: Float,
    U: Deref<Target = [T]>,
{
    vec1.iter().map(|el| *el * k).collect()
}

/// Sum all elements in a vector
pub fn vecsum<'a, T, U>(vec: &'a U) -> T
where
    T: Float + Sum<&'a T> + 'a,
    U: Deref<Target = [T]>,
{
    vec.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vecops_veclistsum() {
        assert_eq!(
            vec![6.0, 6.0, 6.0],
            veclistsum(&[
                &vec![1.0, 1.0, 1.0],
                &vec![2.0, 2.0, 2.0],
                &vec![3.0, 3.0, 3.0],
            ])
        );
    }

    #[test]
    fn vecops_vecvecmin() {
        assert_eq!(
            vec![2.0, 1.0, 2.0],
            vecvecmin(&[2.0, 2.0, 2.0], &[4.0, 1.0, 2.0])
        );
    }

    #[test]
    fn vecops_vecvecsum() {
        assert_eq!(
            vec![4.0, 4.0, 4.0],
            vecvecsum(&[2.0, 1.0, 3.0], &[2.0, 3.0, 1.0])
        );
    }

    #[test]
    fn vecops_vecvecdif() {
        assert_eq!(
            vec![1.0, 1.0, 1.0],
            vecvecdif(&[2.0, 3.0, 4.0], &[1.0, 2.0, 3.0])
        );
    }

    #[test]
    fn vecops_vecvecmul() {
        assert_eq!(
            vec![1.0, 6.0, 4.0],
            vecvecmul(&[1.0, 3.0, 2.0], &[1.0, 2.0, 2.0])
        );
    }

    #[test]
    fn vecops_veckmul() {
        assert_eq!(vec![2.0, 4.0, 6.0], veckmul(&[1.0, 2.0, 3.0], 2.0));
    }

    #[test]
    fn vecops_vecsum() {
        assert_eq!(9.0, vecsum(&[2.0, 3.0, 4.0]));
    }
}
