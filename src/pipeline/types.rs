extern crate nalgebra as na;
use na::{SMatrix, Vector3, Vector4};

pub use crate::pipeline::{HEIGHT, WIDTH};

/// Um vértice com coordenadas homogêneas e vetor normal associado
#[derive(Debug, Clone, Copy)]
pub struct Vertex { 
    pub cords: Vector4<f32>, // cordenadas homogêneas do vértice
    pub normal: Vector3<f32>, // vetor normal associado ao vértice
}

/// Uma face composta por vértices, vetor normal e centroide
pub struct Face {
    pub vertices: Vec<Vertex>, // vértices que compõem a face
    pub normal: Vector3<f32>,  // vetor normal da face
    pub centroid: Vector3<f32>, // centroide da face
}

impl Face {
    pub fn z_avg(&self) -> f32 {
        let mut z_sum = 0.0;
        for vertex in &self.vertices {
            z_sum += vertex.cords.z;
        }
        z_sum / self.vertices.len() as f32
    }

    /// verifica se o ponto (x, y) em SRT está dentro da face
    pub fn is_point_in(&self, x: f32, y: f32) -> bool {
        let mut inside = false;
        if self.vertices.len() < 4 {
            return inside; // Não é um quadrado válido
        }
        let mut j = self.vertices.len() - 1;

        for i in 0..self.vertices.len() {
            let xi = self.vertices[i].cords.x;
            let yi = self.vertices[i].cords.y;
            let xj = self.vertices[j].cords.x;
            let yj = self.vertices[j].cords.y;
                
            let intersect = (yi > y) != (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi;

            if intersect { inside = !inside; }

            j = i;
        }

        inside
    }
}

/// Uma entrada na scanline com as coordenadas
#[derive(Debug, Clone, Copy)]
pub struct ConstantEntry {
    pub x: f32,          // coordenada x do ponto na scanline
    pub z: f32,          // coordenada z do ponto na scanline
}

/// Uma entrada na scanline com coordenadas e vetor normal
#[derive(Debug, Clone, Copy)]
pub struct PhongEntry {
    pub x: f32,          // coordenada x do ponto na scanline
    pub z: f32,          // coordenada z do ponto na scanline
    pub normal: Vector3<f32>, // vetor normal no ponto
}

/// Matriz 4x8 para armazenar os pontos do objeto
pub type RawObj = SMatrix<f32, 4, 8>; 

type LinearRGB = [f32; 3]; // Representação linear de cor RGB (0.0 a 1.0)

/// Parâmetros da cena
pub struct SceneParams {
    pub vrp: Vector3<f32>,     // Posição da câmera
    pub view_up: Vector3<f32>, // Vetor view-up
    pub p: Vector3<f32>,       // Ponto focal
   
    // limites da viewport em SRT  
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,

    // Parâmetros de projeção
    pub su: f32,
    pub sv: f32,
    pub near: f32,
    pub far: f32,
    pub dp: f32,
    pub cu: f32,
    pub cv: f32,
    
    pub lamp_pos: Vector3<f32>, // Posição da lâmpada
    pub i_lamp: LinearRGB,   // Intensidade da lâmpada
    pub i_amb: LinearRGB,    // Intensidade da luz ambiente

    pub use_phong: bool, // Indica se o modelo de iluminação Phong deve ser usado
}

/// Material do objeto, composto dos coeficientes de reflexão (RGB) e o expoente de brilho
pub struct Material {
    pub ka: LinearRGB, // Coeficiente de reflexão ambiente
    pub kd: LinearRGB, // Coeficiente de reflexão difusa
    pub ks: LinearRGB, // Coeficiente de reflexão especular
    pub n: f32,           // Exponente de brilho especular
}

/// Um cubo no universo com seus parâmetros de material
pub struct UCube {
    pub raw: RawObj,
    pub params: Material
}

impl Default for UCube {
    fn default() -> Self {
        Self {
            raw: RawObj::from_row_slice(&[
            -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, 
            -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0,
            -1.0, -1.0, -1.0, -1.0,  1.0,  1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,
            ]),
            params: Material {
                ka: [0.1, 0.1, 0.1],
                kd: [0.7, 0.7, 0.7],
                ks: [0.5, 0.5, 0.5],
                n: 10.0,
            },
        }
    }
}

impl UCube {
    pub fn centroid(&self) -> Vector3<f32> {
        let mut centroid = Vector3::new(0.0, 0.0, 0.0);
        for i in 0..8 {
            centroid.x += self.raw[(0, i)];
            centroid.y += self.raw[(1, i)];
            centroid.z += self.raw[(2, i)];
        }
        centroid /= 8.0;
        centroid
    }

    // como a translação é usada em outras operações, resolvi colocar ela aqui
    pub fn translate(&mut self, translation: Vector3<f32>) {
        for i in 0..8 {
            self.raw[(0, i)] += translation.x;
            self.raw[(1, i)] += translation.y;
            self.raw[(2, i)] += translation.z;
        }
    }
}

impl Default for SceneParams {
    fn default() -> Self {
        Self {
            vrp : Vector3::new(15.0, 15.0, 15.0),
            view_up: Vector3::new(0.0, 1.0, 0.0),
            p: Vector3::new(1.0, 1.0, 1.0),
            u_min: 0.0,
            v_min: 0.0,
            u_max: WIDTH as f32 - 1.0,
            v_max: HEIGHT as f32 - 1.0,
            su: 10.0,
            sv: 8.0,
            near: 20.0,
            far: 120.0,
            dp: 50.0,
            cu: 0.0,
            cv: 0.0,
            lamp_pos: Vector3::new(10.0, 10.0, 10.0),
            i_lamp: [1.0, 1.0, 1.0],
            i_amb: [0.2, 0.2, 0.2],
            use_phong: true,
            }
    }
}

/// Parâmetros de transformação de um objeto
pub struct ObjectTransform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>, 
    pub scale: f32,
}

impl Default for ObjectTransform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}