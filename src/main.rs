use hound::WavReader;
use ndarray::{Array1, Array2};
use serde::Serialize;

#[derive(Serialize)]
struct CircomInput {
    coeffs: Vec<f64>,
    x: f64,
    y: f64,
}

fn load_wav(file_path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(file_path)?;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();
    if samples.is_empty() {
        return Err("WAV file contains no audio data.".into());
    }
    Ok(samples)
}

fn solve_linear_system(mut a: Array2<f64>, mut b: Array1<f64>) -> Result<Array1<f64>, String> {
    let n = a.nrows();

    for i in 0..n {
        let mut max_row = i;
        for k in (i + 1)..n {
            if a[[k, i]].abs() > a[[max_row, i]].abs() {
                max_row = k;
            }
        }
        if a[[max_row, i]] == 0.0 {
            return Err("Matrix is singular and cannot be solved.".to_string());
        }

        if i != max_row {
            for col in 0..a.ncols() {
                a[[i, col]] = a[[i, col]] + a[[max_row, col]];
                a[[max_row, col]] = a[[i, col]] - a[[max_row, col]];
                a[[i, col]] = a[[i, col]] - a[[max_row, col]];
            }
        }

        b[i] = b[i] + b[max_row];
        b[max_row] = b[i] - b[max_row];
        b[i] = b[i] - b[max_row];

        for k in (i + 1)..n {
            let factor = a[[k, i]] / a[[i, i]];
            for j in i..n {
                a[[k, j]] -= factor * a[[i, j]];
            }
            b[k] -= factor * b[i];
        }
    }

    let mut x = Array1::<f64>::zeros(n);
    for i in (0..n).rev() {
        x[i] = b[i];
        for j in (i + 1)..n {
            x[i] -= a[[i, j]] * x[j];
        }
        x[i] /= a[[i, i]];
    }

    Ok(x)
}

fn fit_polynomial(samples: &[f32], degree: usize) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let n = samples.len();
    let x: Array1<f64> = Array1::from_iter((0..n).map(|i| i as f64));
    let y: Array1<f64> = Array1::from_iter(samples.iter().map(|&s| s as f64));

    let x_max = x.iter().cloned().fold(f64::MIN, f64::max);
    let x_min = x.iter().cloned().fold(f64::MAX, f64::min);
    let x = x.mapv(|xi| (xi - x_min) / (x_max - x_min));

    let mut vandermonde = Array2::<f64>::zeros((n, degree + 1));
    for i in 0..n {
        for j in 0..=degree {
            vandermonde[[i, j]] = x[i].powi(j as i32);
        }
    }

    let lhs = vandermonde.t().dot(&vandermonde); 
    let lambda = 1e-6; 
    let lhs = &lhs + lambda * Array2::eye(lhs.nrows()); 
    let rhs = vandermonde.t().dot(&y); // V^T * y

    let coefficients = solve_linear_system(lhs, rhs)?;
    Ok(coefficients.to_vec())
}

fn evaluate_polynomial(coeffs: &[f64], x: f64) -> f64 {
    coeffs
        .iter()
        .enumerate()
        .map(|(i, &c)| c * x.powi(i as i32))
        .sum()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "sample.wav";
    println!("Reading WAV file: {}", file_path);

    let samples = load_wav(file_path)?;
    println!("Loaded {} audio samples.", samples.len());

    let degree = 5;
    let coeffs = fit_polynomial(&samples, degree)?;
    println!("Fitted polynomial coefficients: {:?}", coeffs);

    let x = 10.0;
    let y = evaluate_polynomial(&coeffs, x);

    let circom_input = CircomInput { coeffs, x, y };

    let output_file = "circom_input.json";
    let json = serde_json::to_string_pretty(&circom_input)?;
    std::fs::write(output_file, json)?;
    println!("Circom input saved to {}", output_file);

    Ok(())
}
