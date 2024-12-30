use chumsky::prelude::*;
use std::{error, fmt};
#[derive(Debug, Clone)]
pub enum ObjLine {
    Comment(String),
    Vertex(f64, f64, f64),
    Texture(f64, f64, f64),
    VertexNormal(f64, f64, f64),
    VertexParameter(f64, f64, f64),
    Face(FaceVertex, FaceVertex, FaceVertex),
    Name(String),
    Material(String),
    UseMaterial(String),
    S(i32),
    Blankline,
}

#[derive(Debug, Clone, Copy)]
pub enum FaceVertex {
    Vertex(i32),
    VertexTexture(i32, i32),
    VertexTextureNormal(i32, i32, i32),
    VertexNormal(i32, i32),
}

pub fn obj_parser() -> impl Parser<char, Vec<ObjLine>, Error = Simple<char>> {
    let zero_padded_int = just('0')
        .repeated()
        .collect::<String>()
        .then(text::int(10))
        .map(|(a, b)| a + &b)
        .or(just('0').repeated().collect());
    let float_pos = choice((
        text::int(10)
            .then_ignore(just('.'))
            .then(zero_padded_int)
            .map(|(a, b)| format!("{}.{}", a, b).parse::<f64>().unwrap()),
        just('.')
            .ignore_then(zero_padded_int)
            .map(|b| format!("0.{}", b).parse::<f64>().unwrap()),
        text::int(10)
            .then_ignore(just('.'))
            .map(|a| format!("{}.0", a).parse::<f64>().unwrap()),
        text::int(10).map(|a: String| a.parse::<f64>().unwrap()),
    ));

    let float = just('-').ignore_then(float_pos).map(|x| -x).or(float_pos);
    let blank = empty().to(ObjLine::Blankline);
    let comment = just('#')
        .padded()
        .ignore_then(text::newline().not().repeated())
        .collect()
        .map(ObjLine::Comment);

    let name = just('o')
        .padded()
        .ignore_then(text::newline().not().repeated())
        .collect()
        .map(ObjLine::Name);

    let material = just("mtllib")
        .padded()
        .ignore_then(text::newline().not().repeated())
        .collect()
        .map(ObjLine::Material);

    let use_material = just("usemtl")
        .padded()
        .ignore_then(text::newline().not().repeated())
        .collect()
        .map(ObjLine::UseMaterial);

    let s = just("s")
        .padded()
        .ignore_then(text::newline().not().repeated())
        .collect()
        .map(|x: String| ObjLine::S(x.parse().unwrap()));

    let vertex_texture = text::keyword("vt")
        .ignore_then(float.padded())
        .then(float.padded())
        .then(float)
        .map(|((x, y), z)| ObjLine::Texture(x, y, z));

    let normal = text::keyword("vn")
        .ignore_then(float.padded())
        .then(float.padded())
        .then(float)
        .map(|((x, y), z)| ObjLine::VertexNormal(x, y, z));

    let parameter = text::keyword("vp")
        .ignore_then(float.padded())
        .then(float.padded())
        .then(float)
        .map(|((x, y), z)| ObjLine::VertexParameter(x, y, z));

    let vertex = text::keyword("v")
        .ignore_then(float.padded())
        .then(float.padded())
        .then(float)
        .map(|((x, y), z)| ObjLine::Vertex(x, y, z));

    let face_vertex = choice((
        text::int(10)
            .then_ignore(just("/"))
            .then(text::int(10))
            .then_ignore(just("/"))
            .then(text::int(10))
            .map(|((x, y), z): ((String, String), String)| {
                FaceVertex::VertexTextureNormal(
                    x.parse().unwrap(),
                    y.parse().unwrap(),
                    z.parse().unwrap(),
                )
            }),
        text::int(10)
            .then_ignore(just("//"))
            .then(text::int(10))
            .map(|(x, y): (String, String)| {
                FaceVertex::VertexNormal(x.parse().unwrap(), y.parse().unwrap())
            }),
        text::int(10)
            .then_ignore(just("/"))
            .then(text::int(10))
            .map(|(x, y): (String, String)| {
                FaceVertex::VertexTexture(x.parse().unwrap(), y.parse().unwrap())
            }),
        text::int(10).map(|x: String| FaceVertex::Vertex(x.parse().unwrap())),
    ));

    let face = text::keyword("f")
        .ignore_then(face_vertex.padded())
        .then(face_vertex.padded())
        .then(face_vertex)
        .map(|((x, y), z)| ObjLine::Face(x, y, z));

    let line = choice((
        vertex,
        comment,
        face,
        vertex_texture,
        normal,
        parameter,
        name,
        material,
        use_material,
        s,
        blank,
    ));
    line.separated_by(text::newline()).then_ignore(end())
}

#[derive(Debug, Clone)]
struct ChumWrapper(Vec<Simple<char>>);

impl fmt::Display for ChumWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid error {:?}", self.0)
    }
}

impl error::Error for ChumWrapper {}

pub fn read_obj_file(path: &str) -> Result<Vec<ObjLine>, Box<dyn error::Error>> {
    let src = std::fs::read_to_string(path)?;
    let obj = obj_parser()
        .parse(src)
        .map_err(|e| Box::new(ChumWrapper(e)))?;
    Ok(obj)
}
