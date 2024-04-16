mod chunk;
mod error;

fn main() {
    let mut test_chunk = chunk::Chunk::new();
    test_chunk.write_instruction(chunk::OpCode::OpReturn);
    test_chunk.write_byte(3);
    test_chunk.write_instruction(chunk::OpCode::OpReturn);
    match test_chunk.dissasemble("test") {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    }
}
