# Documentação: Modelador de Cubos 3D

### Parâmetros de cena
- **Câmera:** VRP, P, ViewUp, Cu, Cv, Su, Sv, dp, near, far
- **Iluminação:**
  - Lâmpada:  
    - L $(x, y, z)$
	- Intensidade (R, G, B)
  - Luz Ambiente: (R, G, B)

### Cubos
- Todos começam com um tamanho fixo e centrados na origem
- Os parâmetros de cada um são os coeficientes: 
  - **Ambiente**: Ka (R, G, B)
  - **Difuso**: Kd (R, G, B) 
  - **Especular**: Ks (R, G, B) e $n$
 
### Pipeline

1. Inicialmente, temos um cubo com pontos $P$ no SRU
2. Aplicamos $Ml = DCBA$ em $P$ para levar o objeto para o volume de visão canônico: $P' = Ml * P$
3. Para cada face visível (usamos o teste do vetor normal) em $P'$, a recortamos contra o volume de visão usando o algoritmo de Sutherland-Hodgman, obtendo-se $P''$
4. Aplica-se a matriz de projeção: $P''' = M_p * P''$
5. Para cada coordenada de $P'''$, divide-se pelo respectivo fator $h$, obtendo-se $P''''$
6. Concatenamos e aplicamos a matriz $P''''' = MLKJ * P''''$, onde $P'''''$ está em SRT
		
### Rasterização
Para todos os objetos, para todas as faces:
    
- **Constante:**
    1. Fazer fillpolly interpolando $z$ para cada pixel
    2. Testar cada pixel em relação ao Z-buffer; caso $z$ < Z_buffer escreve no Z-buffer e escreve pixel no buffer de imagem
 - **Phong:**
     1) Fillpolly interpolando Vetor Normal $N(i, j, k)$ e Z; caso Z <  Z_Buffer:
        1. Calcula-se a cor no pixel (normalizamos $N$ que foi interpolado)
        2. Escreve $z$ no Z_buffer e a cor no buffer de imagem

