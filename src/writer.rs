use std::io::{ Write, Result };
use std::string::ToString;
use ply::*;

pub struct Writer {}

impl Writer {
    pub fn new() -> Self {
        Writer {}
    }
    // TODO: think about masking and valid/invalid symbols
    // TODO: make consistency check
    pub fn write_ply<T: Write>(&mut self, out: &mut T, ply: &Ply) -> Result<usize> {
        let mut written = 0;
        written += try!(self.write_header(out, &ply.header));
        written += try!(self.write_payload(out, &ply.payload));
        out.flush().unwrap();
        Ok(written)
    }
    pub fn write_line_magic_number<T: Write>(&self, out: &mut T) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write("ply".as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_line_format<T: Write>(&self, out: &mut T, encoding: &Encoding, version: &Version) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write("format ".as_bytes()));
        written += try!(self.write_encoding(out, encoding));
        written += try!(out.write(format!(" {}.{}", version.major, version.minor).as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_line_comment<T: Write>(&self, out: &mut T, comment: &Comment) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write(format!("comment {}", comment).as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_line_obj_info<T: Write>(&self, out: &mut T, obj_info: &ObjInfo) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write(format!("obj_info {}", obj_info).as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_line_element_decl<T: Write>(&self, out: &mut T, element: &Element) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write(format!("element {} {}", element.name, element.count).as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_line_property_decl<T: Write>(&self, out: &mut T, property: &Property) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write("property ".as_bytes()));
        written += try!(self.write_property_type(out, &property.data_type));
        written += try!(out.write(" ".as_bytes()));
        written += try!(out.write(property.name.as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_element_decl<T: Write>(&self, out: &mut T, element: &Element) -> Result<usize> {
        let mut written = 0;
        written += try!(self.write_line_element_decl(out, element));
        for (_, p) in &element.properties {
            written += try!(self.write_line_property_decl(out, &p));
        }
        Ok(written)
    }
    pub fn write_line_end_header<T: Write>(&mut self, out: &mut T) -> Result<usize> {
        let mut written = 0;
        written += try!(out.write("end_header".as_bytes()));
        written += try!(self.write_new_line(out));
        Ok(written)
    }
    pub fn write_header<T: Write>(&mut self, out: &mut T, header: &Header) -> Result<usize> {
        let mut written = 0;
        written += try!(self.write_line_magic_number(out));
        written += try!(self.write_line_format(out, &header.encoding, &header.version));
        for c in &header.comments {
            written += try!(self.write_line_comment(out, c));
        }
        for oi in &header.obj_infos {
            written += try!(self.write_line_obj_info(out, oi));
        }
        for (_, e) in &header.elements {
            written += try!(self.write_element_decl(out, &e));
        }
        written += try!(self.write_line_end_header(out));
        Ok(written)
    }
    pub fn write_payload<T: Write>(&mut self, out: &mut T, payload: &Payload) -> Result<usize> {
        let mut written = 0;
        for (_, elems) in payload {
            for e in elems {
                written += try!(self.write_line_payload_element(out, e));
            }
        }
        Ok(written)
    }
    pub fn write_line_payload_element<T: Write>(&mut self, out: &mut T, element: &PayloadElement) -> Result<usize> {
        let mut written = 0;
        let mut p_iter = element.iter();
        let (_name, prop_val) = p_iter.next().unwrap();
        written += try!(self.write_payload_property(out, prop_val));
        loop {
            written += try!(out.write(" ".as_bytes()));
            let n = p_iter.next();
            if n == None {
                break;
            }
            let (_name, prop_val) = n.unwrap();
            written += try!(self.write_payload_property(out, prop_val));
        }
        written += try!(self.write_new_line(out));
        Ok(written)
    }
}
impl Writer {
    fn write_encoding<T: Write>(&self, out: &mut T, encoding: &Encoding) -> Result<usize> {
        let s = match *encoding {
            Encoding::Ascii => "ascii",
            Encoding::BinaryBigEndian => "binary_big_endian",
            Encoding::BinaryLittleEndian => "binary_little_endian",
        };
        out.write(s.as_bytes())
    }
    fn write_property_type<T: Write>(&self, out: &mut T, data_type: &DataType) -> Result<usize> {
        match *data_type {
            DataType::Char => out.write("char".as_bytes()),
            DataType::UChar => out.write("uchar".as_bytes()),
            DataType::Short => out.write("short".as_bytes()),
            DataType::UShort => out.write("ushort".as_bytes()),
            DataType::Int => out.write("int".as_bytes()),
            DataType::UInt => out.write("uint".as_bytes()),
            DataType::Float => out.write("float".as_bytes()),
            DataType::Double => out.write("double".as_bytes()),
            DataType::List(ref t) => {
                let mut written = try!(out.write("list uchar ".as_bytes()));
                written += try!(self.write_property_type(out, t));
                Ok(written)
            }
        }
    }
    fn write_payload_property<T: Write>(&self, out: &mut T, data_item: &DataItem) -> Result<usize> {
         let result = match *data_item {
            DataItem::Char(ref v) => self.write_simple_value(v, out),
            DataItem::UChar(ref v) => self.write_simple_value(v, out),
            DataItem::Short(ref v) => self.write_simple_value(v, out),
            DataItem::UShort(ref v) => self.write_simple_value(v, out),
            DataItem::Int(ref v) => self.write_simple_value(v, out),
            DataItem::UInt(ref v) => self.write_simple_value(v, out),
            DataItem::Float(ref v) => self.write_simple_value(v, out),
            DataItem::Double(ref v) => self.write_simple_value(v, out),
            DataItem::List(ref v) => {
                let mut written = 0;
                written += try!(out.write(&v.len().to_string().as_bytes()));
                for e in v {
                    written += try!(out.write(" ".as_bytes()));
                    written += try!(self.write_payload_property(out, &e));
                }
                Ok(written)
            },
        };
        result
    }

    fn write_new_line<T: Write>(&self, out: &mut T) -> Result<usize> {
        out.write("\r\n".as_bytes())
    }
    fn write_simple_value<T: Write, V: ToString>(&self, value: &V, out: &mut T) -> Result<usize> {
        out.write(value.to_string().as_bytes())
    }
}