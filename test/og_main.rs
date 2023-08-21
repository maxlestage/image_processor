use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{Seek, SeekFrom};
use std::time::Instant;

// Taille constante de l'en-tête BMP
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

// Implémentation de la sérialisation de l'en-tête au format octet
impl BmpHeader {
    // Méthode pour sérialiser la structure
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![0; BMP_HEADER_SIZE];

        // Ecrire la signature exemple "BM"
        let magic_number_as_utf8: String = String::from(self.magic_number.clone());
        bytes[0..2].copy_from_slice(magic_number_as_utf8.as_bytes());

        // Ecrire la taille de l'image
        let image_size_bytes: [u8; 4] = self.image_size.to_le_bytes();
        bytes[2..6].copy_from_slice(&image_size_bytes);

        let reserved_bytes: [u8; 4] = self.reserved;
        bytes[6..10].copy_from_slice(&reserved_bytes);

        let data_offset_bytes: [u8; 4] = self.data_offset.to_le_bytes();
        bytes[10..14].copy_from_slice(&data_offset_bytes);

        let header_size_bytes: [u8; 4] = self.header_size.to_le_bytes();
        bytes[14..18].copy_from_slice(&header_size_bytes);

        let width_bytes: [u8; 4] = self.width.to_le_bytes();
        bytes[18..22].copy_from_slice(&width_bytes);

        let height_bytes: [u8; 4] = self.height.to_le_bytes();
        bytes[22..26].copy_from_slice(&height_bytes);

        let planes_bytes: [u8; 2] = self.planes.to_le_bytes();
        bytes[26..28].copy_from_slice(&planes_bytes);

        let bpp_bytes: [u8; 2] = self.bpp.to_le_bytes();
        bytes[28..30].copy_from_slice(&bpp_bytes);

        // Récupérer la valeur numérique de l'enum Compression
        let compressed_num: u32 = match self.compression {
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
            Compression::Unknown(num) => num,
        };

        let compressed_bytes: [u8; 4] = compressed_num.to_le_bytes();
        bytes[30..34].copy_from_slice(&compressed_bytes);

        let compressed_size_bytes: [u8; 4] = self.compressed_size.to_le_bytes();
        bytes[34..38].copy_from_slice(&compressed_size_bytes);

        let x_pixels_per_meter_bytes: [u8; 4] = self.x_pixels_per_meter.to_le_bytes();
        bytes[38..42].copy_from_slice(&x_pixels_per_meter_bytes);

        let x_pixels_per_meter_bytes: [u8; 4] = self.x_pixels_per_meter.to_le_bytes();
        bytes[38..42].copy_from_slice(&x_pixels_per_meter_bytes);

        let y_pixels_per_meter_bytes: [u8; 4] = self.y_pixels_per_meter.to_le_bytes();
        bytes[42..46].copy_from_slice(&y_pixels_per_meter_bytes);

        let colors_used_bytes: [u8; 4] = self.colors_used.to_le_bytes();
        bytes[46..50].copy_from_slice(&colors_used_bytes);

        let important_colors_bytes: [u8; 4] = self.important_colors.to_le_bytes();
        bytes[50..54].copy_from_slice(&important_colors_bytes);

        // return bytes;
        bytes
    }
}

// Read and parse BMP header
fn read_bmp_header(mut file: &File) -> io::Result<BmpHeader> {
    // Crée un tampon de 54 octets pour stocker l'en-tête BMP
    let mut buf: [u8; 54] = [0; 54];

    // Lit exactement 54 octets depuis le fichier et les place dans le tampon
    file.read_exact(&mut buf)?;

    // Convertit les 2 premiers octets du tampon en une chaîne de caractères UTF-8
    let magic: &str = std::str::from_utf8(&buf[0..2]).unwrap();

    // // Vérifie si la chaîne magique est différente de "BM"
    // if magic != "BM" {
    //     // Si la chaîne magique est différente de "BM", renvoie une erreur avec un message d'erreur
    //     return Err(io::Error::new(
    //         io::ErrorKind::InvalidData,
    //         "Invalid magic string",
    //     ));
    // }

    // Convertit les octets 30 à 33 du tampon en un objet Compression
    // Convert bytes 30-33 to a Compression value
    let compression: Compression = Compression::from(Compression::Unknown(u32::from_le_bytes([
        buf[30], buf[31], buf[32], buf[33],
    ])));

    // Retourne une instance de BmpHeader avec les valeurs extraites de l'en-tête
    Ok(BmpHeader {
        // Convertit la chaîne magique en un tableau fixe de 2 octets
        magic_number: magic.try_into().unwrap(),

        // Convertit les octets 2 à 5 du tampon en un entier non signé 32 bits en little-endian
        image_size: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[2..6]).unwrap()),

        // Convertit les octets 6 à 9 du tampon en un tableau fixe de 4 octets
        reserved: <[u8; 4]>::try_from(&buf[6..10]).unwrap(),

        // Convertit les octets 10 à 13 du tampon en un entier non signé 32 bits en little-endian
        data_offset: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[10..14]).unwrap()),

        // Convertit les octets 14 à 17 du tampon en un entier non signé 32 bits en little-endian
        header_size: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[14..18]).unwrap()),

        // Convertit les octets 18 à 21 du tampon en un entier non signé 32 bits en little-endian
        width: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[18..22]).unwrap()),

        // Convertit les octets 22 à 25 du tampon en un entier non signé 32 bits en little-endian
        height: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[22..26]).unwrap()),

        // Convertit les octets 26 à 27 du tampon en un entier non signé 16 bits en little-endian
        planes: u16::from_le_bytes(<[u8; 2]>::try_from(&buf[26..28]).unwrap()),

        // Convertit les octets 28 à 29 du tampon en un entier non signé 16 bits en little-endian
        bpp: u16::from_le_bytes(<[u8; 2]>::try_from(&buf[28..30]).unwrap()),

        // Affecte la valeur de compression extraite précédemment
        compression,

        // Convertit les octets 34 à 37 du tampon en un entier non signé 32 bits en little-endian
        compressed_size: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[34..38]).unwrap()),

        // Convertit les octets 38 à 41 du tampon en un entier non signé 32 bits en little-endian
        x_pixels_per_meter: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[38..42]).unwrap()),

        // Convertit les octets 42 à 45 du tampon en un entier non signé 32 bits en little-endian
        y_pixels_per_meter: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[42..46]).unwrap()),

        // Convertit les octets46 à 49 du tampon en un entier non signé 32 bits en little-endian
        colors_used: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[46..50]).unwrap()),

        // Convertit les octets 50 à 53 du tampon en un entier non signé 32 bits en little-endian
        important_colors: u32::from_le_bytes(<[u8; 4]>::try_from(&buf[50..54]).unwrap()),
    })
}

fn main() -> io::Result<()> {
    let mut file: File = File::open("./assets/Elsa.bmp")?;

    let grayscale_result: Result<(), io::Error> = generate_grayscale(&mut file);
    grayscale_result?;

    Ok(())
}

fn generate_grayscale(file: &mut File) -> io::Result<()> {
    let start: Instant = Instant::now();
    // Lire l'en-tête BMP
    let header: BmpHeader = read_bmp_header(file)?;
    println!("Header: {:#?}", header);

    // Se positionner au début des données pixels
    file.seek(SeekFrom::Start(header.data_offset as u64))?;

    // Buffer pour contenir les pixels
    let mut pixels: Vec<u8> = vec![0; header.image_size as usize];

    // Lire les pixels dans le buffer
    let bytes_read: usize = file.read(&mut pixels)?;

    // Vérifier si tous les pixels ont bien été lus
    if bytes_read != pixels.len() {
        // Repositionner au cas où la lecture soit incomplète
        file.seek(SeekFrom::Start(header.data_offset as u64))?;
    }

    // Convertir les pixels en niveaux de gris en parallèle
    // pour accélérer le traitement sur plusieurs coeurs CPU.

    // Découper les pixels en chunks mutables de 4096 pixels.
    pixels.par_chunks_mut(4095).for_each(|chunk: &mut [u8]| {
        // Traiter chaque chunk en parallèle sur différents threads.
        convert_to_grayscale(chunk);
    });

    // Les chunks sont traités en parallèle et une fois terminés,
    // tous les pixels auront été convertis en niveaux de gris.

    // Cela permet d'exploiter plusieurs coeurs CPU pour accélérer
    // le traitement de façon performante plutôt que pixel par pixel.

    // Revenir au début du fichier
    file.seek(SeekFrom::Start(0))?;

    // Ouvrir le fichier de sortie
    let mut output: File = File::create("./assets/test.bmp")?;

    // Ecrire l'en-tête
    output.write_all(&header.as_bytes())?;

    // Ecrire les pixels convertis
    output.write_all(&pixels)?;

    // La méthode flush() sur les flux (streams) en Rust permet de forcer l'écriture des données buffered en mémoire vers le support physique (disque, réseau, etc).
    output.flush()?;

    let duration: std::time::Duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}

fn convert_to_grayscale(pixels: &mut [u8]) {
    //Parcourir les pixels 3 par 3 (RGB)
    for i in 0..pixels.len() / 3 {
        // i va de 0 au nombre total de pixels
        // avec un pas de 3 pour parcourir les triplets RVB
        // i*3 -> index du R
        // i*3+1 -> index du G
        // i*3+2 -> index du B

        // Extraire les composantes R, G, B
        let r: u32 = pixels[i * 3] as u32;
        let g: u32 = pixels[i * 3 + 1] as u32;
        let b: u32 = pixels[i * 3 + 2] as u32;

        // Niveaux de gris = (R + G + B) / 3
        let grayscale_level: u32 = 3;
        let grayscale: u32 = (r + g + b) / grayscale_level;

        // Ecrire la moyenne dans chaque composante
        pixels[i * 3] = grayscale as u8;
        pixels[i * 3 + 1] = grayscale as u8;
        pixels[i * 3 + 2] = grayscale as u8;
    }
}
