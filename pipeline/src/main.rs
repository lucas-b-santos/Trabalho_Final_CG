extern crate nalgebra as na;

use std::{f32::INFINITY, vec};

use na::{Matrix4, SMatrix, Vector3, Vector4};

macro_rules! value {
    () => {
        1.0
    };
}

#[derive(Debug, Clone, Copy)]
struct Vertex {
    cords: Vector4<f32>,
    normal: Vector3<f32>, 
}

type RawObj = SMatrix<f32, 4, 8>; // Matriz 4x8 para armazenar os pontos do objeto

// Função genérica que recorta um polígono contra UM plano definido por 'boundary_check'
fn clip_against_plane<F>(vertices: &[Vertex], boundary_check: F) -> Vec<Vertex>
where
    F: Fn(Vector4<f32>) -> f32, // Retorna > 0 se dentro, < 0 se fora
{
    let mut output_list = Vec::with_capacity(10);

    if vertices.is_empty() {
        return output_list;
    }

    for i in 0..vertices.len() {
        let curr = vertices[i];
        let prev = vertices[(i + vertices.len() - 1) % vertices.len()];

        let bc_curr = boundary_check(curr.cords);
        let bc_prev = boundary_check(prev.cords);

        let curr_in = bc_curr >= 0.0;
        let prev_in = bc_prev >= 0.0;

        if curr_in {
            if !prev_in {
                // Caso 1: Entrando no volume (Fora -> Dentro)
                // Precisamos calcular a interseção e adicionar
                let t = bc_prev / (bc_prev - bc_curr);

                // 1. Interpola a Posição (Geometria)
                let new_pos = prev.cords + (curr.cords - prev.cords) * t;
                
                // 2. Interpola a Normal (Atributo) USANDO O MESMO t
                let new_normal = prev.normal + (curr.normal - prev.normal) * t;
                
                // IMPORTANTE: A interpolação linear pode desnormalizar o vetor.
                // É recomendável normalizá-lo novamente.
                let new_normal = new_normal.normalize();

                output_list.push(Vertex { 
                    cords: new_pos, 
                    normal: new_normal 
                });
            }
            // Caso 2: Totalmente dentro (Dentro -> Dentro)
            // Apenas adiciona o ponto atual
            output_list.push(curr);
        } else if prev_in {
            // Caso 3: Saindo do volume (Dentro -> Fora)
            // Calcula interseção e adiciona (mas não adiciona o ponto atual que está fora)
            let t = bc_prev / (bc_prev - bc_curr);

            let new_pos = prev.cords + (curr.cords - prev.cords) * t;
            let new_normal = prev.normal + (curr.normal - prev.normal) * t;
            let new_normal = new_normal.normalize();
            
            output_list.push(Vertex { 
                cords: new_pos, 
                normal: new_normal 
            });
        }

        // Caso 4: Totalmente fora (Fora -> Fora) -> Não faz nada
    }

    output_list
}

// Função Principal de Recorte
fn sutherland_hodgman(
    poly: &[Vertex],
    near: f32, 
    far: f32
) -> Vec<Vertex> {
    
    let z_front = near / far; // Limite normalizado do plano near

    // 1. Recorte Esquerda (x >= -z => x + z >= 0)
    let poly = clip_against_plane(&poly, |p| p.x + p.z);

    // 2. Recorte Direita (x <= z => z - x >= 0)
    let poly = clip_against_plane(&poly, |p| p.z - p.x);

    // 3. Recorte Fundo (y >= -z => y + z >= 0)
    let poly = clip_against_plane(&poly, |p| p.y + p.z);

    // 4. Recorte Topo (y <= z => z - y >= 0)
    let poly = clip_against_plane(&poly, |p| p.z - p.y);

    // 5. Recorte Perto (z >= z_front => z - z_front >= 0)
    let poly = clip_against_plane(&poly, |p| p.z - z_front);

    // 6. Recorte Longe (z <= 1.0 => 1.0 - z >= 0)
    let poly = clip_against_plane(&poly, |p| 1.0 - p.z);

    poly
}


enum ClipPlane {
    Left,
    Right,
    Bottom,
    Top,
    Near,
    Far,
}

// Verifica se um ponto está DENTRO em relação a um plano específico
// Baseado na desigualdade |X| <= Z e |Y| <= Z do Alvy Ray Smith (Fig. 2)
fn is_inside(p: &Vector4<f32>, plane: ClipPlane, z_front: f32) -> bool {
    // Nota: p.x, p.y, p.z são as coordenadas no espaço normalizado (pN)
    // Assumimos que p.w é 1.0 ou positivo neste estágio.
    
    match plane {
        // Regra: X >= -Z  =>  X + Z >= 0
        ClipPlane::Left => p.x >= -p.z, 
        
        // Regra: X <= Z   =>  Z - X >= 0
        ClipPlane::Right => p.x <= p.z,
        
        // Regra: Y >= -Z  =>  Y + Z >= 0
        ClipPlane::Bottom => p.y >= -p.z,
        
        // Regra: Y <= Z   =>  Z - Y >= 0
        ClipPlane::Top => p.y <= p.z,
        
        // Regra: Z >= Z_front (Near)
        ClipPlane::Near => p.z >= z_front,
        
        // Regra: Z <= 1.0 (Far)
        ClipPlane::Far => p.z <= 1.0,
    }
}

// Função auxiliar para verificar se o ponto está totalmente seguro (dentro de todos)
fn is_fully_inside(p: &Vector4<f32>, near: f32, far: f32) -> bool {
    let z_front = near / far; // Limite inferior do Z normalizado
    
    is_inside(p, ClipPlane::Left, z_front) &&
    is_inside(p, ClipPlane::Right, z_front) &&
    is_inside(p, ClipPlane::Bottom, z_front) &&
    is_inside(p, ClipPlane::Top, z_front) &&
    is_inside(p, ClipPlane::Near, z_front) &&
    is_inside(p, ClipPlane::Far, z_front)
}


fn create_vertex(index: usize, raw_obj: &RawObj, normals: &[Vector3<f32>; 8]) -> Vertex {
    Vertex {
        cords: Vector4::new(
            raw_obj[(0, index)],
            raw_obj[(1, index)],
            raw_obj[(2, index)],
            raw_obj[(3, index)],
        ),
        normal: normals[index]    
    }
}

fn main() {

    // Definição das faces do cubo (6 faces, cada face com 4 vértices)
    // deve ser convencionado algum sentido para os vértices das faces
    // nesse caso usou-se o sentido ANTI-HORÁRIO
    let faces: [[usize; 4]; 6] = [
        [3, 2, 1, 0],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [4, 7, 3, 0],
        [1, 2, 6, 5],
    ];

    // faces e edges indexam as colunas na matriz de pontos (0 - 7)
    // este é o formato homogêneo, então cada coluna é um ponto na forma (x, y, z, h)
    // let points = Points::from_row_slice(&[
    //     -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, 
    //     -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0,
    //     -1.0, -1.0, -1.0, -1.0,  1.0,  1.0,  1.0,  1.0,
    //      1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,
    // ]);

    let raw_obj = RawObj::from_row_slice(&[
        -value!(),  value!(),  value!(), -value!(), -value!(),  value!(),  value!(), -value!(), 
        -value!(), -value!(),  value!(),  value!(), -value!(), -value!(),  value!(),  value!(),
        -value!(), -value!(), -value!(), -value!(),  value!(),  value!(),  value!(),  value!(),
         1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,
    ]);

    // return;

    let centroids = faces.map(|face| {
    face.iter()
        // Transforma cada índice em um Vector3
        .map(|&index| Vector3::new(
            raw_obj[(0, index)], 
            raw_obj[(1, index)], 
            raw_obj[(2, index)]
        ))
        // Soma todos os vetores (o tipo ::<Vector3> ajuda o compilador a inferir)
        .sum::<Vector3<f32>>() / face.len() as f32
    });

    let face_vectors = faces.map(|face| {
        let p1 = Vector3::new(
            raw_obj[(0, face[0])],
            raw_obj[(1, face[0])],
            raw_obj[(2, face[0])],
        );
        let p2 = Vector3::new(
            raw_obj[(0, face[1])],
            raw_obj[(1, face[1])],
            raw_obj[(2, face[1])],
        );
        let p3 = Vector3::new(
            raw_obj[(0, face[2])],
            raw_obj[(1, face[2])],
            raw_obj[(2, face[2])],
        );

        let a = p1 - p2;
        let b = p3 - p2;

        b.cross(&a).normalize()
    });

    println!("Centroids: {:?}", centroids);
    println!("Face Vectors: {:?}", face_vectors);

    // SRC
    let vrp = Vector3::new(15.0, 15.0, 15.0);
    let view_up = Vector3::new(0.0, 1.0, 0.0);
    let p = Vector3::new(0.0, 0.0, 0.0);

    // para cada face, calcular se é visível ou não
    // enumerate para manter o índice da face
    // filtragem com filter_map para retornar apenas os índices das faces visíveis
    let visible_faces: Vec<usize> = face_vectors.iter().enumerate()
        .filter_map(|(i, normal)| {

            // o: vetor centroide->vrp (normalizado)
            let o = (vrp - centroids[i]).normalize();

            // se o produto escalar for positivo, a face é visível: retorna o índice
            // senão, retorna None
            if o.dot(&normal) > 0.0 {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    println!("Visible Faces: {:?}", visible_faces);

    // extrair os vértices visíveis das faces visíveis
    // usamos flat_map pois ao iterar sobre as faces, cada posição retorna um array de vértices
    // no final, ele concatena todos os arrays em um único vetor
    // cloned() para transformar de &usize para usize (obter valor em vez de referência)
    // let mut visible_vertices: Vec<usize> = visible_faces.iter()
    //     .flat_map(|&face_index| faces[face_index].iter().cloned())
    //     .collect();

    // // remover duplicatas
    // visible_vertices.sort();
    // visible_vertices.dedup();

    // println!("Visible Vertices: {:?}", visible_vertices);

    let mut vertex_normals: [Vector3<f32>; 8] = [Vector3::new(0.0, 0.0, 0.0); 8];
 
    for vertex in 0..8 {
        let mut normal_sum = Vector3::new(0.0, 0.0, 0.0);
        for i in 0..6 {
            if faces[i].contains(&vertex) {
                normal_sum += face_vectors[i];
            }
        }

        vertex_normals[vertex] = normal_sum.normalize();
    }

    println!("Vertex Normals: {:?}", vertex_normals);
    // limites viewport 
    let u_min = 100.0;
    let v_min = 300.0;
    let u_max = 1000.0;
    let v_max = 900.0;

    // limites window 
    let x_min = -10.0;
    let y_min = -8.0;
    let x_max = 10.0;
    let y_max = 8.0;

    // Parâmetros de projeção
    let su = 10.0;
    let sv = 8.0;
    let near = 20.0;
    let far = 120.0;
    let dp = 50.0;
    let cu = 0.0;
    let cv = 0.0;

    // Parâmetros de profundidade
    let zmin = 0.0;
    let zmax = 65000.0;

    // Cálculo dos vetores u, v, n
    let n = p - vrp;
    let n = n.normalize();
    let v = view_up - n * (view_up.dot(&n));
    let v = v.normalize();
    let u = n.cross(&v);

    // return;

    // translação para a origem de VRP
    let mat_a = Matrix4::new(
        1.0, 0.0,  0.0, -vrp.x,
        0.0, 1.0,  0.0, -vrp.y,
        0.0, 0.0,  1.0, -vrp.z,
        0.0, 0.0,  0.0, 1.0
    );

    // rotação para alinhar os eixos
    let mat_b = Matrix4::new(
        u.x, u.y, u.z, 0.0, 
        v.x, v.y, v.z, 0.0, 
        n.x, n.y, n.z, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );

    // translação para alinhar o centro do projection plane
    let mat_c = Matrix4::new(
        1.0, 0.0, -cu/dp, 0.0,
        0.0, 1.0, -cv/dp, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    // escalar para o volume de visão canônico
    let mat_d = Matrix4::new(
        dp / (su * far), 0.0, 0.0, 0.0,
        0.0,dp / (sv * far),0.0,0.0,
        0.0, 0.0,1.0/far,  0.0,
        0.0, 0.0,  0.0, 1.0
    );

    let mat_p = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, far/(far - near),  -near/(far - near),
        0.0, 0.0, 1.0, 0.0
    );

    let  mat_j = Matrix4::new(
        1.0, 0.0,  0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0,  1.0, 0.0,
         0.0, 0.0,  0.0, 1.0
    );

    let  mat_k = Matrix4::new(
         0.5, 0.0, 0.0, 0.5,
         0.0, 0.5, 0.0, 0.5,
         0.0, 0.0, 1.0, 0.0,
         0.0, 0.0, 0.0, 1.0
    );

    let mat_l = Matrix4::new(
        u_max - u_min, 0.0,           0.0,  u_min,
        0.0,           v_max - v_min, 0.0,  v_min,
        0.0,           0.0,           1.0,  0.0,
        0.0,           0.0,           0.0,  1.0
    );

    let mat_m = Matrix4::new(
        1.0, 0.0, 0.0,  0.5,
        0.0, 1.0, 0.0,  0.5,
        0.0, 0.0, 1.0,  0.0,
        0.0, 0.0, 0.0,  1.0
    );

    // matriz que normaliza para o volume de visão canônico (alvy ray smith)
    let m_norm = mat_d * mat_c * mat_b * mat_a;

    // a ideia é 
    // 1. aplicar m_norm em raw_obj para obter os pontos no espaço normalizado
    // 2. realizar o recorte com sutherland-hodgman
    // 3. aplicar mat_p para projeção
    // 4. dividir pelo fator homogêneo
    // 5. aplicar a transformação de viewport (mat_j, mat_k, mat_l, mat_m)

    // matriz que leva para cordenadas de tela
    let m_src = mat_m * mat_l * mat_k * mat_j;

    let p1 = m_norm * raw_obj;

    // dividir pelo fator homogeneo
    // for j in 0..mat_p.ncols() {
    //     let h = mat_p[(3, j)];
    //     for i in 0..mat_p.nrows() {
    //         mat_p[(i, j)] /= h;
    //     }
    // }

    // criar um objeto de vetores nalgebra a partir da matriz bruta
    // isto é interessante para realizar cálculos de álgebra linear
    // como no sutherland-hodgman ou cálculo de normais
    let mut obj = Vec::<[Vertex; 4]>::with_capacity(6);

    for face in faces.iter() {
        let v0 = create_vertex(face[0], &p1, &vertex_normals);
        let v1 = create_vertex(face[1], &p1, &vertex_normals);
        let v2 = create_vertex(face[2], &p1, &vertex_normals);
        let v3 = create_vertex(face[3], &p1, &vertex_normals);

        obj.push([v0, v1, v2, v3]);
    }

    for face in visible_faces.iter() {
        let original_poly = &obj[*face];

        let clipped_poly = sutherland_hodgman(original_poly, near, far);

        println!("Face {}: Original Vertices: {}, Clipped Vertices: {}", *face, original_poly.len(), clipped_poly.len());
        clipped_poly.iter().for_each();
    }

}
