use opcode::Opcode;

pub fn disassemble(buffer: &[u8]) {
    println!("HEX\tOP\tARG1\tARG2\tARG3");
    println!("---\t--\t----\t----\t----");
    for chunk in buffer.chunks(2) {
        let first_byte = chunk[0];
        let second_byte = chunk[1];

        print!("{:#02x}{:02x}\t", first_byte, second_byte);
        if let Ok(opcode) = Opcode::from(first_byte, second_byte) {
            print!("{:?}", opcode);
        }

        println!();
    }
}
