//! Reads FM27's portrait bundle (`portraits_assets_all.bundle`): a UnityFS archive of LZ4 blocks
//! holding one serialized file with a `NewGenPortraitsCatalog` and one inline DXT5 `Texture2D`
//! per catalog face. Face mods replace this file, so it is untrusted input: every size is
//! bounded and a file that does not fit is an error, never a panic.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::{dxt, lz4};

const MAX_BLOCKS: usize = 1 << 20;
const MAX_BLOCK_BYTES: usize = 16 << 20;
const MAX_BLOCKS_INFO_BYTES: usize = 64 << 20;
const MAX_NODES: usize = 16;
const MAX_METADATA_BYTES: usize = 64 << 20;
const MAX_TYPES: usize = 4096;
const MAX_OBJECTS: usize = 200_000;
const MAX_CATALOG_BYTES: usize = 64 << 20;
const MAX_TEXTURE_BYTES: usize = 8 << 20;
const MAX_NAME_BYTES: usize = 128;
const MAX_SIDE: usize = 1024;

const TEXTURE_2D: i32 = 28;
const MONO_BEHAVIOUR: i32 = 114;
const DXT5: i32 = 12;
const CATALOG_NAME: &[u8] = b"NewGenPortraitsCatalog";

/// Why the bundle could not be used. The text is for diagnostics and carries no path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BundleError(pub(crate) &'static str);

type Result<T> = std::result::Result<T, BundleError>;

fn fail<T>(reason: &'static str) -> Result<T> {
    Err(BundleError(reason))
}

/// One catalog face: the six values FM matches on (gender, ethnicity, skin tone, hair style,
/// hair colour, facial hair item) and where its texture object sits in the portrait file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatalogFace {
    pub(crate) properties: [u8; 6],
    texture: Option<(u64, u32)>,
}

#[cfg(test)]
impl CatalogFace {
    pub(crate) fn for_test(properties: [u8; 6]) -> Self {
        Self {
            properties,
            texture: None,
        }
    }

    pub(crate) fn has_texture(&self) -> bool {
        self.texture.is_some()
    }
}

/// The parts of the bundle kept in memory: the block table and the catalog, in FM's order.
pub(crate) struct PortraitBundle {
    blocks: Vec<Block>,
    /// Where each block starts in the uncompressed stream, plus the stream's end.
    block_starts: Vec<u64>,
    file_start: u64,
    file_len: u64,
    faces: Vec<CatalogFace>,
}

struct Block {
    file_offset: u64,
    compressed: u32,
    uncompressed: u32,
    lz4: bool,
}

/// A decoded face, top row first.
pub(crate) struct FaceImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

impl PortraitBundle {
    /// Reads the block table, the object table, the catalog and every texture's name. The
    /// names sit at the start of each texture, so this decodes the start of every block.
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let mut file = File::open(path).map_err(|_| BundleError("the bundle cannot be opened"))?;
        let mut bundle = read_archive(&mut file)?;
        let mut reader = BundleReader::new(&bundle, file);
        let objects = read_objects(&mut reader)?;
        let mut faces = read_catalog(&mut reader, &objects)?;
        let textures = read_texture_names(&mut reader, &objects)?;
        for (name, face) in &mut faces {
            face.texture = textures.get(name.as_slice()).copied();
        }
        // One texture stands for the format of all, so an unsupported bundle fails here once.
        if let Some((offset, len)) = faces.iter().find_map(|(_, face)| face.texture) {
            decode_texture(&reader.read(offset, len as usize)?)?;
        }
        bundle.faces = faces.into_iter().map(|(_, face)| face).collect();
        Ok(bundle)
    }

    pub(crate) fn faces(&self) -> &[CatalogFace] {
        &self.faces
    }

    /// Decodes the texture of catalog face `index`. `Ok(None)` when the catalog names a face
    /// the bundle has no texture for.
    pub(crate) fn read_face(&self, path: &Path, index: usize) -> Result<Option<FaceImage>> {
        let Some((offset, len)) = self.faces.get(index).and_then(|face| face.texture) else {
            return Ok(None);
        };
        let file = File::open(path).map_err(|_| BundleError("the bundle cannot be opened"))?;
        let object = BundleReader::new(self, file).read(offset, len as usize)?;
        decode_texture(&object).map(Some)
    }
}

/// Reads ranges of the portrait file, decoding one block at a time and keeping the last.
struct BundleReader<'a> {
    bundle: &'a PortraitBundle,
    file: File,
    loaded: Option<usize>,
    compressed: Vec<u8>,
    decoded: Vec<u8>,
}

impl<'a> BundleReader<'a> {
    fn new(bundle: &'a PortraitBundle, file: File) -> Self {
        Self {
            bundle,
            file,
            loaded: None,
            compressed: Vec::new(),
            decoded: Vec::new(),
        }
    }

    fn read(&mut self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let end = offset.checked_add(len as u64);
        if !end.is_some_and(|end| end <= self.bundle.file_len) {
            return fail("a range lies outside the portrait file");
        }
        let mut out = Vec::with_capacity(len);
        let mut position = self.bundle.file_start + offset;
        while out.len() < len {
            let index = self
                .bundle
                .block_starts
                .partition_point(|start| *start <= position)
                - 1;
            let within = (position - self.bundle.block_starts[index]) as usize;
            let block_len = self.bundle.blocks[index].uncompressed as usize;
            let take = (len - out.len()).min(block_len - within);
            self.load(index, within + take)?;
            out.extend_from_slice(&self.decoded[within..within + take]);
            position += take as u64;
        }
        Ok(out)
    }

    /// Makes at least the first `wanted` bytes of block `index` available in `decoded`.
    fn load(&mut self, index: usize, wanted: usize) -> Result<()> {
        if self.loaded == Some(index) && self.decoded.len() >= wanted {
            return Ok(());
        }
        let block = &self.bundle.blocks[index];
        if self.loaded != Some(index) {
            self.loaded = None;
            self.compressed.resize(block.compressed as usize, 0);
            self.file
                .seek(SeekFrom::Start(block.file_offset))
                .and_then(|_| self.file.read_exact(&mut self.compressed))
                .map_err(|_| BundleError("a block cannot be read"))?;
        }
        if block.lz4 {
            lz4::decode(
                &self.compressed,
                &mut self.decoded,
                wanted,
                block.uncompressed as usize,
            )
            .map_err(|()| BundleError("a block is not valid LZ4"))?;
        } else {
            self.decoded.clone_from(&self.compressed);
        }
        if self.decoded.len() < wanted {
            return fail("a block is shorter than declared");
        }
        self.loaded = Some(index);
        Ok(())
    }
}

/// A bounds-checked cursor over bytes already in memory.
struct Bytes<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Bytes<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(len)
            .filter(|end| *end <= self.data.len());
        let Some(end) = end else {
            return fail("the bundle ends inside a record");
        };
        let slice = &self.data[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.take(N)?.try_into().expect("take returns N bytes"))
    }

    fn skip(&mut self, len: usize) -> Result<()> {
        self.take(len).map(|_| ())
    }

    fn align4(&mut self) {
        self.at = (self.at + 3) & !3;
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.array::<1>()?[0])
    }

    fn i32_le(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn i64_le(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    fn u32_be(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn i64_be(&mut self) -> Result<i64> {
        Ok(i64::from_be_bytes(self.array()?))
    }

    /// A count or length that must lie in `0..=max`.
    fn count_le(&mut self, max: usize) -> Result<usize> {
        match usize::try_from(self.i32_le()?) {
            Ok(count) if count <= max => Ok(count),
            _ => fail("a count in the bundle is out of range"),
        }
    }

    fn c_string(&mut self) -> Result<&'a [u8]> {
        let rest = &self.data[self.at.min(self.data.len())..];
        let Some(len) = rest.iter().position(|byte| *byte == 0) else {
            return fail("a name in the bundle has no end");
        };
        self.at += len + 1;
        Ok(&rest[..len])
    }

    /// A length-prefixed string followed by padding to four bytes.
    fn string(&mut self, max: usize) -> Result<&'a [u8]> {
        let len = self.count_le(max)?;
        let bytes = self.take(len)?;
        self.align4();
        Ok(bytes)
    }
}

fn read_exact_at(file: &mut File, offset: u64, len: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0; len];
    file.seek(SeekFrom::Start(offset))
        .and_then(|_| file.read_exact(&mut bytes))
        .map_err(|_| BundleError("the bundle is shorter than its header says"))?;
    Ok(bytes)
}

/// Reads the UnityFS header and block table, and finds the serialized portrait file.
fn read_archive(file: &mut File) -> Result<PortraitBundle> {
    let file_len = file
        .metadata()
        .map_err(|_| BundleError("the bundle cannot be read"))?
        .len();
    let head = read_exact_at(file, 0, file_len.min(256) as usize)?;
    let mut header = Bytes::new(&head);
    if header.c_string()? != b"UnityFS" {
        return fail("the file is not a UnityFS bundle");
    }
    let version = header.u32_be()?;
    if !(6..=8).contains(&version) {
        return fail("the UnityFS version is not supported");
    }
    header.c_string()?;
    header.c_string()?;
    header.i64_be()?;
    let info_compressed = header.u32_be()? as usize;
    let info_len = header.u32_be()? as usize;
    let flags = header.u32_be()?;
    if info_compressed > MAX_BLOCKS_INFO_BYTES
        || info_len > MAX_BLOCKS_INFO_BYTES
        || info_compressed as u64 > file_len
    {
        return fail("the block table is too large");
    }
    let mut data_start = header.at as u64;
    if version >= 7 {
        data_start = (data_start + 15) & !15;
    }
    let info_at = if flags & 0x80 != 0 {
        file_len
            .checked_sub(info_compressed as u64)
            .ok_or(BundleError("the block table lies outside the bundle"))?
    } else {
        let at = data_start;
        data_start += info_compressed as u64;
        at
    };
    if flags & 0x200 != 0 {
        data_start = (data_start + 15) & !15;
    }
    let raw_info = read_exact_at(file, info_at, info_compressed)?;
    let info = match flags & 0x3F {
        0 => raw_info,
        2 | 3 => {
            let mut info = Vec::new();
            lz4::decode(&raw_info, &mut info, info_len, info_len)
                .map_err(|()| BundleError("the block table is not valid LZ4"))?;
            info
        }
        _ => return fail("the block table uses unsupported compression"),
    };

    let mut info = Bytes::new(&info);
    info.skip(16)?;
    let block_count = usize::try_from(info.u32_be()?).unwrap_or(usize::MAX);
    // Each block takes ten bytes of the table.
    if block_count > MAX_BLOCKS || block_count > info.data.len() / 10 {
        return fail("the bundle has too many blocks");
    }
    let mut blocks = Vec::with_capacity(block_count);
    let mut block_starts = Vec::with_capacity(block_count + 1);
    let (mut file_offset, mut stream_offset) = (data_start, 0u64);
    for _ in 0..block_count {
        let uncompressed = info.u32_be()?;
        let compressed = info.u32_be()?;
        let block_flags = u16::from_be_bytes(info.array()?);
        let lz4 = match block_flags & 0x3F {
            0 => false,
            2 | 3 => true,
            _ => return fail("a block uses unsupported compression"),
        };
        if uncompressed as usize > MAX_BLOCK_BYTES
            || compressed as usize > MAX_BLOCK_BYTES
            || uncompressed == 0
            || (!lz4 && compressed != uncompressed)
        {
            return fail("a block size is out of range");
        }
        block_starts.push(stream_offset);
        blocks.push(Block {
            file_offset,
            compressed,
            uncompressed,
            lz4,
        });
        file_offset += u64::from(compressed);
        stream_offset += u64::from(uncompressed);
    }
    block_starts.push(stream_offset);
    if file_offset > file_len {
        return fail("the blocks lie outside the bundle");
    }

    let node_count = usize::try_from(info.u32_be()?).unwrap_or(usize::MAX);
    if node_count > MAX_NODES {
        return fail("the bundle holds too many files");
    }
    for _ in 0..node_count {
        let start = info.i64_be()?;
        let len = info.i64_be()?;
        info.u32_be()?;
        let name = info.c_string()?;
        // The serialized file has no extension; `.resS` and `.resource` hold streamed data.
        if name.contains(&b'.') {
            continue;
        }
        let (Ok(file_start), Ok(file_len)) = (u64::try_from(start), u64::try_from(len)) else {
            return fail("a file in the bundle has a negative range");
        };
        if file_start
            .checked_add(file_len)
            .map_or(true, |end| end > stream_offset)
        {
            return fail("a file in the bundle lies outside its blocks");
        }
        return Ok(PortraitBundle {
            blocks,
            block_starts,
            file_start,
            file_len,
            faces: Vec::new(),
        });
    }
    fail("the bundle holds no serialized file")
}

/// The objects this reader needs, as ranges of the portrait file.
struct Objects {
    textures: Vec<(u64, u32)>,
    scripts: Vec<(u64, u32)>,
}

/// Reads the serialized file's type and object tables (format version 22, little-endian).
fn read_objects(reader: &mut BundleReader<'_>) -> Result<Objects> {
    let head = reader.read(0, 48)?;
    let mut header = Bytes::new(&head);
    header.skip(8)?;
    let version = header.u32_be()?;
    header.skip(4)?;
    let big_endian = header.u8()? != 0;
    header.skip(3)?;
    if version < 22 || big_endian {
        return fail("the serialized file version is not supported");
    }
    let metadata_len = header.u32_be()? as usize;
    header.i64_be()?;
    let data_offset = u64::try_from(header.i64_be()?)
        .map_err(|_| BundleError("the object data has a negative offset"))?;
    if metadata_len > MAX_METADATA_BYTES {
        return fail("the object table is too large");
    }

    let metadata = reader.read(48, metadata_len)?;
    let mut metadata = Bytes::new(&metadata);
    metadata.c_string()?;
    metadata.skip(4)?;
    let has_type_trees = metadata.u8()? != 0;
    let type_count = metadata.count_le(MAX_TYPES)?;
    let mut classes = Vec::with_capacity(type_count);
    for _ in 0..type_count {
        let class = metadata.i32_le()?;
        metadata.skip(3)?;
        metadata.skip(if class == MONO_BEHAVIOUR { 32 } else { 16 })?;
        if has_type_trees {
            let nodes = metadata.count_le(MAX_METADATA_BYTES / 32)?;
            let strings = metadata.count_le(MAX_METADATA_BYTES)?;
            metadata.skip(nodes * 32 + strings)?;
        }
        let dependencies = metadata.count_le(MAX_TYPES)?;
        metadata.skip(dependencies * 4)?;
        classes.push(class);
    }

    let object_count = metadata.count_le(MAX_OBJECTS)?;
    let mut objects = Objects {
        textures: Vec::new(),
        scripts: Vec::new(),
    };
    for _ in 0..object_count {
        metadata.align4();
        metadata.i64_le()?;
        let start = metadata.i64_le()?;
        let len = u32::from_le_bytes(metadata.array()?);
        let class = usize::try_from(metadata.i32_le()?)
            .ok()
            .and_then(|index| classes.get(index));
        let offset = u64::try_from(start)
            .ok()
            .and_then(|start| start.checked_add(data_offset));
        let (Some(class), Some(offset)) = (class, offset) else {
            return fail("an object in the bundle is malformed");
        };
        match *class {
            TEXTURE_2D => objects.textures.push((offset, len)),
            MONO_BEHAVIOUR => objects.scripts.push((offset, len)),
            _ => {}
        }
    }
    objects.textures.sort_unstable();
    Ok(objects)
}

/// Finds the `NewGenPortraitsCatalog` and returns each face's file name and properties.
fn read_catalog(
    reader: &mut BundleReader<'_>,
    objects: &Objects,
) -> Result<Vec<(Vec<u8>, CatalogFace)>> {
    for (offset, len) in &objects.scripts {
        if *len as usize > MAX_CATALOG_BYTES {
            continue;
        }
        let object = reader.read(*offset, *len as usize)?;
        let mut object = Bytes::new(&object);
        // m_GameObject, m_Enabled with padding, and m_Script.
        object.skip(12 + 4 + 12)?;
        if object.string(MAX_NAME_BYTES).ok() != Some(CATALOG_NAME) {
            continue;
        }
        let count = object.count_le(MAX_OBJECTS)?;
        let mut faces = Vec::with_capacity(count);
        for _ in 0..count {
            let name = object.string(MAX_NAME_BYTES)?.to_vec();
            if object.count_le(6)? != 6 {
                return fail("a catalog face does not have six properties");
            }
            let mut properties = [0; 6];
            for property in &mut properties {
                *property = u8::try_from(object.i32_le()?)
                    .map_err(|_| BundleError("a catalog property is out of range"))?;
            }
            faces.push((
                name,
                CatalogFace {
                    properties,
                    texture: None,
                },
            ));
        }
        return Ok(faces);
    }
    fail("the bundle has no portrait catalog")
}

/// Maps each texture's name, which starts its object, to the object's range.
fn read_texture_names(
    reader: &mut BundleReader<'_>,
    objects: &Objects,
) -> Result<HashMap<Vec<u8>, (u64, u32)>> {
    let mut names = HashMap::with_capacity(objects.textures.len());
    for (offset, len) in &objects.textures {
        let head = reader.read(*offset, (*len as usize).min(4 + MAX_NAME_BYTES))?;
        if let Ok(name) = Bytes::new(&head).string(MAX_NAME_BYTES) {
            names.insert(name.to_vec(), (*offset, *len));
        }
    }
    Ok(names)
}

/// Decodes a `Texture2D` object (Unity 6000.3 layout) whose image is inline DXT5.
fn decode_texture(object: &[u8]) -> Result<FaceImage> {
    if object.len() > MAX_TEXTURE_BYTES {
        return fail("a texture is too large");
    }
    let mut object = Bytes::new(object);
    object.string(MAX_NAME_BYTES)?;
    // m_IsAlphaChannelOptional with padding.
    object.skip(4)?;
    let width = object.count_le(MAX_SIDE)?;
    let height = object.count_le(MAX_SIDE)?;
    // m_CompleteImageSize and m_MipsStripped.
    object.skip(8)?;
    let format = object.i32_le()?;
    // m_MipCount, then m_IsReadable, m_IsPreProcessed and m_IgnoreMipmapLimit with padding.
    object.skip(8)?;
    object.string(256)?;
    // m_StreamingMipmaps with padding, m_StreamingMipmapsPriority, m_ImageCount,
    // m_TextureDimension, m_TextureSettings, m_LightmapFormat and m_ColorSpace.
    object.skip(4 + 4 + 4 + 4 + 24 + 4 + 4)?;
    let platform_blob = object.count_le(MAX_TEXTURE_BYTES)?;
    object.skip(platform_blob)?;
    object.align4();
    let image_len = object.count_le(MAX_TEXTURE_BYTES)?;
    let image = object.take(image_len)?;
    if format != DXT5 {
        return fail("a texture is not DXT5");
    }
    let rgba = dxt::decode_dxt5_bottom_up(image, width, height)
        .ok_or(BundleError("a texture's image data does not fit its size"))?;
    Ok(FaceImage {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The colour of a test texture: every pixel of a 4x4 DXT5 image, opaque.
    pub(crate) fn solid_block(red: bool) -> [u8; 16] {
        let colour: u16 = if red { 0xF800 } else { 0x001F };
        let mut block = [0; 16];
        block[0] = 255;
        block[1] = 255;
        block[8..10].copy_from_slice(&colour.to_le_bytes());
        block[10..12].copy_from_slice(&colour.to_le_bytes());
        block
    }

    fn string(out: &mut Vec<u8>, text: &[u8]) {
        out.extend((text.len() as i32).to_le_bytes());
        out.extend(text);
        out.resize((out.len() + 3) & !3, 0);
    }

    fn texture(name: &[u8], format: i32, image: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        string(&mut out, name);
        out.extend([0; 4]);
        out.extend(4i32.to_le_bytes());
        out.extend(4i32.to_le_bytes());
        out.extend([0; 8]);
        out.extend(format.to_le_bytes());
        out.extend([0; 8]);
        string(&mut out, b"MG_LicenseData");
        out.extend([0; 48]);
        out.extend(0i32.to_le_bytes());
        out.extend((image.len() as i32).to_le_bytes());
        out.extend(image);
        out.resize((out.len() + 3) & !3, 0);
        out.extend([0; 16]);
        out
    }

    fn catalog(faces: &[(&[u8], [i32; 6])]) -> Vec<u8> {
        let mut out = vec![0; 28];
        string(&mut out, CATALOG_NAME);
        out.extend((faces.len() as i32).to_le_bytes());
        for (name, properties) in faces {
            string(&mut out, name);
            out.extend(6i32.to_le_bytes());
            for property in properties {
                out.extend(property.to_le_bytes());
            }
        }
        out
    }

    /// A serialized file (version 22) holding `objects` as (class, bytes), with type trees.
    fn serialized_file(objects: &[(i32, Vec<u8>)]) -> Vec<u8> {
        let mut classes: Vec<i32> = objects.iter().map(|(class, _)| *class).collect();
        classes.dedup();
        let mut metadata = b"6000.3.18f1\0".to_vec();
        metadata.extend(5i32.to_le_bytes());
        metadata.push(1);
        metadata.extend((classes.len() as i32).to_le_bytes());
        for class in &classes {
            metadata.extend(class.to_le_bytes());
            metadata.extend([0; 3]);
            metadata.extend(vec![0; if *class == MONO_BEHAVIOUR { 32 } else { 16 }]);
            // One type tree node and a two-byte string buffer.
            metadata.extend(1i32.to_le_bytes());
            metadata.extend(2i32.to_le_bytes());
            metadata.extend([0; 34]);
            metadata.extend(0i32.to_le_bytes());
        }
        metadata.extend((objects.len() as i32).to_le_bytes());
        let mut data = Vec::new();
        for (index, (class, bytes)) in objects.iter().enumerate() {
            metadata.resize((metadata.len() + 3) & !3, 0);
            metadata.extend((index as i64 + 1).to_le_bytes());
            metadata.extend((data.len() as i64).to_le_bytes());
            metadata.extend((bytes.len() as u32).to_le_bytes());
            let class_index = classes.iter().position(|known| known == class).unwrap();
            metadata.extend((class_index as i32).to_le_bytes());
            data.extend(bytes);
            data.resize((data.len() + 7) & !7, 0);
        }
        let data_offset = (48 + metadata.len() + 15) & !15;
        let mut out = vec![0; 8];
        out.extend(22u32.to_be_bytes());
        out.extend([0; 8]);
        out.extend((metadata.len() as u32).to_be_bytes());
        out.extend(((data_offset + data.len()) as i64).to_be_bytes());
        out.extend((data_offset as i64).to_be_bytes());
        out.extend([0; 8]);
        out.extend(&metadata);
        out.resize(data_offset, 0);
        out.extend(data);
        out
    }

    /// An LZ4 block of literals only.
    fn lz4_literals(data: &[u8]) -> Vec<u8> {
        let mut out = vec![0xF0];
        let mut rest = data.len() - 15;
        while rest >= 255 {
            out.push(255);
            rest -= 255;
        }
        out.push(rest as u8);
        out.extend(data);
        out
    }

    /// A UnityFS bundle of 64-byte LZ4 blocks with the block table at the end.
    pub(crate) fn unityfs(serialized: &[u8]) -> Vec<u8> {
        let mut out = b"UnityFS\0".to_vec();
        out.extend(8u32.to_be_bytes());
        out.extend(b"5.x.x\0");
        out.extend(b"6000.3.18f1\0");
        let chunks: Vec<&[u8]> = serialized.chunks(64).collect();
        let blocks: Vec<Vec<u8>> = chunks
            .iter()
            .map(|chunk| {
                if chunk.len() > 15 {
                    lz4_literals(chunk)
                } else {
                    chunk.to_vec()
                }
            })
            .collect();
        let mut info = vec![0; 16];
        info.extend((blocks.len() as u32).to_be_bytes());
        for (chunk, block) in chunks.iter().zip(&blocks) {
            info.extend((chunk.len() as u32).to_be_bytes());
            info.extend((block.len() as u32).to_be_bytes());
            info.extend((if chunk.len() > 15 { 2u16 } else { 0 }).to_be_bytes());
        }
        info.extend(2u32.to_be_bytes());
        info.extend(0i64.to_be_bytes());
        info.extend((serialized.len() as i64).to_be_bytes());
        info.extend(4u32.to_be_bytes());
        info.extend(b"CAB-portraits\0");
        info.extend((serialized.len() as i64).to_be_bytes());
        info.extend(0i64.to_be_bytes());
        info.extend(0u32.to_be_bytes());
        info.extend(b"CAB-portraits.resS\0");
        out.extend(0i64.to_be_bytes());
        out.extend((info.len() as u32).to_be_bytes());
        out.extend((info.len() as u32).to_be_bytes());
        out.extend(0x80u32.to_be_bytes());
        out.resize((out.len() + 15) & !15, 0);
        for block in &blocks {
            out.extend(block);
        }
        out.extend(info);
        out
    }

    /// A bundle with a red and a blue face, a catalog face without a texture, and a texture
    /// the catalog does not list.
    pub(crate) fn sample_bundle() -> Vec<u8> {
        unityfs(&serialized_file(&[
            (
                TEXTURE_2D,
                texture(b"portrait_male-0-1-2-3-0", DXT5, &solid_block(true)),
            ),
            (115, vec![0; 20]),
            (
                TEXTURE_2D,
                texture(b"portrait_female-0-1-4-2-1", DXT5, &solid_block(false)),
            ),
            (TEXTURE_2D, texture(b"unlisted", DXT5, &solid_block(true))),
            (
                MONO_BEHAVIOUR,
                catalog(&[
                    (b"portrait_female-0-1-4-2-1", [1, 0, 1, 4, 2, 1]),
                    (b"portrait_male-0-1-2-3-0", [0, 0, 1, 2, 3, 0]),
                    (b"portrait_male-9-9-9-9-9", [0, 9, 9, 9, 9, 9]),
                ]),
            ),
        ]))
    }

    pub(crate) fn write(bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("portraits_assets_all.bundle");
        std::fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    #[test]
    fn the_catalog_keeps_its_order_and_finds_each_texture() {
        let (_dir, path) = write(&sample_bundle());

        let bundle = PortraitBundle::open(&path).unwrap();

        let properties: Vec<[u8; 6]> = bundle.faces().iter().map(|face| face.properties).collect();
        assert_eq!(
            properties,
            [[1, 0, 1, 4, 2, 1], [0, 0, 1, 2, 3, 0], [0, 9, 9, 9, 9, 9]]
        );
        let blue = bundle.read_face(&path, 0).unwrap().unwrap();
        assert_eq!((blue.width, blue.height), (4, 4));
        assert_eq!(&blue.rgba[..4], &[0, 0, 255, 255]);
        let red = bundle.read_face(&path, 1).unwrap().unwrap();
        assert_eq!(&red.rgba[60..], &[255, 0, 0, 255]);
        assert!(bundle.read_face(&path, 2).unwrap().is_none());
        assert!(bundle.read_face(&path, 3).unwrap().is_none());
    }

    #[test]
    fn a_bundle_whose_textures_are_in_another_format_is_an_error() {
        let bytes = unityfs(&serialized_file(&[
            (
                TEXTURE_2D,
                texture(b"portrait_male-0-1-2-3-0", 25, &solid_block(true)),
            ),
            (
                MONO_BEHAVIOUR,
                catalog(&[(b"portrait_male-0-1-2-3-0", [0, 0, 1, 2, 3, 0])]),
            ),
        ]));
        let (_dir, path) = write(&bytes);

        assert_eq!(
            PortraitBundle::open(&path).err(),
            Some(BundleError("a texture is not DXT5"))
        );
    }

    #[test]
    fn a_bundle_without_a_catalog_is_an_error() {
        let bytes = unityfs(&serialized_file(&[(
            TEXTURE_2D,
            texture(b"portrait_male-0-1-2-3-0", DXT5, &solid_block(true)),
        )]));
        let (_dir, path) = write(&bytes);

        assert_eq!(
            PortraitBundle::open(&path).err(),
            Some(BundleError("the bundle has no portrait catalog"))
        );
    }

    #[test]
    fn a_truncated_or_foreign_file_is_an_error_at_every_length() {
        let bytes = sample_bundle();
        for len in (0..bytes.len()).step_by(7) {
            let (_dir, path) = write(&bytes[..len]);
            assert!(PortraitBundle::open(&path).is_err(), "length {len}");
        }
        let (_dir, path) = write(b"not a bundle at all, just some text that is long enough");
        assert!(PortraitBundle::open(&path).is_err());
    }

    #[test]
    fn corrupted_bytes_never_panic() {
        let bytes = sample_bundle();
        for at in 0..bytes.len() {
            let mut corrupted = bytes.clone();
            corrupted[at] ^= 0xFF;
            let (_dir, path) = write(&corrupted);
            if let Ok(bundle) = PortraitBundle::open(&path) {
                for index in 0..bundle.faces().len() {
                    let _ = bundle.read_face(&path, index);
                }
            }
        }
    }
}
