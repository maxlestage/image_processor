# Convertisseur d'images BMP en niveaux de gris

Ce projet implémente un convertisseur d'images BMP en niveaux de gris à l'aide du Langage Rust.

![sample](https://github.com/maxlestage/image_processor/blob/master/assets/title.png)

## Fonctionnalités

- Lecture et analyse de l'en-tête BMP
- Lecture des pixels 24 bits RGB
- Conversion de chaque pixel en niveau de gris 8 bits
- Écriture du nouveau fichier BMP en niveaux de gris

## dependencies 
 - rayon = "1.7.0"

rustc 1.71.1 (eb26296b5 2023-08-03)

## Utilisation

Pour convertir une image BMP :

```
<!-- before build  -->
cargo build --release --profile=release
```

```
<!-- after build  -->
 ./target/release/image_processor 
```

Cela génèrera un nouveau fichier `grayscladed.bmp` avec l'image convertie en niveaux de gris.

### Pour exécuter un binaire Rust compilé qui se trouve dans le répertoire target/release, il y a deux méthodes principales

  Spécifier le chemin complet vers le binaire :

```
./target/release/image_processor
```

Cela permettra d'exécuter directement le binaire image_processor généré dans target/release.

Ajouter target/release dans votre variable d'environnement PATH :

```
export PATH="$PATH:/chemin/vers/votre/projet/target/release"
```

Cela ajoute le répertoire target/release à votre PATH. Vous pouvez alors exécuter le binaire directement par son nom :

```
image_processor ./assets/esla.bmp
```

Quelques points à noter :

Le binaire doit avoir les permissions d'exécution avec chmod +x

Pour que la modification du PATH soit permanente, il faut l'ajouter dans votre fichier de config shell (bashrc, zshrc, etc)

Vous pouvez aussi copier le binaire ailleurs si voulu pour l'exécuter plus facilement

L'utilisation de cargo run dans le répertoire du projet build et exécute automatiquement

## Implémentation

- La structure `BmpHeader` représente l'en-tête BMP. Elle contient les métadonnées comme les dimensions, le nombre de bits par pixel etc.

- La fonction `read_bmp_header` analyse l'en-tête brut et crée une instance de `BmpHeader`.

- `BmpHeader` implémente `as_bytes` pour sérialiser l'en-tête au format binaire.

- La fonction `generate_grayscale` contient la logique principale :

  - Lecture de l'en-tête

  - Lecture des pixels

  - Conversion RGB vers niveau de gris pour chaque pixel
  
  - Écriture du nouveau fichier

- Le niveau de gris est calculé en faisant la moyenne des composantes R, G, B.

## Head & Data Structure

| Offset | Field                           | Description                                                 |
| ------ | ------------------------------- | ----------------------------------------------------------- |
| 0      | `BM`                            | Magic number that identifies the file as a BMP image        |
| 2      | `File size`                     | The size of the file in bytes                               |
| 6      | `Reserved`                      | Reserved bytes                                              |
| 10     | `Offset to start of image data` | The offset to the start of the image data, in bytes         |
| 14     | `DIB header size`               | The size of the DIB header, in bytes                        |
| 18     | `Image width`                   | The width of the image, in pixels                           |
| 22     | `Image height`                  | The height of the image, in pixels                          |
| 26     | `Number of planes`              | The number of color planes in the image                     |
| 28     | `Bits per pixel`                | The number of bits per pixel in the image                   |
| 30     | `Compression`                   | The compression type used for the image data                |
| 34     | `Image size`                    | The size of the image data, in bytes                        |
| 38     | `Horizontal resolution`         | The horizontal resolution of the image, in pixels per meter |
| 42     | `Vertical resolution`           | The vertical resolution of the image, in pixels per meter   |
| 46     | `Colors used`                   | The number of colors used in the image                      |
| 50     | `Important colors`              | The number of important colors in the image                 |

## Conversion octets -> accès données

Les 14 premiers octets forment une section de taille fixe avec le nombre magique, la taille du fichier et le décalage des données d'image.
Les 40 octets suivants sont de taille variable et décrivent les dimensions, profondeur couleur et type de compression.
Le nombre magique identifie le fichier BMP sur 2 octets. La taille du fichier indique la taille totale sur 4 octets. Le décalage des données image pointe vers le début des données pixels sur 4 octets.
Les 40 octets restants contiennent des informations sur les dimensions, profondeur couleur et type de compression de l'image.
La largeur de l'image est codée sur 4 octets. La hauteur de l'image est codée sur 4 octets.
Le nombre de plans est une valeur sur 2 octets qui spécifie le nombre de plans de couleur dans l'image. Un plan de couleur est un groupe de bits qui représentent une couleur unique.
Le nombre de bits par pixel est une valeur sur 2 octets qui spécifie le nombre de bits par pixel dans l'image.
Le type de compression est une valeur sur 4 octets qui spécifie le type de compression utilisé pour les données d'image. Le type de compression peut être une des valeurs suivantes :

BI_RGB : Pas de compression  
BI_RLE8 : Compression RLE 8 bits
BI_RLE4 : Compression RLE 4 bits
BI_BITFIELDS : Bitfields pour stocker les données d'image
BI_JPEG : Compression JPEG
BI_PNG : Compression PNG

La taille des données d'image est une valeur sur 4 octets qui indique la taille des données d'image en octets.
La résolution horizontale est une valeur sur 4 octets qui indique la résolution horizontale de l'image en pixels par mètre.
La résolution verticale est une valeur sur 4 octets qui indique la résolution verticale de l'image en pixels par mètre.
Le nombre de couleurs utilisées est une valeur sur 4 octets qui indique le nombre de couleurs utilisées dans l'image.
Le nombre de couleurs importantes est une valeur sur 4 octets qui indique le nombre de couleurs importantes dans l'image.
Les couleurs importantes sont les couleurs les plus fréquemment utilisées dans l'image. Les autres couleurs ne sont pas utilisées et sont interpolées à partir des couleurs importantes.
