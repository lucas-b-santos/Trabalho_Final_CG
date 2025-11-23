import { useEffect, useRef, useState } from 'react';
import init, { SimpleEngine, init_panic_hook } from '../motor/pkg/';

function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<SimpleEngine | null>(null);
  // Precisamos de uma referência ao módulo WASM para acessar a memória
  const wasmMemoryRef = useRef<WebAssembly.Memory | null>(null);

  useEffect(() => {
    let animId: number;
    let isActive = true; // Flag para evitar executar se o componente desmontar

    const startApp = async () => {
      // 1. Aguarda o carregamento do WASM
      const wasmModule = await init();

      // Se o componente desmontou enquanto carregava, para tudo.
      if (!isActive) return;
      if (!canvasRef.current) return;

      // 2. Instancia a Engine e Pega a Memória
      engineRef.current = new SimpleEngine();
      wasmMemoryRef.current = wasmModule.memory;

      const ctx = canvasRef.current.getContext('2d');
      if (!ctx) return;

      // Definição das arestas (Topologia do Cubo)
      const edges = [
        [0, 1], [1, 2], [2, 3], [3, 0],
        [4, 5], [5, 6], [6, 7], [7, 4],
        [0, 4], [1, 5], [2, 6], [3, 7]
      ];

      let angle = 0;

      // 3. Define o Loop de Renderização
      const renderLoop = () => {
        if (!isActive) return;

        // Atualiza lógica
        angle += 0.02;
        const width = canvasRef.current!.width;
        const height = canvasRef.current!.height;

        const engine = engineRef.current!;
        engine.update(angle, width, height);

        // Acessa memória (Zero-Copy)
        const dataPtr = engine.get_buffer_ptr();
        const dataSize = engine.get_buffer_size();
        const wasmCells = new Float64Array(
          wasmMemoryRef.current!.buffer,
          dataPtr,
          dataSize
        );

        // Desenha
        ctx.clearRect(0, 0, width, height);
        ctx.strokeStyle = 'black';
        ctx.lineWidth = 2;
        ctx.beginPath();

        edges.forEach(([start, end]) => {
          // Multiplicamos por 2 pois cada ponto tem X e Y
          const x1 = wasmCells[start * 2];
          const y1 = wasmCells[start * 2 + 1];
          const x2 = wasmCells[end * 2];
          const y2 = wasmCells[end * 2 + 1];

          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
        });
        ctx.stroke();

        // Agenda o próximo quadro
        animId = requestAnimationFrame(renderLoop);
      };

      // 4. Inicia o Loop
      renderLoop();
    };

    startApp();

    // Função de Limpeza (Cleanup)
    return () => {
      isActive = false;
      if (animId) cancelAnimationFrame(animId);
      // Opcional: Se quiser liberar memória do Rust explicitamente:
      if (engineRef.current) engineRef.current.free();
    };
  }, []); // Executa apenas uma vez na montagem

  return (
    <div style={{ textAlign: 'center' }}>
      <h1>Wireframe 3D via Rust</h1>
      <canvas ref={canvasRef} width={600} height={400} style={{ border: '1px solid #ccc' }} />
      <p>Cálculos de projeção feitos em WASM. Renderização no Canvas 2D.</p>
    </div>
  );
}

export default App;