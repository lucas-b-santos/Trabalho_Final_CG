extern crate nalgebra as na;

use std::{f32::INFINITY, vec};

use na::{Matrix4, SMatrix, Vector3};

type Points = SMatrix::<f32, 4, 8>; 

fn main() {
    let edges: [[usize; 2]; 12] = [
        [0, 1], [1, 2], [2, 3], [3, 0],
        [4, 5], [5, 6], [6, 7], [7, 4],
        [0, 4], [1, 5], [2, 6], [3, 7]
    ];

    // vértices devem seguir sentido ANTI-HORÁRIO
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
    let points = Points::from_row_slice(&[
        -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, 
        -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0,
        -1.0, -1.0, -1.0, -1.0,  1.0,  1.0,  1.0,  1.0,
         1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,
    ]);

    let centroids = faces.map(|face| {
    face.iter()
        // Transforma cada índice em um Vector3
        .map(|&index| Vector3::new(
            points[(0, index)], 
            points[(1, index)], 
            points[(2, index)]
        ))
        // Soma todos os vetores (o tipo ::<Vector3> ajuda o compilador a inferir)
        .sum::<Vector3<f32>>() / face.len() as f32
    });

    let face_vectors = faces.map(|face| {
        let p1 = Vector3::new(
            points[(0, face[0])],
            points[(1, face[0])],
            points[(2, face[0])],
        );
        let p2 = Vector3::new(
            points[(0, face[1])],
            points[(1, face[1])],
            points[(2, face[1])],
        );
        let p3 = Vector3::new(
            points[(0, face[2])],
            points[(1, face[2])],
            points[(2, face[2])],
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
    // cloned() para transformar de &usize para usize (valor em vez de referência)
    let mut visible_vertices: Vec<usize> = visible_faces.iter()
        .flat_map(|&face_index| faces[face_index].iter().cloned())
        .collect();

    // remover duplicatas
    visible_vertices.sort();
    visible_vertices.dedup();

    println!("Visible Vertices: {:?}", visible_vertices);

    // vetores normais unitários médios de cada vértice visível
    let mut vertex_normals = vec![Vector3::new(0.0, 0.0, 0.0); 8];
 
    for vertex in visible_vertices {
        let mut normal_sum = Vector3::new(0.0, 0.0, 0.0);
        for i in 0..6 {
            if faces[i].contains(&vertex) {
                normal_sum += face_vectors[i];
            }
        }

        vertex_normals[vertex] = normal_sum.normalize();
    }

    println!("Vertex Normals: {:?}", vertex_normals);
    
    return;

    // Viewport 
    let u_min = 100.0;
    let v_min = 300.0;
    let u_max = 1000.0;
    let v_max = 900.0;

    // Window 
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
    let zmax = INFINITY;

    // Cálculo dos vetores u, v, n
    let n = p - vrp;
    let n = n.normalize();
    let v = view_up - n * (view_up.dot(&n));
    let v = v.normalize();
    let u = n.cross(&v);

    let mat_a = Matrix4::new(
        1.0, 0.0,  0.0, -vrp.x,
        0.0, 1.0,  0.0, -vrp.y,
        0.0, 0.0,  1.0, -vrp.z,
        0.0, 0.0,  0.0, 1.0
    );

    let mat_b = Matrix4::new(
        u.x, u.y, u.z, 0.0, 
        v.x, v.y, v.z, 0.0, 
        n.x, n.y, n.z, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );

    let mat_c = Matrix4::new(
        1.0, 0.0, -cu/dp, 0.0,
        0.0, 1.0, -cv/dp, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

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

    let m_final = mat_p * mat_d * mat_c * mat_b * mat_a;

    let mut mat_p = m_final * points;

    // dividir pelo fator homogeneo
    // for j in 0..mat_p.ncols() {
    //     let h = mat_p[(3, j)];
    //     for i in 0..mat_p.nrows() {
    //         mat_p[(i, j)] /= h;
    //     }
    // }

    println!("Matrix:\n{}", mat_p);
}
