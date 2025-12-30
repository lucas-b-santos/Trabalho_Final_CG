use wasm_bindgen::prelude::*;
extern crate nalgebra as na;

pub mod pipeline;

use na::{SVector};

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Dados que vão ser usados como índice PRECISAM SER USIZE
struct Obj {
    x: SVector<usize, 8>,
    y: SVector<usize, 8>,
}

#[wasm_bindgen]
pub struct SoftwareRenderer {
    width: usize,
    height: usize,
    obj: Obj,
    // Buffer de cor: 1 inteiro = 4 bytes (R, G, B, A)
    color_buffer: Vec<u32>,
    // Z-Buffer: guarda a profundidade (1.0 = perto, 100.0 = longe, etc)
    z_buffer: Vec<f32>,
}

#[wasm_bindgen]
impl SoftwareRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> SoftwareRenderer {
        let size = width * height;
        // Pixel é lido em little-endian: 0xAABBGGRR
        let mut buffer: Vec<u32> = vec![0xFF000000; size];
        
        let obj = Obj {
            x: SVector::from_row_slice(&[550, 670, 675, 550, 430, 550, 550, 425]),
            y: SVector::from_row_slice(&[600, 658, 540, 485, 658, 721, 600, 540]),
        };

        for i in 0..8 {
            buffer[obj.y[i] * width + obj.x[i]] = 0xFFFFFFFF;
        }

        SoftwareRenderer {
            width,
            height,
            obj,
            color_buffer: buffer, // Preto, Alpha 255
            z_buffer: vec![f32::INFINITY; size],  // Infinito (fundo)
        }
    }

    // Chamado no início de cada frame
    pub fn clear(&mut self) {
        // Reinicia as cores para preto (ou cor de fundo)
        // fill é extremamente otimizado no Rust (usa memset)
        self.color_buffer.fill(0xFF000000);

        // Reinicia o Z-Buffer para "infinito"
        self.z_buffer.fill(f32::INFINITY);
    }

    // Uma função que faz o frame completo de uma vez
    pub fn render_frame(&mut self, dx: usize, dy: usize) {
        self.clear(); // Limpa Z-Buffer e Cor

        self.obj.x += SVector::<usize, 8>::from_element(dx);
        self.obj.y += SVector::<usize, 8>::from_element(dy);

        for i in 0..8 {
            self.color_buffer[self.obj.y[i] * self.width + self.obj.x[i]] = 0xFFFFFFFF;
        }

        // ... Lógica de projeção, rasterização e phong ...
    }

    pub fn get_color_ptr(&self) -> *const u32 {
        self.color_buffer.as_ptr()
    }
}
