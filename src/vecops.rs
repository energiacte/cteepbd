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
Vector utilities
================

Helper utilities for vector handling, mostly elementwise ops.
*/

use num::{Float, Zero};
use std::iter::Sum;
use std::ops::Mul;

/// Elementwise sum res[i] = vec1[i] + vec2[i] + ... + vecj[i]
pub fn veclistsum<T: Float>(veclist: &[&[T]]) -> Vec<T> {
    let maxlen: usize = veclist.iter().map(|lst| lst.len()).max().unwrap_or(0_usize);
    veclist.iter().fold(vec![Zero::zero()], |acc, x| {
        (0..maxlen)
            .map(|idx| {
                *acc.get(idx).unwrap_or(&Zero::zero()) + *x.get(idx).unwrap_or(&Zero::zero())
            })
            .collect()
    })
}

/// Elementwise minimum min res[i] = min(vec1[i], vec2[i])
pub fn vecvecmin<T: Float>(vec1: &[T], vec2: &[T]) -> Vec<T> {
    assert_eq!(vec1.len(), vec2.len());
    vec1.iter()
        .zip(vec2.iter())
        .map(|(a, b)| a.min(*b))
        .collect()
}

/// Elementwise sum of arrays
pub fn vecvecsum<T: Float>(vec1: &[T], vec2: &[T]) -> Vec<T> {
    assert_eq!(vec1.len(), vec2.len());
    vec1.iter().zip(vec2.iter()).map(|(a, b)| *a + *b).collect()
}

/// Elementwise difference res[i] = vec1[i] - vec2[i]
pub fn vecvecdif<T: Float>(vec1: &[T], vec2: &[T]) -> Vec<T> {
    assert_eq!(vec1.len(), vec2.len());
    vec1.iter().zip(vec2.iter()).map(|(a, b)| *a - *b).collect()
}

/// Elementwise multiplication res[i] = vec1[i] * vec2[i]
pub fn vecvecmul<T: Float>(vec1: &[T], vec2: &[T]) -> Vec<T> {
    assert_eq!(vec1.len(), vec2.len());
    vec1.iter().zip(vec2.iter()).map(|(a, b)| *a * *b).collect()
}

/// Multiply vector by scalar
pub fn veckmul<T, I>(iter: I, k: T) -> Vec<T>
where
    T: Float,
    I: IntoIterator,
    I::Item: Mul<T, Output = T>,
{
    iter.into_iter().map(|el| el * k).collect()
}

/// Sum all elements in a vector
pub fn vecsum<'a, T>(vec: &'a [T]) -> T
where
    T: Float + Sum<&'a T> + 'a,
{
    vec.iter().sum()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::useless_vec)]
    use super::*;

    #[test]
    fn vecops_veclistsum() {
        assert_eq!(
            vec![6.0, 6.0, 6.0],
            veclistsum(&[&[1.0, 1.0, 1.0], &[2.0, 2.0, 2.0], &[3.0, 3.0, 3.0]])
        );
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
        assert_eq!(vec![2.0, 4.0, 6.0], veckmul(vec![1.0, 2.0, 3.0], 2.0));
    }

    #[test]
    fn vecops_vecsum() {
        assert!(f32::abs(9.0 - vecsum(&[2.0, 3.0, 4.0])) < f32::EPSILON);
    }
}
