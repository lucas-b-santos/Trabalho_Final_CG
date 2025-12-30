extern crate nalgebra as na;

use na::{SMatrix};

pub type Cubo = SMatrix::<f64, 4, 8>; 

pub fn create_cube() -> Cubo {
    Cubo::from_row_slice(&[
        -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, 
         1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,
         1.0,  1.0,  1.0,  1.0, -1.0, -1.0, -1.0, -1.0,
         1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,
    ])
}

