use eframe::egui;
use egui::{ColorImage, TextureHandle, Vec2};

struct MyApp {
    width: usize,
    height: usize,
    pixels: Vec<egui::Color32>,
    texture: Option<TextureHandle>,
    // Adicionamos um contador simples para animação
    frame_count: usize, 
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Vamos usar 800x600 como base
        let width = 800;
        let height = 600;
        Self {
            width,
            height,
            // Inicializa com preto
            pixels: vec![egui::Color32::BLACK; width * height], 
            texture: None,
            frame_count: 0,
        }
    }

    fn render_scene(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        let time_offset = self.frame_count as usize;

        // --- TESTE DE RASTERIZAÇÃO ---
        // Percorre cada pixel e gera uma cor baseada na posição (X, Y) e no Tempo.
        // Isso cria um padrão visual que confirma se o buffer está mapeado corretamente.
        
        for y in 0..self.height {
            for x in 0..self.width {
                // Índice linear do buffer
                let index = y * self.width + x;

                // Lógica simples de teste visual:
                // R: Baseado no X
                // G: Baseado no Y
                // B: Baseado no Tempo (para ver se está animando)
                
                let r = (x % 255) as u8;
                let g = (y % 255) as u8;
                
                // Um padrão "alienígena" (XOR pattern) clássico de demoscene
                // Se isso aparecer, sua escrita de memória está perfeita.
                let pattern = ((x ^ y) + time_offset) % 255; 
                let b = pattern as u8;

                self.pixels[index] = egui::Color32::from_rgb(r, g, b);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Renderiza (CPU)
        self.render_scene();

        // 2. Converte para ColorImage
        // Nota: Clonar 800x600 a 60FPS é "ok" em desktop, mas otimizações existem se precisar.
        let image = ColorImage {
            size: [self.width, self.height],
            source_size: Vec2::new(self.width as f32, self.height as f32),
            pixels: self.pixels.clone(), 
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Teste do Rasterizador");

            // 3. Gerencia a Textura na GPU
            let texture = self.texture.get_or_insert_with(|| {
                // Carrega inicial
                ui.ctx().load_texture(
                    "framebuffer",
                    image.clone(),
                    egui::TextureOptions::NEAREST // NEAREST é melhor para pixel art/rasterizadores retro
                )
            });

            // Atualiza os dados da textura existente
            texture.set(image, egui::TextureOptions::NEAREST);

            // 4. Exibe a imagem
            // Sized serve para garantir que a imagem ocupe o espaço correto na UI
            ui.image((texture.id(), texture.size_vec2()));

            ui.label(format!("FPS: {:.1}", 1.0 / ctx.input(|i| i.stable_dt)));
            ui.label("Se você vê um padrão colorido se movendo, o buffer está funcionando!");
        });

        // Solicita renderização contínua (game loop)
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    // Aumentei o tamanho da janela inicial para caber o buffer de 800x600
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Rasterizador CG",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // IMPORTANTE: Chamar o new() aqui, não o default()
            Ok(Box::new(MyApp::new(cc)))
        }),
    )
}