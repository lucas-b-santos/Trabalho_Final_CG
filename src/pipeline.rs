extern crate nalgebra as na;
use eframe::egui;
use na::{Matrix4, Vector3, Vector4};

pub const WIDTH : usize = 1251;
pub const HEIGHT : usize = 851;

pub mod types;
use types::{Vertex, Face, ConstantEntry, PhongEntry, RawObj, SceneParams, Material, UCube};

/// Função genérica que recorta um polígono contra UM plano definido por 'boundary_check'
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

                // t é a fração ao longo da aresta onde ocorre a interseção
                let t = bc_prev / (bc_prev - bc_curr);

                // Interpola a Posição
                let new_pos = prev.cords + (curr.cords - prev.cords) * t;
                
                // Interpola a Normal
                let new_normal = prev.normal + (curr.normal - prev.normal) * t;
               
                output_list.push(Vertex { 
                    cords: new_pos, 
                    normal: new_normal.normalize()
                });
            }

            // Caso 2: Totalmente dentro (Dentro -> Dentro)
            // Apenas adiciona o ponto atual
            output_list.push(curr);

        } else if prev_in {
            // Caso 3: Saindo do volume (Dentro -> Fora)
            // Calcula interseção e adiciona (mas não adiciona o ponto atual que está fora)
            // Procedimento igual ao Caso 1

            let t = bc_prev / (bc_prev - bc_curr);
            let new_pos = prev.cords + (curr.cords - prev.cords) * t;
            let new_normal = prev.normal + (curr.normal - prev.normal) * t;
            
            output_list.push(Vertex { 
                cords: new_pos, 
                normal: new_normal.normalize()
            });
        }

        // Caso 4: Totalmente fora (Fora -> Fora) -> Não faz nada
    }

    output_list
}

/// Aplica Sutherland-Hodgman e retorna o polígono recortado
// O conceito de boundary_check é usado para definir os 6 planos de recorte
// Isto se baseia na matemática do volume de visão canônico (Alvy Ray Smith)
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

/// Cria um vértice a partir da matriz bruta e normais dos vértices
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

/// Produto componente a componente entre dois vetores 3D
fn one_by_one_prod(a: [f32; 3], b: [f32; 3]) -> Vector3<f32> {
    Vector3::new(
        a[0] * b[0],
        a[1] * b[1],
        a[2] * b[2],
    )
}

/// Calcula a cor de um ponto baseado no modelo de iluminação informado
// aqui, aproveitamos o fato de que o cálculo difuso e ambiente são iguais para ambos os modelos (Phong e Constante)
// só usamos o parâmetro 'phong' para decidir o cálculo do especular
fn calc_color(scene: &SceneParams, normal: Vector3<f32>, centroid: Vector3<f32>, material: &Material, phong: bool) -> Vector3<f32> {
    let mut total_intensity = one_by_one_prod(scene.i_amb, material.ka);

    let vet_l = (scene.lamp_pos - centroid).normalize();
    let n_l = normal.dot(&vet_l);

    if n_l < 0.0 { return total_intensity; } // não há efeito difuso

    let total_dif = one_by_one_prod(scene.i_lamp, material.kd) * n_l;

    total_intensity += total_dif;

    let mut total_esp = one_by_one_prod(scene.i_lamp, material.ks);
    let vet_s = (scene.vrp - centroid).normalize();

    if phong {
        let vet_h = (vet_l + vet_s).normalize();
        let n_h = normal.dot(&vet_h);
        if n_h < 0.0 { return total_intensity; } // não há efeito especular
        total_esp = total_esp * n_h.powf(material.n);
        total_intensity += total_esp;
        return total_intensity;
    }

    let vet_r = ((2.0 * vet_l).dot(&normal)) * normal - vet_l;

    let r_s = vet_r.dot(&vet_s);

    if r_s < 0.0 { return total_intensity; } // não há efeito especular

    total_esp = total_esp * r_s.powf(material.n);

    total_intensity += total_esp;   
    total_intensity
}


/// Retorna informações do polígono: y_min e número de scanlines
fn get_poly_info(poly: &[Vertex]) -> (usize, usize) {
    // obtém lista de coordenadas Y do polígono
    let y_cords = poly.iter().map(|v| v.cords.y.round() as usize).collect::<Vec<usize>>(); 
    
    // obtém o menor e maior valor de Y do polígono
    let y_min_poly = y_cords.iter().min().cloned().unwrap_or(0);
    let y_max_poly = y_cords.iter().max().cloned().unwrap_or(0);

    // calcula o número de scanlines
    let ns = y_max_poly - y_min_poly;   

    (y_min_poly, ns)
}

/// Preenche o polígono usando o modelo de iluminação constante
fn fill_constant(face: &Face, scene: &SceneParams, selected: bool, material: &Material, buffer: &mut[egui::Color32], z_buffer: &mut [u16]) {
    let polygon = &face.vertices;

    let face_color;

    if selected {
        face_color = Vector3::new(255.0, 0.0, 0.0); // vermelho para seleção
    } 
    else {
        face_color = calc_color(scene, face.normal, face.centroid, material,false) * 255.0; // converter para escala 0-255
    }
                        
    let (y_min_poly, ns) = get_poly_info(polygon);

    // cria uma lista de scanlines, cada scanline é uma lista de pontos
    // cada ponto é um objeto com x, z e vetor normal interpolados
    let mut scanlines = vec![Vec::<ConstantEntry>::with_capacity(5); ns];

    // Para cada aresta do polígono, calculamos os pontos de cada scanline
    for i in 0..polygon.len() {

        // Aresta é formada por dois pontos consecutivos do polígono (a e b)
        let a = polygon[i];
        // usamos índice circular p/ conectar o último ponto com o primeiro
        let b = polygon[(i + 1) % polygon.len()];

        // Aresta AB é uma lista de pontos [a, b]
        let mut edge = [a, b];

        // Verifica se a aresta é horizontal, se for, pula para a próxima iteração
        if edge[0].cords.y == edge[1].cords.y { continue; }

        // necessário ordenar pela coordenada Y (de menor para maior)
        if a.cords.y > b.cords.y {
            edge.swap(0, 1);
        }
       
        let ymax = edge[1].cords.y; 
        let ymin = edge[0].cords.y;
        let xmax = edge[1].cords.x;
        let xmin = edge[0].cords.x;
        let zmin = edge[0].cords.z;
        let zmax = edge[1].cords.z;

        let variacao_y = ymax - ymin;

        let tx = (xmax - xmin) / variacao_y;
        let tz = (zmax - zmin) / variacao_y;

        let mut current = ConstantEntry {
            x: xmin,
            z: zmin,
        };

        for ii in (ymin.round() as usize - y_min_poly)..(ymax.round() as usize - y_min_poly) {
            scanlines[ii].push(current);

            // incrementa as taxas
            current.x += tx;
            current.z += tz;
        }
    }
    
    // ordena os pontos de cada scanline por coordenada x
    for i in 0..ns {
        scanlines[i].sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    }

    // Y inicial
    let mut current_y = y_min_poly;

    // Para cada scanline
    for i in 0..ns {

        // Para cada par de pontos na scanline
        for ii in (0..scanlines[i].len()).step_by(2) {

            // obtemos a e b, um intervalo que deve ser preenchido
            let a = scanlines[i][ii];
            let b = scanlines[i][ii + 1];
            
            let variacao_x = b.x - a.x;

            if variacao_x == 0.0 {
                continue; // evita divisão por zero
            }
            
            let x_start = a.x.floor() as i16; 
            let x_end = b.x.floor() as i16;   
            
            let tz = (b.z - a.z) / variacao_x;

            // Calculamos quanto "andamos" do ponto real (a.x) até o primeiro pixel (x_start)
            let dx_prestep = (x_start as f32) - a.x;

            // Ajustamos os valores iniciais com base nesse "pulo"
            let mut current_z = a.z + (tz * dx_prestep);

            for iii in x_start..x_end {
                let idx = current_y * WIDTH + iii as usize;
                let z_value = current_z as u16;

                if z_buffer[idx] > z_value {
                    let color = face_color;

                    buffer[idx] = egui::Color32::from_rgb(
                        color.x as u8,
                        color.y as u8,
                        color.z as u8,
                    );

                    z_buffer[idx] = z_value;
                }

                // incrementa com as variações calculadas
                current_z += tz;
            }

        }

        current_y+=1;
    }

}

/// Preenche o polígono usando o modelo de iluminação Phong
fn fill_phong(face: &Face, scene: &SceneParams, selected: bool, material: &Material, buffer: &mut[egui::Color32], z_buffer: &mut [u16]) {

    let polygon = &face.vertices;

    let (y_min_poly, ns) = get_poly_info(polygon);

    // cria uma lista de scanlines, cada scanline é uma lista de pontos
    // cada ponto é um objeto com x, z e vetor normal interpolados
    let mut scanlines = vec![Vec::<PhongEntry>::with_capacity(5); ns];

    // Para cada aresta do polígono, calculamos os pontos de cada scanline
    for i in 0..polygon.len() {

        // Aresta é formada por dois pontos consecutivos do polígono (a e b)
        let a = polygon[i];
        // usamos índice circular p/ conectar o último ponto com o primeiro
        let b = polygon[(i + 1) % polygon.len()];

        // Aresta AB é uma lista de pontos [a, b]
        let mut edge = [a, b];

        // Verifica se a aresta é horizontal, se for, pula para a próxima iteração
        if edge[0].cords.y == edge[1].cords.y { continue; }

        // necessário ordenar pela coordenada Y (de menor para maior)
        if a.cords.y > b.cords.y {
            edge.swap(0, 1);
        }
       
        let ymax = edge[1].cords.y; 
        let ymin = edge[0].cords.y;
        let xmax = edge[1].cords.x;
        let xmin = edge[0].cords.x;
        let zmin = edge[0].cords.z;
        let zmax = edge[1].cords.z;
        let normal_min = edge[0].normal;
        let normal_max = edge[1].normal;

        let variacao_y = ymax - ymin;

        let tx = (xmax - xmin) / variacao_y;
        let tz = (zmax - zmin) / variacao_y;
        let tnormal = (normal_max - normal_min) / variacao_y;

        let mut current = PhongEntry {
            x: xmin,
            z: zmin,
            normal: normal_min,
        };

        for ii in (ymin.round() as usize - y_min_poly)..(ymax.round() as usize - y_min_poly) {
            scanlines[ii].push(current);

            // incrementa as taxas
            current.x += tx;
            current.z += tz;
            current.normal += tnormal;
        }
    }
    
    // ordena os pontos de cada scanline por coordenada x
    for i in 0..ns {
        scanlines[i].sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    }

    // Y inicial
    let mut current_y = y_min_poly;

    // Para cada scanline
    for i in 0..ns {

        // Para cada par de pontos na scanline
        for ii in (0..scanlines[i].len()).step_by(2) {

            // obtemos a e b, um intervalo que deve ser preenchido
            let a = scanlines[i][ii];
            let b = scanlines[i][ii + 1];
            
            let variacao_x = b.x - a.x;

            if variacao_x == 0.0 {
                continue; // evita divisão por zero
            }
            
            let x_start = a.x.floor() as i16; 
            let x_end = b.x.floor() as i16;   
            
            let tz = (b.z - a.z) / variacao_x;
            let tnormal = (b.normal - a.normal) / variacao_x;

            // Calculamos quanto "andamos" do ponto real (a.x) até o primeiro pixel (x_start)
            let dx_prestep = (x_start as f32) - a.x;

            // Ajustamos os valores iniciais com base nesse "pulo"
            let mut current_z = a.z + (tz * dx_prestep);
            let mut current_normal = a.normal + (tnormal * dx_prestep);

            for iii in x_start..x_end {
                let idx = current_y * WIDTH + iii as usize;
                let z_value = current_z as u16;

                if z_buffer[idx] > z_value {
                    let color = if selected {
                        Vector3::new(255.0, 0.0, 0.0) // vermelho para seleção
                    } 
                    else {
                        calc_color(
                            scene, 
                            current_normal.normalize(), // necessário normalizar o vetor interpolado
                            face.centroid, 
                            material,
                            true // usa modelo Phong
                        ) * 255.0 // converter para escala 0-255
                    };
                    
                    buffer[idx] = egui::Color32::from_rgb(
                        color.x as u8,
                        color.y as u8,
                        color.z as u8,
                    );

                    z_buffer[idx] = z_value;
                }

                // incrementa com as variações calculadas
                current_z += tz;
                current_normal += tnormal;
            }


        }

        current_y+=1;
    }

}



/// Verifica se a face está visível 
fn is_face_visible(normal: Vector3<f32>, centroid: Vector3<f32>, vrp: Vector3<f32>) -> bool {
    // o: vetor centroide->vrp (normalizado)
    let o = (vrp - centroid).normalize();
    o.dot(&normal) > 0.0    
}

/// Recebe um cubo no SRU, aplica o pipeline nele, renderiza as faces e as retorna (em SRT)
pub fn render_cube(buffer: &mut [egui::Color32], z_buffer: &mut [u16], scene: &SceneParams, obj: &UCube, selected: bool) -> Vec<Face> {

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

    // cálculo dos centróides das faces
    let centroids = faces.map(|face| {
    face.iter()
        // Transforma cada índice em um Vector3
        .map(|&index| Vector3::new(
            obj.raw[(0, index)], 
            obj.raw[(1, index)], 
            obj.raw[(2, index)]
        ))
        // Soma todos os vetores (o tipo ::<Vector3> ajuda o compilador a inferir)
        .sum::<Vector3<f32>>() / face.len() as f32
    });

    // cálculo dos vetores normais das faces
    let face_vectors = faces.map(|face| {
        let p1 = Vector3::new(
            obj.raw[(0, face[0])],
            obj.raw[(1, face[0])],
            obj.raw[(2, face[0])],
        );
        let p2 = Vector3::new(
            obj.raw[(0, face[1])],
            obj.raw[(1, face[1])],
            obj.raw[(2, face[1])],
        );
        let p3 = Vector3::new(
            obj.raw[(0, face[2])],
            obj.raw[(1, face[2])],
            obj.raw[(2, face[2])],
        );

        let a = p1 - p2;
        let b = p3 - p2;

        b.cross(&a).normalize()
    });

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

    // z varia de 0 a 65535 
    let z_max = u16::MAX as f32;

    // Cálculo dos vetores u, v, n
    let n = scene.p - scene.vrp;
    let n = n.normalize();
    let v = scene.view_up - n * (scene.view_up.dot(&n));
    let v = v.normalize();
    let u = n.cross(&v);

    // translação para a origem de VRP
    let mat_a = Matrix4::new(
        1.0, 0.0,  0.0, -scene.vrp.x,
        0.0, 1.0,  0.0, -scene.vrp.y,
        0.0, 0.0,  1.0, -scene.vrp.z,
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
        1.0, 0.0, -scene.cu/scene.dp, 0.0,
        0.0, 1.0, -scene.cv/scene.dp, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    // escalar para o volume de visão canônico
    let mat_d = Matrix4::new(
        scene.dp / (scene.su * scene.far), 0.0, 0.0, 0.0,
        0.0,scene.dp / (scene.sv * scene.far),0.0,0.0,
        0.0, 0.0,1.0/scene.far,  0.0,
        0.0, 0.0,  0.0, 1.0
    );

    // matriz de projeção perspectiva
    let mat_p = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, scene.far/(scene.far - scene.near),  -scene.near/(scene.far - scene.near),
        0.0, 0.0, 1.0, 0.0
    );

    // matriz que leva para cordenadas de tela
    // optou-se por escalar z no intervalo [0, z_max]
    let mat_s = Matrix4::new(
        (scene.u_max - scene.u_min) * 0.5, 0.0, 0.0,  (scene.u_min + scene.u_max + 1.0) * 0.5,
        0.0, (scene.v_min - scene.v_max) * 0.5, 0.0,  (scene.v_min + scene.v_max + 1.0) * 0.5,
        0.0, 0.0, z_max,  0.5,
        0.0, 0.0, 0.0,  1.0
    );

    // matriz que normaliza para o volume de visão canônico (alvy ray smith)
    let mat_norm = mat_d * mat_c * mat_b * mat_a;

    // normalizamos os pontos do objeto para realizar recorte
    let p1 = mat_norm * obj.raw;

    // criar um objeto de vetores nalgebra a partir da matriz bruta
    // isto é interessante para realizar cálculos de álgebra linear
    let mut final_obj = Vec::<[Vertex; 4]>::with_capacity(6);
    for face in faces {
        let v0 = create_vertex(face[0], &p1, &vertex_normals);
        let v1 = create_vertex(face[1], &p1, &vertex_normals);
        let v2 = create_vertex(face[2], &p1, &vertex_normals);
        let v3 = create_vertex(face[3], &p1, &vertex_normals);

        final_obj.push([v0, v1, v2, v3]);
    }

    let mut screen_obj = Vec::<Face>::with_capacity(6);

    for face in 0..6 {
        let normal = face_vectors[face];
        let centroid = centroids[face];

        // se a face não estiver visível, pula para a próxima
        if !is_face_visible(normal, centroid, scene.vrp) {
            continue;
        }

        // uma face originalmente é um polígono com 4 vértices
        let original_poly = &final_obj[face];

        let mut clipped_poly = sutherland_hodgman(original_poly, scene.near, scene.far);

        // para cada vértice do polígono recortado, aplicar as transformações finais
        for vertex in clipped_poly.iter_mut() {
            vertex.cords = mat_p * vertex.cords; // projeção
            vertex.cords /= vertex.cords.w; // Divisão pelo fator homogêneo (ele usa w no lugar do h)
            vertex.cords = mat_s * vertex.cords; // transformação p/ coordenadas de tela
        }

        let face = Face {
            vertices: clipped_poly,
            normal: normal,
            centroid: centroid,
        };

        if scene.use_phong {
            fill_phong(&face, scene, selected, &obj.params, buffer, z_buffer);
        }
        else {
            fill_constant(&face, scene, selected, &obj.params, buffer, z_buffer);
        }

        screen_obj.push(face);
    }

    screen_obj
}
