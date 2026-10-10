#[cfg(test)]
mod solver_tests {
    use crate::solver::{solve_complex_matrix, solve_real_matrix};
    use crate::types::Cx;

    #[test]
    fn test_solve_real_matrix_simple() {
        // 2x2 system: x + y = 3, x - y = 1 => x = 2, y = 1
        let mut a = vec![
            vec![1.0, 1.0],
            vec![1.0, -1.0],
        ];
        let mut b = vec![3.0, 1.0];
        
        let result = solve_real_matrix(&mut a, &mut b).expect("Should solve");
        
        assert!((result[0] - 2.0).abs() < 1e-10, "x = {}", result[0]);
        assert!((result[1] - 1.0).abs() < 1e-10, "y = {}", result[1]);
    }

    #[test]
    fn test_solve_real_matrix_3x3() {
        // 2x + y - z = 8
        // -3x - y + 2z = -11
        // -2x + y + 2z = -3
        // Solution: x = 2, y = 3, z = -1
        let mut a = vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ];
        let mut b = vec![8.0, -11.0, -3.0];
        
        let result = solve_real_matrix(&mut a, &mut b).expect("Should solve");
        
        assert!((result[0] - 2.0).abs() < 1e-10);
        assert!((result[1] - 3.0).abs() < 1e-10);
        assert!((result[2] + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_real_matrix_singular() {
        // Singular matrix: x + y = 1, 2x + 2y = 3 (inconsistent)
        let mut a = vec![
            vec![1.0, 1.0],
            vec![2.0, 2.0],
        ];
        let mut b = vec![1.0, 3.0];
        
        let result = solve_real_matrix(&mut a, &mut b);
        
        assert!(result.is_none(), "Should return None for singular matrix");
    }

    #[test]
    fn test_solve_complex_matrix_simple() {
        // (1+j)x + (1-j)y = 2
        // (1-j)x + (1+j)y = 2
        // Solution: x = 1, y = 1
        let mut a = vec![
            vec![Cx::new(1.0, 1.0), Cx::new(1.0, -1.0)],
            vec![Cx::new(1.0, -1.0), Cx::new(1.0, 1.0)],
        ];
        let mut b = vec![Cx::new(2.0, 0.0), Cx::new(2.0, 0.0)];
        
        let result = solve_complex_matrix(&mut a, &mut b).expect("Should solve");
        
        assert!((result[0].re - 1.0).abs() < 1e-10);
        assert!((result[0].im - 0.0).abs() < 1e-10);
        assert!((result[1].re - 1.0).abs() < 1e-10);
        assert!((result[1].im - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_complex_matrix_rc_circuit() {
        // RC circuit at 1kHz: R = 1k, C = 1uF
        // Impedance: 1000 - j159.15
        let omega = 2.0 * std::f64::consts::PI * 1000.0;
        let r = 1000.0;
        let c = 1e-6;
        
        // Node equation: (Vin - Vout)/R + j*omega*C*Vout = 0
        // Vout*(1/R + j*omega*C) = Vin/R
        let y = Cx::new(1.0/r, omega * c);
        let mut a = vec![vec![y]];
        let mut b = vec![Cx::new(1.0/r, 0.0)];
        
        let result = solve_complex_matrix(&mut a, &mut b).expect("Should solve");
        
        // Vout = (1/R) / (1/R + j*omega*C) = 1 / (1 + j*omega*R*C)
        let expected = Cx::new(1.0, 0.0) / Cx::new(1.0, omega * r * c);
        
        assert!((result[0].re - expected.re).abs() < 1e-6);
        assert!((result[0].im - expected.im).abs() < 1e-6);
    }

    #[test]
    fn test_solve_complex_matrix_singular() {
        let mut a = vec![
            vec![Cx::new(1.0, 0.0), Cx::new(1.0, 0.0)],
            vec![Cx::new(2.0, 0.0), Cx::new(2.0, 0.0)],
        ];
        let mut b = vec![Cx::new(1.0, 0.0), Cx::new(3.0, 0.0)];
        
        let result = solve_complex_matrix(&mut a, &mut b);
        
        assert!(result.is_none(), "Should return None for singular matrix");
    }

    #[test]
    fn test_solve_real_matrix_empty() {
        let mut a: Vec<Vec<f64>> = vec![];
        let mut b: Vec<f64> = vec![];
        
        let result = solve_real_matrix(&mut a, &mut b);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_solve_real_matrix_mismatched_dims() {
        let mut a = vec![vec![1.0, 2.0]];
        let mut b = vec![1.0, 2.0];
        
        let result = solve_real_matrix(&mut a, &mut b);
        
        assert!(result.is_none());
    }
}