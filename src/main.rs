use rayon::prelude::*;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::time::Instant;

// Taille constante de l'en-tête BMP (14 octets de fichier + 40 de DIB).
const BMP_HEADER_SIZE: usize = 14 + 40;

// Bitmap header struct
#[allow(dead_code)]
#[derive(Debug)]
struct BmpHeader {
    // File signature, always "BM"
    magic_number: String,

    // Size of the image in bytes
    image_size: u32,

    // Reserved, should be 0
    reserved: [u8; 4],

    // Offset to start of pixel data
    data_offset: u32,

    // Size of header in bytes
    header_size: u32,

    // Image dimensions
    width: u32,
    height: u32,

    // Number of color planes, usually 1
    planes: u16,

    // Bits per pixel
    bpp: u16,

    // Compression method
    compression: Compression,

    // Image size again (?)
    compressed_size: u32,

    // Resolution
    x_pixels_per_meter: u32,
    y_pixels_per_meter: u32,

    // Number of colors in palette
    colors_used: u32,

    // Important colors
    important_colors: u32,
}

// Bitmap compression types
#[allow(dead_code)]
#[derive(Debug)]
enum Compression {
    RGB,
    RLE8,
    RLE4,
    Bitfields,
    JPEG,
    PNG,
    AlphaBits,
    CMYK,
    CMYKRBG,
    BitfieldsTwo,
    Unknown(u32),
}

impl Compression {
    // Convertit la valeur brute (octets 30..34) en variante d'enum.
    fn from_u32(value: u32) -> Compression {
        match value {
            0 => Compression::RGB,
            1 => Compression::RLE8,
            2 => Compression::RLE4,
            3 => Compression::Bitfields,
            4 => Compression::JPEG,
            5 => Compression::PNG,
            6 => Compression::AlphaBits,
            7 => Compression::CMYK,
            8 => Compression::CMYKRBG,
            9 => Compression::BitfieldsTwo,
            other => Compression::Unknown(other),
        }
    }

    fn as_u32(&self) -> u32 {
        match self {
            Compression::RGB => 0,
            Compression::RLE8 => 1,
            Compression::RLE4 => 2,
            Compression::Bitfields => 3,
            Compression::JPEG => 4,
            Compression::PNG => 5,
            Compression::AlphaBits => 6,
            Compression::CMYK => 7,
            Compression::CMYKRBG => 8,
            Compression::BitfieldsTwo => 9,
            Compression::Unknown(num) => *num,
        }
    }
}

impl BmpHeader {
    // Sérialise l'en-tête sur 54 octets au format BMP (little-endian).
    fn as_bytes(&self) -> [u8; BMP_HEADER_SIZE] {
        let mut bytes = [0u8; BMP_HEADER_SIZE];

        // Signature, p.ex. "BM" (tronquée/complétée sur 2 octets).
        let magic = self.magic_number.as_bytes();
        let magic_len = magic.len().min(2);
        bytes[0..magic_len].copy_from_slice(&magic[..magic_len]);

        bytes[2..6].copy_from_slice(&self.image_size.to_le_bytes());
        bytes[6..10].copy_from_slice(&self.reserved);
        bytes[10..14].copy_from_slice(&self.data_offset.to_le_bytes());
        bytes[14..18].copy_from_slice(&self.header_size.to_le_bytes());
        bytes[18..22].copy_from_slice(&self.width.to_le_bytes());
        bytes[22..26].copy_from_slice(&self.height.to_le_bytes());
        bytes[26..28].copy_from_slice(&self.planes.to_le_bytes());
        bytes[28..30].copy_from_slice(&self.bpp.to_le_bytes());
        bytes[30..34].copy_from_slice(&self.compression.as_u32().to_le_bytes());
        bytes[34..38].copy_from_slice(&self.compressed_size.to_le_bytes());
        bytes[38..42].copy_from_slice(&self.x_pixels_per_meter.to_le_bytes());
        bytes[42..46].copy_from_slice(&self.y_pixels_per_meter.to_le_bytes());
        bytes[46..50].copy_from_slice(&self.colors_used.to_le_bytes());
        bytes[50..54].copy_from_slice(&self.important_colors.to_le_bytes());

        bytes
    }
}

// Petit utilitaire de lecture little-endian sur une tranche d'octets.
fn read_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}

fn read_u16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

// Analyse l'en-tête BMP à partir des 54 premiers octets du fichier.
fn parse_bmp_header(buf: &[u8]) -> io::Result<BmpHeader> {
    if buf.len() < BMP_HEADER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "fichier trop court pour contenir un en-tête BMP",
        ));
    }

    let magic = std::str::from_utf8(&buf[0..2])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "signature BMP invalide"))?;

    Ok(BmpHeader {
        magic_number: magic.to_string(),
        image_size: read_u32(buf, 2),
        reserved: <[u8; 4]>::try_from(&buf[6..10]).unwrap(),
        data_offset: read_u32(buf, 10),
        header_size: read_u32(buf, 14),
        width: read_u32(buf, 18),
        height: read_u32(buf, 22),
        planes: read_u16(buf, 26),
        bpp: read_u16(buf, 28),
        compression: Compression::from_u32(read_u32(buf, 30)),
        compressed_size: read_u32(buf, 34),
        x_pixels_per_meter: read_u32(buf, 38),
        y_pixels_per_meter: read_u32(buf, 42),
        colors_used: read_u32(buf, 46),
        important_colors: read_u32(buf, 50),
    })
}

fn main() -> io::Result<()> {
    convert_bmp_to_grayscale("./assets/Elsa.bmp", "./assets/test.bmp")
}

fn convert_bmp_to_grayscale(input_path: &str, output_path: &str) -> io::Result<()> {
    let start = Instant::now();

    // Lire tout le fichier en mémoire d'un coup : évite les lectures
    // partielles et la dépendance au champ `image_size` (souvent 0 en BI_RGB).
    let mut data = fs::read(input_path)?;

    let header = parse_bmp_header(&data)?;
    println!("Header: {:#?}", header);

    if header.bpp != 24 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "seules les images BMP 24 bits sont supportées (bpp = {})",
                header.bpp
            ),
        ));
    }

    let offset = header.data_offset as usize;
    let width = header.width as usize;
    // La hauteur peut être négative (BMP top-down) : on prend la valeur absolue.
    let height = (header.height as i32).unsigned_abs() as usize;

    // Dans un BMP, chaque ligne de pixels est complétée ("padding") pour
    // que sa taille en octets soit un multiple de 4. `row_bytes` est la
    // partie utile (3 octets/pixel), `stride` la ligne complète alignée.
    let row_bytes = width * 3;
    let stride = (row_bytes + 3) & !3;

    let pixels_len = stride
        .checked_mul(height)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "dimensions invalides"))?;
    let pixels_end = offset
        .checked_add(pixels_len)
        .filter(|&end| end <= data.len())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "données pixels tronquées ou data_offset invalide",
            )
        })?;

    // Conversion en niveaux de gris parallélisée avec rayon.
    //
    // `par_chunks_mut(stride)` confie une ligne entière à chaque tâche :
    // rayon répartit les lignes sur tous les cœurs CPU. À l'intérieur
    // d'une ligne, on n'itère que sur `row_bytes` (triplets B, G, R
    // exactement alignés) afin d'ignorer les octets de padding.
    data[offset..pixels_end]
        .par_chunks_mut(stride)
        .for_each(|row| {
            for pixel in row[..row_bytes].chunks_exact_mut(3) {
                let sum = pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32;
                let gray = (sum / 3) as u8;
                pixel[0] = gray;
                pixel[1] = gray;
                pixel[2] = gray;
            }
        });

    // Écrire l'en-tête régénéré puis les pixels convertis.
    let mut output = BufWriter::new(fs::File::create(output_path)?);
    output.write_all(&header.as_bytes())?;
    output.write_all(&data[offset..])?;
    output.flush()?;

    println!("Time elapsed: {:?}", start.elapsed());
    Ok(())
}
