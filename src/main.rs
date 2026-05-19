use image::{GrayImage, ImageFormat, RgbImage};
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

// Opération appliquée à l'image.
#[derive(Clone, Copy)]
enum Operation {
    // Conversion en niveaux de gris (moyenne R+G+B).
    Grayscale,
    // Effet miroir horizontal (gauche/droite), couleurs conservées.
    Mirror,
}

impl Operation {
    fn parse(s: &str) -> Option<Operation> {
        match s.to_ascii_lowercase().as_str() {
            "grayscale" | "gray" | "gris" | "g" => Some(Operation::Grayscale),
            "mirror" | "miroir" | "m" => Some(Operation::Mirror),
            _ => None,
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (input, output) = match (args.next(), args.next()) {
        (Some(input), Some(output)) => (input, output),
        _ => {
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    // 3e argument optionnel : l'opération (défaut = niveaux de gris).
    let operation = match args.next() {
        None => Operation::Grayscale,
        Some(mode) => match Operation::parse(&mode) {
            Some(op) => op,
            None => {
                eprintln!("Mode inconnu : « {mode} » (attendu : grayscale | mirror)");
                print_usage();
                return ExitCode::FAILURE;
            }
        },
    };

    match process(&input, &output, operation) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Erreur : {err}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("Usage : image_processor <entrée> <sortie> [mode]");
    eprintln!();
    eprintln!("Modes :");
    eprintln!("  grayscale  (défaut)  conversion en niveaux de gris");
    eprintln!("  mirror               effet miroir horizontal (couleurs conservées)");
    eprintln!();
    eprintln!("Formats d'entrée :");
    eprintln!("  - raster : PNG, JPEG, GIF, WebP, TIFF, BMP, ICO, etc.");
    eprintln!("  - RAW    : Canon CR2/CR3, Nikon NEF, Sony ARW, Fuji RAF,");
    eprintln!("             Adobe DNG, Panasonic RW2, Olympus ORF, etc.");
    eprintln!("Le format de sortie est déduit de l'extension (PNG par défaut).");
}

fn process(input: &str, output: &str, operation: Operation) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let image = decode_any(input)?;
    println!("Image décodée : {}x{}", image.width, image.height);

    // Format de sortie déduit de l'extension ; PNG par défaut si inconnue.
    let format = ImageFormat::from_path(output).unwrap_or(ImageFormat::Png);

    let label = match operation {
        Operation::Grayscale => {
            to_grayscale(&image)?.save_with_format(output, format)?;
            "niveaux de gris"
        }
        Operation::Mirror => {
            mirror_horizontal(image)?.save_with_format(output, format)?;
            "miroir"
        }
    };

    println!(
        "Écrit « {output} » ({format:?}) — {label} — {:?}",
        start.elapsed()
    );
    Ok(())
}

// Conversion en niveaux de gris parallélisée avec rayon : chaque triplet
// RGB est réduit à un octet de luminance (moyenne R+G+B).
// `par_chunks_exact(3)` + `collect` préservent l'ordre des pixels.
fn to_grayscale(image: &Rgb8) -> Result<GrayImage, Box<dyn Error>> {
    let gray: Vec<u8> = image
        .data
        .par_chunks_exact(3)
        .map(|px| ((px[0] as u32 + px[1] as u32 + px[2] as u32) / 3) as u8)
        .collect();

    GrayImage::from_raw(image.width, image.height, gray)
        .ok_or_else(|| "buffer de pixels incohérent avec les dimensions".into())
}

// Effet miroir horizontal parallélisé avec rayon : chaque ligne est
// confiée à une tâche (`par_chunks_mut`) et ses pixels (triplets RGB)
// sont inversés gauche/droite sur place. Les couleurs sont conservées.
fn mirror_horizontal(mut image: Rgb8) -> Result<RgbImage, Box<dyn Error>> {
    let row_len = image.width as usize * 3;

    image.data.par_chunks_mut(row_len).for_each(|row| {
        let pixels = row.len() / 3;
        for i in 0..pixels / 2 {
            let (left, right) = (i * 3, (pixels - 1 - i) * 3);
            row.swap(left, right);
            row.swap(left + 1, right + 1);
            row.swap(left + 2, right + 2);
        }
    });

    RgbImage::from_raw(image.width, image.height, image.data)
        .ok_or_else(|| "buffer de pixels incohérent avec les dimensions".into())
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
