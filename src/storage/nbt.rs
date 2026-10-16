// REA Verified: net.minecraft.nbt.NbtIo (mapped: ta.class)
// REA Verified: net.minecraft.nbt.CompoundTag (mapped: ub.class)
//
// Minecraft Named Binary Tag (NBT) Format Reader & Writer:
// - Authentic big-endian binary encoding for all primitive & collection tags.
// - Full support for Compounds, Lists, ByteArrays, Strings, Ints, Longs.
// - Zero external NBT crate dependencies; robust, fast, and compliant with vanilla.

use std::collections::HashMap;
use std::io::{self, Cursor, Read, Write};

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

/// A Minecraft NBT Value Tag
#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(u8, Vec<NbtTag>), // element_type, elements
    Compound(HashMap<String, NbtTag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtTag {
    pub fn tag_type(&self) -> u8 {
        match self {
            NbtTag::Byte(_) => TAG_BYTE,
            NbtTag::Short(_) => TAG_SHORT,
            NbtTag::Int(_) => TAG_INT,
            NbtTag::Long(_) => TAG_LONG,
            NbtTag::Float(_) => TAG_FLOAT,
            NbtTag::Double(_) => TAG_DOUBLE,
            NbtTag::ByteArray(_) => TAG_BYTE_ARRAY,
            NbtTag::String(_) => TAG_STRING,
            NbtTag::List(elem_type, _) => {
                let _ = elem_type;
                TAG_LIST
            }
            NbtTag::Compound(_) => TAG_COMPOUND,
            NbtTag::IntArray(_) => TAG_INT_ARRAY,
            NbtTag::LongArray(_) => TAG_LONG_ARRAY,
        }
    }

    pub fn as_compound(&self) -> Option<&HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_compound_mut(&mut self) -> Option<&mut HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let NbtTag::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    pub fn as_byte_array(&self) -> Option<&[u8]> {
        if let NbtTag::ByteArray(bytes) = self {
            Some(bytes.as_slice())
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&[NbtTag]> {
        if let NbtTag::List(_, list) = self {
            Some(list.as_slice())
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        if let NbtTag::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_byte(&self) -> Option<i8> {
        if let NbtTag::Byte(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

/// Helper wrapper for constructing Compounds conveniently
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NbtCompound {
    pub map: HashMap<String, NbtTag>,
}

impl NbtCompound {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, tag: NbtTag) {
        self.map.insert(key.into(), tag);
    }

    pub fn get(&self, key: &str) -> Option<&NbtTag> {
        self.map.get(key)
    }

    pub fn get_compound(&self, key: &str) -> Option<&HashMap<String, NbtTag>> {
        self.map.get(key).and_then(|t| t.as_compound())
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.map.get(key).and_then(|t| t.as_string())
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.map.get(key).and_then(|t| t.as_int())
    }

    pub fn get_byte(&self, key: &str) -> Option<i8> {
        self.map.get(key).and_then(|t| t.as_byte())
    }

    pub fn get_byte_array(&self, key: &str) -> Option<&[u8]> {
        self.map.get(key).and_then(|t| t.as_byte_array())
    }

    pub fn get_list(&self, key: &str) -> Option<&[NbtTag]> {
        self.map.get(key).and_then(|t| t.as_list())
    }
}

// ---------------------------------------------------------------------------
// Serialization (Writing)
// ---------------------------------------------------------------------------

pub fn write_tag_payload<W: Write>(writer: &mut W, tag: &NbtTag) -> io::Result<()> {
    match tag {
        NbtTag::Byte(b) => writer.write_all(&[*b as u8]),
        NbtTag::Short(s) => writer.write_all(&s.to_be_bytes()),
        NbtTag::Int(i) => writer.write_all(&i.to_be_bytes()),
        NbtTag::Long(l) => writer.write_all(&l.to_be_bytes()),
        NbtTag::Float(f) => writer.write_all(&f.to_be_bytes()),
        NbtTag::Double(d) => writer.write_all(&d.to_be_bytes()),
        NbtTag::ByteArray(bytes) => {
            let len = bytes.len() as i32;
            writer.write_all(&len.to_be_bytes())?;
            writer.write_all(bytes)
        }
        NbtTag::String(s) => {
            let bytes = s.as_bytes();
            let len = bytes.len() as u16;
            writer.write_all(&len.to_be_bytes())?;
            writer.write_all(bytes)
        }
        NbtTag::List(elem_type, items) => {
            writer.write_all(&[*elem_type])?;
            let len = items.len() as i32;
            writer.write_all(&len.to_be_bytes())?;
            for item in items {
                write_tag_payload(writer, item)?;
            }
            Ok(())
        }
        NbtTag::Compound(map) => {
            for (key, val) in map {
                let tag_id = val.tag_type();
                writer.write_all(&[tag_id])?;
                let key_bytes = key.as_bytes();
                let key_len = key_bytes.len() as u16;
                writer.write_all(&key_len.to_be_bytes())?;
                writer.write_all(key_bytes)?;
                write_tag_payload(writer, val)?;
            }
            // Terminate compound with TAG_End (0)
            writer.write_all(&[TAG_END])
        }
        NbtTag::IntArray(ints) => {
            let len = ints.len() as i32;
            writer.write_all(&len.to_be_bytes())?;
            for i in ints {
                writer.write_all(&i.to_be_bytes())?;
            }
            Ok(())
        }
        NbtTag::LongArray(longs) => {
            let len = longs.len() as i32;
            writer.write_all(&len.to_be_bytes())?;
            for l in longs {
                writer.write_all(&l.to_be_bytes())?;
            }
            Ok(())
        }
    }
}

/// Writes a root NBT Compound with the given name (typically "" in vanilla)
pub fn write_root_compound<W: Write>(writer: &mut W, name: &str, compound: &NbtCompound) -> io::Result<()> {
    // Root tag header: TAG_Compound (10), name length, name string
    writer.write_all(&[TAG_COMPOUND])?;
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len() as u16;
    writer.write_all(&name_len.to_be_bytes())?;
    writer.write_all(name_bytes)?;

    let root_tag = NbtTag::Compound(compound.map.clone());
    write_tag_payload(writer, &root_tag)
}

/// Serializes root compound into a byte vector
pub fn serialize_compound(compound: &NbtCompound) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    write_root_compound(&mut buf, "", compound)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Deserialization (Reading)
// ---------------------------------------------------------------------------

fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut len_buf = [0u8; 2];
    reader.read_exact(&mut len_buf)?;
    let len = u16::from_be_bytes(len_buf) as usize;
    let mut str_buf = vec![0u8; len];
    reader.read_exact(&mut str_buf)?;
    String::from_utf8(str_buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn read_tag_payload<R: Read>(reader: &mut R, tag_type: u8) -> io::Result<NbtTag> {
    match tag_type {
        TAG_BYTE => {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::Byte(buf[0] as i8))
        }
        TAG_SHORT => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::Short(i16::from_be_bytes(buf)))
        }
        TAG_INT => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::Int(i32::from_be_bytes(buf)))
        }
        TAG_LONG => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::Long(i64::from_be_bytes(buf)))
        }
        TAG_FLOAT => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::Float(f32::from_be_bytes(buf)))
        }
        TAG_DOUBLE => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::Double(f64::from_be_bytes(buf)))
        }
        TAG_BYTE_ARRAY => {
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let len = i32::from_be_bytes(len_buf).max(0) as usize;
            let mut data = vec![0u8; len];
            reader.read_exact(&mut data)?;
            Ok(NbtTag::ByteArray(data))
        }
        TAG_STRING => {
            let s = read_string(reader)?;
            Ok(NbtTag::String(s))
        }
        TAG_LIST => {
            let mut elem_type_buf = [0u8; 1];
            reader.read_exact(&mut elem_type_buf)?;
            let elem_type = elem_type_buf[0];

            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let len = i32::from_be_bytes(len_buf).max(0) as usize;

            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                items.push(read_tag_payload(reader, elem_type)?);
            }
            Ok(NbtTag::List(elem_type, items))
        }
        TAG_COMPOUND => {
            let mut map = HashMap::new();
            loop {
                let mut type_buf = [0u8; 1];
                reader.read_exact(&mut type_buf)?;
                let child_type = type_buf[0];
                if child_type == TAG_END {
                    break;
                }
                let name = read_string(reader)?;
                let child_payload = read_tag_payload(reader, child_type)?;
                map.insert(name, child_payload);
            }
            Ok(NbtTag::Compound(map))
        }
        TAG_INT_ARRAY => {
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let len = i32::from_be_bytes(len_buf).max(0) as usize;
            let mut ints = Vec::with_capacity(len);
            for _ in 0..len {
                let mut int_buf = [0u8; 4];
                reader.read_exact(&mut int_buf)?;
                ints.push(i32::from_be_bytes(int_buf));
            }
            Ok(NbtTag::IntArray(ints))
        }
        TAG_LONG_ARRAY => {
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let len = i32::from_be_bytes(len_buf).max(0) as usize;
            let mut longs = Vec::with_capacity(len);
            for _ in 0..len {
                let mut long_buf = [0u8; 8];
                reader.read_exact(&mut long_buf)?;
                longs.push(i64::from_be_bytes(long_buf));
            }
            Ok(NbtTag::LongArray(longs))
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown NBT tag type: {other}"),
        )),
    }
}

/// Reads a named root NBT Compound
pub fn read_root_compound<R: Read>(reader: &mut R) -> io::Result<(String, NbtCompound)> {
    let mut type_buf = [0u8; 1];
    reader.read_exact(&mut type_buf)?;
    if type_buf[0] != TAG_COMPOUND {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected root compound tag (10), found {}", type_buf[0]),
        ));
    }

    let root_name = read_string(reader)?;
    let payload = read_tag_payload(reader, TAG_COMPOUND)?;
    if let NbtTag::Compound(map) = payload {
        Ok((root_name, NbtCompound { map }))
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Expected compound payload"))
    }
}

/// Deserializes root compound from a byte slice
pub fn deserialize_compound(data: &[u8]) -> io::Result<NbtCompound> {
    let mut cursor = Cursor::new(data);
    let (_name, compound) = read_root_compound(&mut cursor)?;
    Ok(compound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nbt_compound_roundtrip() {
        let mut comp = NbtCompound::new();
        comp.insert("xPos", NbtTag::Int(15));
        comp.insert("zPos", NbtTag::Int(-4));
        comp.insert("Status", NbtTag::String("full".to_string()));

        let mut sub = HashMap::new();
        sub.insert("Y".to_string(), NbtTag::Byte(2));
        sub.insert("BlockLight".to_string(), NbtTag::ByteArray(vec![15, 14, 13, 12]));
        comp.insert("section", NbtTag::Compound(sub));

        let bytes = serialize_compound(&comp).expect("Serialization failed");
        assert!(!bytes.is_empty());

        let decoded = deserialize_compound(&bytes).expect("Deserialization failed");
        assert_eq!(decoded.get_int("xPos"), Some(15));
        assert_eq!(decoded.get_int("zPos"), Some(-4));
        assert_eq!(decoded.get_string("Status"), Some("full"));

        let sec = decoded.get_compound("section").expect("Missing section");
        assert_eq!(sec.get("Y"), Some(&NbtTag::Byte(2)));
        assert_eq!(sec.get("BlockLight"), Some(&NbtTag::ByteArray(vec![15, 14, 13, 12])));
    }
}
