import { useEffect, useRef, useState } from 'react';
import init, { SoftwareRenderer, init_panic_hook } from '../motor/pkg/';

const WIDTH = 800;
const HEIGHT = 800;

function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<SoftwareRenderer | null>(null);
  // Precisamos de uma referência ao módulo WASM para acessar a memória
  const wasmMemory = useRef<WebAssembly.Memory | null>(null);
  const rotationRef = useRef({ x: 0, y: 0 });
  const needsRender = useRef(true);

  const handleMouseMove = (e: React.MouseEvent) => {
    if (e.buttons === 1) {
      rotationRef.current.x += e.movementY;
      rotationRef.current.y += e.movementX;
      console.log(`Mouse Move: ${e.movementX}, ${e.movementY}`);
      needsRender.current = true;
    }
  };

  useEffect(() => {
    let animId: number;
    let ctx: CanvasRenderingContext2D | null;
    let bufferPtr: number;
    let array: Uint8ClampedArray<ArrayBuffer>;
    let imageData: ImageData;

    try {
      init_panic_hook();
    } catch (e) {
      console.log("Panic hook já inicializado");
    }

    const startApp = async () => {
      // Aguarda o carregamento do WASM
      const wasmModule = await init();
      engineRef.current = new SoftwareRenderer(WIDTH, HEIGHT);
      wasmMemory.current = wasmModule.memory;

      const tick = () => {
        // Só gasta CPU do Rust se for necessário
        if (needsRender.current && engineRef.current && canvasRef.current) {

          // A. Chama o Rust (Pesado)
          engineRef.current.render_frame(rotationRef.current.x, rotationRef.current.y);

          // B. Copia para o Canvas (Pesado)
          ctx = canvasRef.current.getContext('2d');
          bufferPtr = engineRef.current.get_color_ptr();
          array = new Uint8ClampedArray(
            wasmMemory.current!.buffer,
            bufferPtr,
            WIDTH * HEIGHT * 4
          );

          imageData = new ImageData(array, WIDTH, HEIGHT);

          ctx?.putImageData(imageData, 0, 0);

          // C. Desliga a flag
          needsRender.current = false;
        }

        // Continua checando no próximo frame (baixo custo se needsRender for false)
        animId = requestAnimationFrame(tick);
      };

      tick();
    };

    startApp();

    return () => cancelAnimationFrame(animId);
  }, []);

  return (
    <div style={{ textAlign: 'center' }}>
      <h1>{WIDTH} x {HEIGHT}</h1>
      <canvas onMouseMove={handleMouseMove} ref={canvasRef} width={WIDTH} height={HEIGHT} style={{ border: '1px solid #ccc' }} />
    </div>
  );
}

export default App;