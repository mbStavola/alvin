// Sprite data borrowed from https://github.com/massung/CHIP-8/blob/master/chip8/rom.go
const SPRITE_DATA: [u8; 0x1C0] = [
    // 4x5 low-res mode font sprites (0-F)
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0xA0, 0xA0, 0xF0, 0x20, 0x20, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
    // 8x10 high-res mode font sprites (0-F)
    0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, 0x18, 0x38, 0x58, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x3C, 0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, 0x3C, 0x7E,
    0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, 0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF,
    0x06, 0x06, 0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, 0x3E, 0x7C, 0xC0, 0xC0,
    0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, 0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60,
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, 0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F,
    0x03, 0x03, 0x3E, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // 6-bit ASCII character patterns
    0x00, // |        |
    0x10, // |   *    |
    0x20, // |  *     |
    0x88, // |*   *   |
    0xA8, // |* * *   |
    0x50, // | * *    |
    0xF8, // |*****   |
    0x70, // | ***    |
    0x80, // |*       |
    0x90, // |*  *    |
    0xA0, // |* *     |
    0xB0, // |* **    |
    0xC0, // |**      |
    0xD0, // |** *    |
    0xE0, // |***     |
    0xF0, // |****    |
    // 6-bit ASCII characters from 0x100-
    0x46, 0x3E, 0x56, // @
    0x99, 0x9F, 0x4F, // A
    0x5F, 0x57, 0x4F, // B
    0x8F, 0x88, 0x4F, // C
    0x5F, 0x55, 0x4F, // D
    0x8F, 0x8F, 0x4F, // E
    0x88, 0x8F, 0x4F, // F
    0x9F, 0x8B, 0x4F, // G
    0x99, 0x9F, 0x49, // H
    0x27, 0x22, 0x47, // I
    0xAE, 0x22, 0x47, // J
    0xA9, 0xAC, 0x49, // K
    0x8F, 0x88, 0x48, // L
    0x43, 0x64, 0x53, // M
    0x99, 0xDB, 0x49, // N
    0x9F, 0x99, 0x4F, // O
    0x88, 0x9F, 0x4F, // P
    0x9F, 0x9B, 0x4F, // Q
    0xA9, 0x9F, 0x4F, // R
    0x1F, 0x8F, 0x4F, // S
    0x22, 0x22, 0x56, // T
    0x9F, 0x99, 0x49, // U
    0x22, 0x55, 0x53, // V
    0x55, 0x44, 0x54, // W
    0x53, 0x52, 0x53, // X
    0x22, 0x52, 0x53, // Y
    0xCF, 0x12, 0x4F, // Z
    0x8C, 0x88, 0x3C, // [
    0x10, 0xC2, 0x40, // \
    0x2E, 0x22, 0x3E, // ]
    0x30, 0x25, 0x50, // ^
    0x06, 0x00, 0x50, // _
    0x00, 0x00, 0x40, // space
    0x0C, 0xCC, 0x2C, // !
    0x00, 0x50, 0x45, // "
    0x65, 0x65, 0x55, // #
    0x46, 0x46, 0x56, // $
    0xDF, 0xBF, 0x4F, // %
    0x5F, 0xAF, 0x4E, // &
    0x00, 0x80, 0x18, // '
    0x21, 0x22, 0x41, // (
    0x12, 0x11, 0x42, // )
    0x53, 0x56, 0x53, // *
    0x22, 0x26, 0x52, // +
    0x2E, 0x00, 0x30, // ,
    0x00, 0x06, 0x50, // -
    0xCC, 0x00, 0x20, // .
    0xC0, 0x12, 0x40, // /
    0x9F, 0x99, 0x4F, // 0
    0x22, 0x22, 0x32, // 1
    0x8F, 0x1F, 0x4F, // 2
    0x1F, 0x1F, 0x4F, // 3
    0x22, 0xAF, 0x4A, // 4
    0x1F, 0x8F, 0x4F, // 5
    0x9F, 0x8F, 0x4F, // 6
    0x11, 0x11, 0x4F, // 7
    0x9F, 0x9F, 0x4F, // 8
    0x1F, 0x9F, 0x4F, // 9
    0x80, 0x80, 0x10, // :
    0x2E, 0x20, 0x30, // ;
    0x21, 0x2C, 0x41, // <
    0xE0, 0xE0, 0x30, // =
    0x2C, 0x21, 0x4C, // >
    0x88, 0x1F, 0x4F, // ?
];

pub fn load_fonts(memory: &mut [u8]) {
    let mut i = 0x0;
    for sprite in SPRITE_DATA.iter() {
        memory[i] = *sprite;
        i += 1;
    }
}

pub fn load_program(memory: &mut [u8], program: &[u8]) {
    let mut current_address = 0x200;
    for byte in program.iter() {
        if current_address == 0xEA0 {
            break;
        }

        memory[current_address] = *byte;
        current_address += 1;
    }
}
