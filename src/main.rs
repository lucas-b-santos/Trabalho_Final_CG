#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate nalgebra as na;
use core::f32;

use na::{Matrix4};

use eframe::egui;
use egui::{ColorImage, TextureHandle, Vec2};

pub mod pipeline;
use pipeline::{HEIGHT, WIDTH, render_cube};
use pipeline::types::{SceneParams, UCube, Face, ObjectTransform};

struct MyApp {
    pixels: Vec<egui::Color32>, // buffer de imagem
    z_buffer: Vec<u16>, // z-buffer
    texture: Option<TextureHandle>, // textura que o egui usa para mostrar o buffer
    cubes: Vec<UCube>, // cubos na cena (em SRU)
    s_cubes: Vec<Vec<Face>>, // cubos pós-processamento (em SRT)
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

    /// Função auxiliar que varre a cena para achar quem está sob o mouse
    // A lógica consiste em verificar qual face possui a média de Z mais próxima do observador
    fn check_hover(&self, mouse_pos: [f32; 2]) -> Option<usize> {
        let mut found_idx = None;
        let mut min_avg = f32::INFINITY;

        for (idx, cube) in self.s_cubes.iter().enumerate() {

            for face in cube.iter() {
                let px = mouse_pos[0];
                let py = mouse_pos[1];

                // Se o ponto não estiver dentro da face, pula para a próxima
                if !face.is_point_in(px, py) { continue; }

                let z_avg = face.z_avg();

                if min_avg > z_avg {
                    min_avg = z_avg;
                    found_idx = Some(idx);
                }

            }

        }
        found_idx
    }

    fn render_scene(&mut self) {
        // Limpar buffer
        self.pixels.fill(egui::Color32::GRAY);
        self.z_buffer.fill(u16::MAX);
        self.s_cubes.clear();

        for (idx, cube) in self.cubes.iter().enumerate() {
            let mut selected = false;

            if let Some(hover_idx) = self.hovered_cube_index {
                if hover_idx == idx {
                    selected = true; // destaca o cubo sob o mouse
                }
            }
    
            let screen_cube = render_cube(&mut self.pixels, &mut self.z_buffer, &self.scene, cube, selected);
            self.s_cubes.push(screen_cube);
        }

    }

    fn translate(&mut self, idx: usize) {
        self.cubes[idx].translate(self.obj_transform.position);
    }

    fn escale(&mut self, idx: usize) {
        let s = self.obj_transform.scale;
        let scale_matrix = Matrix4::new(
            s, 0.0, 0.0, 0.0,
            0.0, s, 0.0, 0.0,
            0.0, 0.0, s, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        let centroid = self.cubes[idx].centroid();

        self.cubes[idx].translate(-centroid);
        self.cubes[idx].raw = scale_matrix * &self.cubes[idx].raw;
        self.cubes[idx].translate(centroid);
    }

    fn rotate_x(&mut self, idx: usize) {
        let angle_rad = self.obj_transform.rotation.x.to_radians();
        let rotation_matrix = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, angle_rad.cos(), -angle_rad.sin(), 0.0,
            0.0, angle_rad.sin(), angle_rad.cos(), 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        let centroid = self.cubes[idx].centroid();

        self.cubes[idx].translate(-centroid);
        self.cubes[idx].raw = rotation_matrix * &self.cubes[idx].raw;
        self.cubes[idx].translate(centroid);    
    }

    fn rotate_y(&mut self, idx: usize) {
        let angle_rad = self.obj_transform.rotation.y.to_radians();
        let rotation_matrix = Matrix4::new(
            angle_rad.cos(), 0.0, angle_rad.sin(), 0.0,
            0.0, 1.0, 0.0, 0.0,
            -angle_rad.sin(), 0.0, angle_rad.cos(), 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        let centroid = self.cubes[idx].centroid();

        self.cubes[idx].translate(-centroid);
        self.cubes[idx].raw = rotation_matrix * &self.cubes[idx].raw;
        self.cubes[idx].translate(centroid);    
    }
    fn rotate_z(&mut self, idx: usize) {
        let angle_rad = self.obj_transform.rotation.z.to_radians();
        let rotation_matrix = Matrix4::new(
            angle_rad.cos(), -angle_rad.sin(), 0.0, 0.0,
            angle_rad.sin(), angle_rad.cos(), 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        let centroid = self.cubes[idx].centroid();

        self.cubes[idx].translate(-centroid);
        self.cubes[idx].raw = rotation_matrix * &self.cubes[idx].raw;
        self.cubes[idx].translate(centroid);  
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       
        // Renderiza a cena no buffer
        self.render_scene();

        // Cria a imagem a partir do buffer de pixels (p/ o egui renderizar como textura)
        let image = ColorImage {
            size: [WIDTH, HEIGHT],
            source_size: Vec2::new(WIDTH as f32, HEIGHT as f32),
            pixels: self.pixels.clone(),
        };

        // LÓGICA DE HOVER / CLICK ---
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
                    if ui.button("Transformar Objeto").clicked() {
                        self.translate(idx);
                        self.escale(idx);
                        self.rotate_x(idx);
                        self.rotate_y(idx);
                        self.rotate_z(idx);
                    }
                    if ui.button("Resetar").clicked() {
                        self.obj_transform = ObjectTransform::default();
                    }
                });
                
                ui.separator();

                ui.label("Ka (Ambiente):");
                ui.color_edit_button_rgb(&mut self.cubes[idx].params.ka);
                ui.label("Kd (Difusa):");
                ui.color_edit_button_rgb(&mut self.cubes[idx].params.kd);
                ui.label("Ks (Especular):");
                ui.color_edit_button_rgb(&mut self.cubes[idx].params.ks);
                ui.label("Expoente de Brilho (n):");
                ui.add(egui::DragValue::new(&mut self.cubes[idx].params.n).speed(0.1).range(1.0..=100.0));
                
                ui.separator();

                ui.horizontal(|ui| {
                    
                    if ui.button("Cancelar").clicked() {
                        self.obj_transform = ObjectTransform::default();
                        self.selected_cube_index = None;
                        self.hovered_cube_index = None;
                    }
                
                    if ui.button("Remover Cubo").clicked() {
                        self.cubes.remove(idx);
                        self.obj_transform = ObjectTransform::default();
                        self.selected_cube_index = None;
                        self.hovered_cube_index = None;
                    }
                });
            } else {
                ui.label("Nenhum objeto selecionado.");
                ui.label("Clique em um cubo na tela para editar.");
            }
            
            ui.separator();

            ui.collapsing("Câmera", |ui| {
                ui.label("VRP (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.scene.vrp.x));
                    ui.add(egui::DragValue::new(&mut self.scene.vrp.y));
                    ui.add(egui::DragValue::new(&mut self.scene.vrp.z));
                });
                ui.label("ViewUp (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.scene.view_up.x));
                    ui.add(egui::DragValue::new(&mut self.scene.view_up.y));
                    ui.add(egui::DragValue::new(&mut self.scene.view_up.z));
                });
                ui.label("Ponto Focal (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.scene.p.x));
                    ui.add(egui::DragValue::new(&mut self.scene.p.y));
                    ui.add(egui::DragValue::new(&mut self.scene.p.z));
                });
            });

            ui.separator();

            ui.collapsing("Projeção", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Su:");
                    ui.add(egui::DragValue::new(&mut self.scene.su));
                    ui.label("Sv:");
                    ui.add(egui::DragValue::new(&mut self.scene.sv));
                });
                ui.horizontal(|ui| {
                    ui.label("Near:");
                    ui.add(egui::DragValue::new(&mut self.scene.near));
                    ui.label("Far:");
                    ui.add(egui::DragValue::new(&mut self.scene.far));
                });
                ui.horizontal(|ui| {
                    ui.label("Cu:");
                    ui.add(egui::DragValue::new(&mut self.scene.cu));
                    ui.label("Cv:");
                    ui.add(egui::DragValue::new(&mut self.scene.cv));
                });
                ui.horizontal(|ui| {
                    ui.label("Distância do Plano:");
                    ui.add(egui::DragValue::new(&mut self.scene.dp));
                });
            });

            ui.separator();

            ui.collapsing("Tela", |ui| {
                ui.label("Umin:");
                ui.add(egui::Slider::new(&mut self.scene.u_min, 0.0..=WIDTH as f32 - 1.0 ).step_by(10.0));
                ui.label("Umax:");
                ui.add(egui::Slider::new(&mut self.scene.u_max, 0.0..=WIDTH as f32 - 1.0 ).step_by(10.0));
                ui.label("Vmin:");
                ui.add(egui::Slider::new(&mut self.scene.v_min, 0.0..=HEIGHT as f32 - 1.0 ).step_by(10.0));
                ui.label("Vmax:");
                ui.add(egui::Slider::new(&mut self.scene.v_max, 0.0..=HEIGHT as f32 - 1.0 ).step_by(10.0));
            });

            ui.separator();

            ui.collapsing("Iluminação", |ui| {
                ui.label("Posição da Lâmpada (X, Y, Z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.scene.lamp_pos.x));
                    ui.add(egui::DragValue::new(&mut self.scene.lamp_pos.y));
                    ui.add(egui::DragValue::new(&mut self.scene.lamp_pos.z));
                });
                ui.label("Intensidade da Lâmpada:");
                ui.color_edit_button_rgb(&mut self.scene.i_lamp);
                ui.separator();
                ui.label("Luz Ambiente:");
                ui.color_edit_button_rgb(&mut self.scene.i_amb);
            });
            
            ui.add_space(20.0);

            ui.checkbox(&mut self.scene.use_phong, "Modelo Phong");

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

            if !ui.ui_contains_pointer() {
                self.mouse_in_buffer = None; // reseta se o mouse sair da área
            }
            
            // Verifica se o mouse está em cima da imagem renderizada
            if let Some(pos) = img_response.hover_pos() {
                // 'pos' é relativo à janela inteira.
                // Precisamos converter para coordenadas relativas à imagem (0,0 no canto da imagem)
                let rel_x = pos.x - img_response.rect.min.x;
                let rel_y = pos.y - img_response.rect.min.y;

                // Valida limites
                if rel_x >= 0.0
                    && rel_y >= 0.0
                    && rel_x < WIDTH as f32
                    && rel_y < HEIGHT as f32
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
            .with_inner_size([1500.0, 900.0])
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
