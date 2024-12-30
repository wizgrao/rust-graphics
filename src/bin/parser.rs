use graphics::path_tracer::obj::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    dbg!(read_obj_file(&path).unwrap());
}
