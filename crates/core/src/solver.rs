use crate::types::Cx;

pub fn solve_complex_matrix(a: &mut Vec<Vec<Cx>>, b: &mut Vec<Cx>) -> Option<Vec<Cx>> {
    let n = a.len();
    if n == 0 || b.len() != n {
        return None;
    }
    for i in 0..n {
        if a[i].len() != n {
            return None;
        }
    }

    for k in 0..n {
        let mut max_row = k;
        let mut max_val = a[k][k].norm();
        for i in (k + 1)..n {
            let val = a[i][k].norm();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }
        if max_val < 1e-15 {
            return None;
        }
        if max_row != k {
            a.swap(k, max_row);
            b.swap(k, max_row);
        }
        let pivot = a[k][k];
        for j in k..n {
            a[k][j] /= pivot;
        }
        b[k] /= pivot;
        for i in 0..n {
            if i != k {
                let factor = a[i][k];
                let pivot_row = a[k].clone();
                let pivot_b = b[k];
                for j in k..n {
                    a[i][j] -= factor * pivot_row[j];
                }
                b[i] -= factor * pivot_b;
            }
        }
    }
    Some(b.clone())
}

pub fn solve_real_matrix(a: &mut Vec<Vec<f64>>, b: &mut Vec<f64>) -> Option<Vec<f64>> {
    let n = a.len();
    if n == 0 || b.len() != n {
        return None;
    }
    for i in 0..n {
        if a[i].len() != n {
            return None;
        }
    }

    for k in 0..n {
        let mut max_row = k;
        let mut max_val = a[k][k].abs();
        for i in (k + 1)..n {
            let val = a[i][k].abs();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }
        if max_val < 1e-15 {
            return None;
        }
        if max_row != k {
            a.swap(k, max_row);
            b.swap(k, max_row);
        }
        let pivot = a[k][k];
        for j in k..n {
            a[k][j] /= pivot;
        }
        b[k] /= pivot;
        for i in 0..n {
            if i != k {
                let factor = a[i][k];
                let pivot_row = a[k].clone();
                let pivot_b = b[k];
                for j in k..n {
                    a[i][j] -= factor * pivot_row[j];
                }
                b[i] -= factor * pivot_b;
            }
        }
    }
    Some(b.clone())
}