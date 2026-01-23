#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate nalgebra as na;
use na::{Matrix4};

use eframe::egui;
use egui::{ColorImage, TextureHandle, Vec2};

pub mod pipeline;
use pipeline::{HEIGHT, WIDTH, render};
use pipeline::types::{SceneParams, UCube, Face, ObjectTransform};

pub mod utils;
use utils::point_to_edge_distance;

struct MyApp {
    pixels: Vec<egui::Color32>,
    z_buffer: Vec<u16>,
    texture: Option<TextureHandle>,
    width: usize,
    height: usize,
    cubes: Vec<UCube>,
    s_cubes: Vec<Vec<Face>>,
    obj_transform: ObjectTransform, // Transformações do objeto
    selected_cube_index: Option<usize>, // Índice do cubo selecionado
    hovered_cube_index: Option<usize>,  // Índice do cubo sob o mouse
    scene: SceneParams,
    mouse_in_buffer: Option<[f32; 2]>, // Posição do mouse na área de renderização
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut cubes = Vec::with_capacity(10);
        let s_cubes = Vec::<Vec<Face>>::with_capacity(10);
        cubes.push(UCube::default());

        Self {
            width: WIDTH,
            height: HEIGHT,
            mouse_in_buffer: None,
            obj_transform: ObjectTransform::default(),
            z_buffer: vec![u16::MAX; WIDTH * HEIGHT],
            pixels: vec![egui::Color32::GRAY; WIDTH * HEIGHT],
            texture: None,
            cubes: cubes,
            s_cubes: s_cubes,
            selected_cube_index: None,
            hovered_cube_index: None,
            scene: SceneParams::default(),
        }
    }

    // Função auxiliar que varre a cena para achar quem está sob o mouse
    // A lógica consiste em verificar qual aresta (entre todos os cubos) está mais próxima do mouse
    fn check_hover(&self, mouse_pos: [f32; 2]) -> Option<usize> {
        let mut found_idx = None;
        let mut min_distance = f32::INFINITY;

        for (idx, cube) in self.s_cubes.iter().enumerate() {

            for face in cube.iter() {
                let px = mouse_pos[0];
                let py = mouse_pos[1];

                // Se o ponto não estiver dentro da face, pula para a próxima
                if !face.is_point_in(px, py) { continue; }

                for i in 0..face.vertices.len() {
                    let x1 = face.vertices[i].cords.x;
                    let y1 = face.vertices[i].cords.y;
                    let x2 = face.vertices[(i + 1) % face.vertices.len()].cords.x;
                    let y2 = face.vertices[(i + 1) % face.vertices.len()].cords.y;

                    // Calcula a distância do ponto até a aresta atual
                    let distance = point_to_edge_distance(
                        [px, py],
                        [x1, y1],
                        [x2, y2],
                    );

                    if distance < min_distance {
                        min_distance = distance;
                        found_idx = Some(idx);
                    }
                }
                }

        }
        found_idx
    }

    fn render_scene(&mut self) {
        // 1. Limpar buffer
        self.pixels.fill(egui::Color32::GRAY);
        self.z_buffer.fill(u16::MAX);
        self.s_cubes.clear();

        for (idx, cube) in self.cubes.iter().enumerate() {
            let mut selected = false;

            if let Some(sel_idx) = self.selected_cube_index {
                if sel_idx == idx {
                    selected = true; // destaca o cubo selecionado
                }
            } 

            if let Some(hover_idx) = self.hovered_cube_index {
                if hover_idx == idx {
                    selected = true; // destaca o cubo sob o mouse
                }
            }
    
            let screen_cube = render(&mut self.pixels, &mut self.z_buffer, &self.scene, cube, selected, true);
            self.s_cubes.push(screen_cube);
        }

    }

    fn translate(&mut self, idx: usize) {
        let dx = self.obj_transform.position.x;
        let dy = self.obj_transform.position.y;
        let dz = self.obj_transform.position.z;

        let translation_matrix = Matrix4::new(
            1.0, 0.0, 0.0, dx,
            0.0, 1.0, 0.0, dy,
            0.0, 0.0, 1.0, dz,
            0.0, 0.0, 0.0, 1.0,
        );

        self.cubes[idx].raw = translation_matrix * &self.cubes[idx].raw;
    }

    fn escale(&mut self, idx: usize) {
        let s = self.obj_transform.scale;
        let scale_matrix = Matrix4::new(
            s, 0.0, 0.0, 0.0,
            0.0, s, 0.0, 0.0,
            0.0, 0.0, s, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        self.cubes[idx].raw = scale_matrix * &self.cubes[idx].raw;
    }

    fn rotate_x(&mut self, idx: usize) {
        let angle_rad = self.obj_transform.rotation.x.to_radians();
        let rotation_matrix = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, angle_rad.cos(), -angle_rad.sin(), 0.0,
            0.0, angle_rad.sin(), angle_rad.cos(), 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        self.cubes[idx].raw = rotation_matrix * &self.cubes[idx].raw;
    }

    fn rotate_y(&mut self, idx: usize) {
        let angle_rad = self.obj_transform.rotation.y.to_radians();
        let rotation_matrix = Matrix4::new(
            angle_rad.cos(), 0.0, angle_rad.sin(), 0.0,
            0.0, 1.0, 0.0, 0.0,
            -angle_rad.sin(), 0.0, angle_rad.cos(), 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        self.cubes[idx].raw = rotation_matrix * &self.cubes[idx].raw;
    }

    fn rotate_z(&mut self, idx: usize) {
        let angle_rad = self.obj_transform.rotation.z.to_radians();
        let rotation_matrix = Matrix4::new(
            angle_rad.cos(), -angle_rad.sin(), 0.0, 0.0,
            angle_rad.sin(), angle_rad.cos(), 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        self.cubes[idx].raw = rotation_matrix * &self.cubes[idx].raw;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       
        // Renderiza a cena no buffer
        self.render_scene();

        let image = ColorImage {
            size: [self.width, self.height],
            source_size: Vec2::new(self.width as f32, self.height as f32),
            pixels: self.pixels.clone(),
        };

        // --- 2. LÓGICA DE HOVER / CLICK ---
        if let Some(mouse_pos) = self.mouse_in_buffer 
        && !self.selected_cube_index.is_some() 
        {
            // Calcula qual objeto está sob o mouse
            self.hovered_cube_index = self.check_hover(mouse_pos);

            if let Some(hov_index) = self.hovered_cube_index {
                if ctx.input(|i| i.pointer.primary_clicked()) {
                    self.selected_cube_index = Some(hov_index);
                }
            }
        } else {
            self.hovered_cube_index = None;
        }

        egui::SidePanel::left("props").resizable(false).show(ctx, |ui| {

            // Botão para adicionar novo cubo
            if ui.button("Adicionar Cubo").clicked() {
                self.cubes.push(UCube::default());
            }

            ui.separator();

            if let Some(idx) = self.selected_cube_index {
                // Aqui mostramos os controles APENAS do cubo selecionado
                ui.separator();
                
                ui.label("Translação (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.obj_transform.position.x));
                    ui.add(egui::DragValue::new(&mut self.obj_transform.position.y));
                    ui.add(egui::DragValue::new(&mut self.obj_transform.position.z));
                });

                ui.label("Rotação (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.obj_transform.rotation.x).suffix("°")); 
                    ui.add(egui::DragValue::new(&mut self.obj_transform.rotation.y).suffix("°"));
                    ui.add(egui::DragValue::new(&mut self.obj_transform.rotation.z).suffix("°"));
                });

                ui.label("Escala:");
                ui.add(egui::Slider::new(&mut self.obj_transform.scale, 0.1..=5.0));
                
                ui.horizontal(|ui| {
                    if ui.button("Transformar").clicked() {
                        self.translate(idx);
                        self.escale(idx);
                        self.rotate_x(idx);
                        self.rotate_y(idx);
                        self.rotate_z(idx);
                    }
                    if ui.button("Cancelar").clicked() {
                        self.obj_transform = ObjectTransform::default();
                        self.selected_cube_index = None;
                        self.hovered_cube_index = None;
                    }
                
            });
            }
            else {
                ui.label("Nenhum objeto selecionado.");
                ui.label("Clique em um cubo na tela para editar.");
            }

            ui.heading("Controles da Cena");
            
            ui.separator();

            ui.collapsing("Câmera", |ui| {
                ui.label("VRP (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.scene.vrp.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.scene.vrp.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.scene.vrp.z).speed(0.1));
                });
            });
            ui.separator();

            // 3. Iluminação
            ui.collapsing("Luz", |ui| {
                ui.label("Intensidade da Luz:");
                ui.color_edit_button_rgb(&mut self.scene.i_lamp);
                ui.separator();
                ui.label("Cor Ambiente:");
                ui.color_edit_button_rgb(&mut self.scene.i_amb);
            });
            
            ui.add_space(20.0);

            if ui.button("Resetar Cena").clicked() {
                self.cubes.clear();
                self.cubes.push(UCube::default());
                self.obj_transform = ObjectTransform::default();
                self.scene = SceneParams::default();
                self.selected_cube_index = None;
                self.hovered_cube_index = None;
            }
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Renderiza a textura...
            let texture = self.texture.get_or_insert_with(|| {
                ui.ctx()
                    .load_texture("framebuffer", image.clone(), egui::TextureOptions::NEAREST)
            });
            texture.set(image, egui::TextureOptions::NEAREST);

            let img_response = ui.image((texture.id(), texture.size_vec2()));

            // Verifica se o mouse está em cima da imagem renderizada
            if let Some(pos) = img_response.hover_pos() {
                // 'pos' é relativo à janela inteira.
                // Precisamos converter para coordenadas relativas à imagem (0,0 no canto da imagem)
                let rel_x = pos.x - img_response.rect.min.x;
                let rel_y = pos.y - img_response.rect.min.y;

                // Valida limites
                if rel_x >= 0.0
                    && rel_y >= 0.0
                    && rel_x < self.width as f32
                    && rel_y < self.height as f32
                {
                    self.mouse_in_buffer = Some([rel_x, rel_y]);
                }
            }
            
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 650.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Modelador de Cubos 3D",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(cc)))
        }),
    )
}
