# Convertisseur d'images

Outil en ligne de commande écrit en Rust qui transforme **n'importe quelle
image** (y compris les fichiers **RAW des constructeurs**) au choix en
**niveaux de gris** ou en **miroir horizontal**.

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
./target/release/image_processor <entrée> <sortie> [mode]
```

`mode` (optionnel) :

- `grayscale` (défaut) — conversion en niveaux de gris (alias : `gris`, `g`)
- `mirror` — effet miroir horizontal, couleurs conservées (alias : `miroir`, `m`)

Exemples :

```sh
# RAW Nikon -> PNG en niveaux de gris (mode par défaut)
./target/release/image_processor photo.NEF gris.png

# JPEG -> BMP en miroir horizontal
./target/release/image_processor photo.jpg flip.bmp mirror

# Détection par contenu (fichier sans extension)
./target/release/image_processor capture out.png
```

## Implémentation

- `decode_any` route le décodage : `image` pour les formats raster,
  `imagepipe`/`rawloader` pour les RAW (avec repli RAW si `image` échoue).
- Les deux opérations sont parallélisées avec `rayon` :
  - *grayscale* : chaque triplet RGB est réduit à un octet de luminance
    (moyenne R, G, B), l'ordre des pixels étant préservé.
  - *mirror* : chaque ligne est confiée à une tâche et ses pixels sont
    inversés gauche/droite sur place (couleurs conservées).
- La sortie (8 bits niveaux de gris, ou RGB pour le miroir) est encodée
  selon le format déduit de l'extension du fichier de sortie.
