  pub fn point_to_edge_distance(p: [f32; 2], v1: [f32; 2], v2: [f32; 2]) -> f32 {
    let x1 = v1[0];
    let y1 = v1[1];
    let x2 = v2[0];
    let y2 = v2[1];
    let px = p[0];
    let py = p[1];

    // Produto escalar entre o vetor (P, v1) e o vetor (v1, v2)
    // Se < 0, o ângulo formado é > 90 graus
    // Então, o ponto projetado está antes de x1, y1
    // Logo, a distância mínima é a distância entre P e v1
    let dot = (px - x1) * (x2 - x1) + (py - y1) * (y2 - y1);
    if dot < 0.0 { return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt() }

    // Produto escalar entre o vetor (P, v2) e o vetor (v2, v1)
    // Se < 0, o ângulo formado é > 90 graus
    // Então, o ponto projetado está depois de x2, y2
    // Logo, a distância mínima é a distância entre P e v2
    let dot = (px - x2) * (x1 - x2) + (py - y2) * (y1 - y2);
    if dot < 0.0 { return ((px - x2).powi(2) + (py - y2).powi(2)).sqrt() }

    // Se o ponto projetado está entre x1 e x2, a distância mínima é a perpendicular
    let numerator = ((y2 - y1) * px - (x2 - x1) * py + x2 * y1 - y2 * x1).abs();
    let denominator = ((y2 - y1).powi(2) + (x2 - x1).powi(2)).sqrt();
    let distance_to_line = numerator / denominator;

    distance_to_line
}
 
 