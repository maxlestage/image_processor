use image::{GrayImage, ImageFormat};
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

// Extensions de fichiers RAW constructeurs courantes (en minuscules).
// Le décodage RAW passe par `imagepipe`/`rawloader` (pur Rust).
const RAW_EXTENSIONS: &[&str] = &[
    "cr2", "cr3", "crw", // Canon
    "nef", "nrw", // Nikon
    "arw", "srf", "sr2", // Sony
    "raf",  // Fujifilm
    "orf",  // Olympus / OM System
    "rw2",  // Panasonic
    "pef",  // Pentax
    "dng",  // Adobe / DNG générique
    "rwl",  // Leica
    "dcr", "kdc", // Kodak
    "mrw", // Minolta
    "3fr", "fff", // Hasselblad
    "mef", // Mamiya
    "mos", // Leaf
    "erf", // Epson
    "srw", // Samsung
    "x3f", // Sigma (Foveon)
    "iiq", // Phase One
];

// Image décodée, pixels RGB entrelacés (3 octets/pixel).
struct Rgb8 {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (input, output) = match (args.next(), args.next()) {
        (Some(input), Some(output)) => (input, output),
        _ => {
            eprintln!("Usage : image_processor <entrée> <sortie>");
            eprintln!();
            eprintln!("Convertit une image en niveaux de gris. Formats d'entrée :");
            eprintln!("  - raster : PNG, JPEG, GIF, WebP, TIFF, BMP, ICO, etc.");
            eprintln!("  - RAW    : Canon CR2/CR3, Nikon NEF, Sony ARW, Fuji RAF,");
            eprintln!("             Adobe DNG, Panasonic RW2, Olympus ORF, etc.");
            eprintln!("Le format de sortie est déduit de l'extension (PNG par défaut).");
            return ExitCode::FAILURE;
        }
    };

    match convert_to_grayscale(&input, &output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Erreur : {err}");
            ExitCode::FAILURE
        }
    }
}

fn convert_to_grayscale(input: &str, output: &str) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let Rgb8 {
        width,
        height,
        data,
    } = decode_any(input)?;
    println!("Image décodée : {width}x{height}");

    // Conversion en niveaux de gris parallélisée avec rayon : chaque
    // triplet RGB est réduit à un octet de luminance (moyenne R+G+B).
    // `par_chunks_exact(3)` + `collect` préservent l'ordre des pixels.
    let gray: Vec<u8> = data
        .par_chunks_exact(3)
        .map(|px| ((px[0] as u32 + px[1] as u32 + px[2] as u32) / 3) as u8)
        .collect();

    let img: GrayImage = GrayImage::from_raw(width, height, gray)
        .ok_or("buffer de pixels incohérent avec les dimensions")?;

    // Format de sortie déduit de l'extension ; PNG par défaut si inconnue.
    let format = ImageFormat::from_path(output).unwrap_or(ImageFormat::Png);
    img.save_with_format(output, format)?;

    println!(
        "Écrit « {output} » ({format:?}) en niveaux de gris — {:?}",
        start.elapsed()
    );
    Ok(())
}

// Décode n'importe quel format en RGB8 : `image` pour les formats raster
// classiques, `imagepipe` (pur Rust) pour les RAW constructeurs.
fn decode_any(path: &str) -> Result<Rgb8, Box<dyn Error>> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    if RAW_EXTENSIONS.contains(&ext.as_str()) {
        return decode_raw(path);
    }

    // Détection par contenu (magic bytes) en plus de l'extension ; si
    // `image` échoue, on tente le RAW en dernier recours (extension absente
    // ou non listée mais fichier RAW malgré tout).
    match decode_with_image(path) {
        Ok(rgb) => Ok(rgb),
        Err(image_err) => decode_raw(path).map_err(|raw_err| {
            format!("décodage impossible (image : {image_err} ; raw : {raw_err})").into()
        }),
    }
}

fn decode_with_image(path: &str) -> Result<Rgb8, Box<dyn Error>> {
    let reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let rgb = reader.decode()?.to_rgb8();
    let (width, height) = rgb.dimensions();
    Ok(Rgb8 {
        width,
        height,
        data: rgb.into_raw(),
    })
}

fn decode_raw(path: &str) -> Result<Rgb8, Box<dyn Error>> {
    let decoded = imagepipe::simple_decode_8bit(path, 0, 0)
        .map_err(|e| format!("RAW non décodable : {e}"))?;
    Ok(Rgb8 {
        width: decoded.width as u32,
        height: decoded.height as u32,
        data: decoded.data,
    })
}
