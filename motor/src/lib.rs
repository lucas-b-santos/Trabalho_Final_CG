use wasm_bindgen::prelude::*;

// Ativa o hook de pânico para melhor depuração no console do navegador
extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Representa um ponto no espaço 3D
#[derive(Clone, Copy)]
struct Point3D { x: f64, y: f64, z: f64 }

#[wasm_bindgen]
pub struct SimpleEngine {
    points: Vec<Point3D>,
    // NOVO: Um buffer persistente para guardar o resultado 2D
    render_buffer: Vec<f64>, 
}

#[wasm_bindgen]
impl SimpleEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SimpleEngine {
        let points = vec![
            Point3D { x: -1.0, y: -1.0, z: -1.0 },
            Point3D { x:  1.0, y: -1.0, z: -1.0 },
            Point3D { x:  1.0, y:  1.0, z: -1.0 },
            Point3D { x: -1.0, y:  1.0, z: -1.0 },
            Point3D { x: -1.0, y: -1.0, z:  1.0 },
            Point3D { x:  1.0, y: -1.0, z:  1.0 },
            Point3D { x:  1.0, y:  1.0, z:  1.0 },
            Point3D { x: -1.0, y:  1.0, z:  1.0 },
        ];
        
        // Pré-alocamos espaço para 8 pontos * 2 coordenadas (x, y) = 16 floats
        // Isso evita re-alocação durante o loop
        let render_buffer = Vec::with_capacity(16);

        SimpleEngine { points, render_buffer }
    }

    // Agora esta função NÃO retorna nada, ela apenas atualiza o buffer interno
    pub fn update(&mut self, angle: f64, width: f64, height: f64) {
        self.render_buffer.clear(); // Limpa os dados, mas mantém a memória alocada
        
        let fov = 300.0;
        let distance = 4.0;

        for p in &self.points {
            let rot_x = p.x * angle.cos() - p.z * angle.sin();
            let rot_z = p.x * angle.sin() + p.z * angle.cos();
            let rot_y = p.y;

            let mut z_camera = rot_z + distance;
            
            // Proteção contra divisão por zero (pode causar problemas no Canvas)
            if z_camera.abs() < 0.001 { z_camera = 0.001; }

            let mut px = (rot_x / z_camera) * fov;
            let mut py = (rot_y / z_camera) * fov;

            px += width / 2.0;
            py += height / 2.0;

            self.render_buffer.push(px);
            self.render_buffer.push(py);
        }
    }

    // 1. Retorna o PONTEIRO de memória onde começa o buffer
    pub fn get_buffer_ptr(&self) -> *const f64 {
        self.render_buffer.as_ptr()
    }

    // 2. Retorna o TAMANHO do buffer (quantos floats existem)
    pub fn get_buffer_size(&self) -> usize {
        self.render_buffer.len()
    }
}