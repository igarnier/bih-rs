use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, one_of},
    combinator::{map, map_res, recognize},
    error::ParseError,
    multi::{many0, many1},
    number::complete::float,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct V3(pub f32, pub f32, pub f32);

#[derive(Debug)]
pub struct Triangle(pub u32, pub u32, pub u32);

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<V3>,
    pub normals: Vec<V3>,
    pub triangles: Vec<Triangle>,
}

enum Item {
    V(V3),
    VN(V3),
    VT(V3),
    F(Triangle),
}

fn empty_mesh() -> Mesh {
    Mesh {
        vertices: Vec::new(),
        normals: Vec::new(),
        triangles: Vec::new(),
    }
}

pub fn decimal(input: &str) -> IResult<&str, u32> {
    map_res(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |out: &str| u32::from_str_radix(out, 10),
    )(input)
}

pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn parse_v3(input: &str) -> IResult<&str, V3> {
    let (remaining, (x, (y, z))) =
        separated_pair(float, tag(" "), separated_pair(float, tag(" "), float))(input)?;
    Ok((remaining, V3(x, y, z)))
}

pub fn parse_triangle(input: &str) -> IResult<&str, Triangle> {
    let (remaining, (t0, (t1, t2))) = separated_pair(
        decimal,
        tag(" "),
        separated_pair(decimal, tag(" "), decimal),
    )(input)?;
    Ok((remaining, Triangle(t0, t1, t2)))
}

fn parse_vertex(input: &str) -> IResult<&str, Item> {
    map(preceded(ws(tag("v")), parse_v3), |x| Item::V(x))(input)
}

fn parse_vertex_texcoord(input: &str) -> IResult<&str, Item> {
    map(preceded(ws(tag("vt")), parse_v3), |x| Item::VT(x))(input)
}

fn parse_vertex_normal(input: &str) -> IResult<&str, Item> {
    map(preceded(ws(tag("vn")), parse_v3), |x| Item::VN(x))(input)
}

fn parse_face(input: &str) -> IResult<&str, Item> {
    map(preceded(ws(tag("f")), parse_triangle), |x| Item::F(x))(input)
}

fn parse_line(input: &str) -> IResult<&str, Item> {
    alt((
        parse_vertex,
        parse_vertex_texcoord,
        parse_vertex_normal,
        parse_face,
    ))(input)
}

pub fn load(filename: &str) -> Mesh {
    let mut buf = Vec::with_capacity(128);

    let mut fd = File::open(filename).unwrap();
    fd.read_to_end(&mut buf).unwrap();

    let mut mesh = empty_mesh();

    buf.split(|&c| c == b'\n')
        .into_iter()
        .fold((), |_acc, line| {
            if line.len() == 0 {
                ()
            } else if line[0] == b'#' {
                ()
            } else {
                let line_as_str = std::str::from_utf8(line).unwrap();
                let (_remaining, parse_result) = parse_line(line_as_str).unwrap();
                match parse_result {
                    Item::V(v) => mesh.vertices.push(v),
                    Item::F(t) => mesh.triangles.push(t),
                    Item::VN(v) => mesh.normals.push(v),
                    Item::VT(_) => (),
                }
            }
        });

    mesh
}

// use nom::{
//   IResult,
//   bytes::complete::{tag, take_while_m_n},
//   combinator::map_res,
//   sequence::tuple};

// #[derive(Debug,PartialEq)]
// pub struct Color {
//   pub red:     u8,
//   pub green:   u8,
//   pub blue:    u8,
// }

// fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
//   u8::from_str_radix(input, 16)
// }

// fn is_hex_digit(c: char) -> bool {
//   c.is_digit(16)
// }

// fn hex_primary(input: &str) -> IResult<&str, u8> {
//   map_res(
//     take_while_m_n(2, 2, is_hex_digit),
//     from_hex
//   )(input)
// }

// fn hex_color(input: &str) -> IResult<&str, Color> {
//   let (input, _) = tag("#")(input)?;
//   let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

//   Ok((input, Color { red, green, blue }))
// }

// fn main() {
//   assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
//     red: 47,
//     green: 20,
//     blue: 223,
//   })));
// }
