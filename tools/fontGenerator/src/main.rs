use rusttype::{Font, Scale, point};
use std::fs::File;
use std::io::Write;

fn processar_ttf(caminho: &str, file: &mut File, nome_array: &str) {
    // 1. Lê os bytes do arquivo .ttf
    let font_data = std::fs::read(caminho)
        .unwrap_or_else(|_| panic!("Falha ao abrir o arquivo: {}", caminho));
    let font = Font::try_from_vec(font_data).expect("Erro ao processar o arquivo TTF");

    // 2. Define o tamanho da escala para 16 pixels
    let scale = Scale::uniform(16.0);
    
    // Vamos processar 128 caracteres da tabela ASCII padrão.
    // Se a sua fonte de emoji tiver os desenhos espalhados em outros índices,
    // podemos ajustar esse range depois!
    let total_caracteres = 128; 

    writeln!(file, "pub static {}: [[u16; 16]; {}] = [", nome_array, total_caracteres).unwrap();

    for i in 0..total_caracteres {
        let c = char::from(i as u8);
        write!(file, "    [").unwrap();

        // Renderiza o caractere vetorial
        let glyph = font.glyph(c).scaled(scale).positioned(point(0.0, 0.0));
        
        // Cria uma matriz temporária de 16x16 pixels
        let mut matriz = [[0u8; 16]; 16];

        if let Some(_bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                // Se a intensidade do pixel for maior que 40%, pintamos ele na matriz
                if v > 0.4 && x < 16 && y < 16 {
                    matriz[y as usize][x as usize] = 1;
                }
            });
        }

        // 3. Converte as linhas da matriz 16x16 para valores de 16 bits (u16)
        for y in 0..16 {
            let mut row_value: u16 = 0;
            for x in 0..16 {
                if matriz[y][x] == 1 {
                    // Empurra o bit para a posição correta (da esquerda para a direita)
                    row_value |= 1 << (15 - x);
                }
            }
            // Cospe em formato hexadecimal de 4 dígitos (ex: 0xFC00)
            write!(file, "0x{:04X}, ", row_value).unwrap();
        }

        writeln!(file, "], // Índice {}", i).unwrap();
    }
    writeln!(file, "];\n").unwrap();
}

fn main() {
    let mut file = File::create("db_font.rs").expect("Falha ao criar o arquivo font_gerada.rs");

    // Processa a fonte de texto normal
    println!("Renderizando galactic_memesbruh03.ttf em 16x16...");
    processar_ttf("galactic_memesbruh03.ttf", &mut file, "FONT_TEXT");

    // Processa a fonte de emojis
    println!("Renderizando EmojiFont.ttf em 16x16...");
    processar_ttf("EmojiFont.ttf", &mut file, "FONT_EMOJI");

    println!("Sucesso! O arquivo 'db_font.rs' foi criado com as duas fontes em u16!");
}