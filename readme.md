# Convertisseur d'images en niveaux de gris

Outil en ligne de commande écrit en Rust qui convertit **n'importe quelle
image** en niveaux de gris, y compris les fichiers **RAW des constructeurs**.

![sample](https://github.com/maxlestage/image_processor/blob/master/assets/title.png)

## Formats supportés

**En entrée :**

- Raster classiques : PNG, JPEG, GIF, WebP, TIFF, BMP, ICO, etc. (via la
  crate [`image`](https://crates.io/crates/image)).
- RAW constructeurs : Canon (CR2/CR3/CRW), Nikon (NEF/NRW), Sony
  (ARW/SR2/SRF), Fujifilm (RAF), Panasonic (RW2), Olympus (ORF), Pentax
  (PEF), Adobe DNG, Leica, Kodak, Hasselblad, Sigma X3F, Phase One, etc.
  (via [`imagepipe`](https://crates.io/crates/imagepipe) /
  [`rawloader`](https://crates.io/crates/rawloader), 100 % Rust, sans
  dépendance système).

Le format est détecté par l'extension **et** par le contenu (magic bytes).

**En sortie :** le format est déduit de l'extension du fichier de sortie
(PNG par défaut si l'extension est absente ou inconnue).

## Dépendances

- `image` — décodage/encodage des formats raster
- `imagepipe` + `rawloader` — décodage des RAW constructeurs
- `rayon` — conversion en niveaux de gris parallélisée (multi-cœurs)

## Compilation

```sh
cargo build --release
```

## Utilisation

```sh
./target/release/image_processor <entrée> <sortie>
```

Exemples :

```sh
# RAW Nikon -> PNG en niveaux de gris
./target/release/image_processor photo.NEF gris.png

# JPEG -> BMP (format de sortie déduit de l'extension)
./target/release/image_processor photo.jpg gris.bmp

# Détection par contenu (fichier sans extension)
./target/release/image_processor capture out.png
```

## Implémentation

- `decode_any` route le décodage : `image` pour les formats raster,
  `imagepipe`/`rawloader` pour les RAW (avec repli RAW si `image` échoue).
- La conversion est parallélisée avec `rayon` : chaque triplet RGB est
  réduit à un octet de luminance (moyenne des composantes R, G, B), en
  préservant l'ordre des pixels.
- La sortie est une image 8 bits en niveaux de gris, encodée selon le
  format déduit de l'extension du fichier de sortie.
