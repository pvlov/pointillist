# Pointillist

Pointillist is a smol cli tool that allows you to convert any gif into a [pointillism-style](https://en.wikipedia.org/wiki/Pointillism) gif. 

## Usage:

```
pointillist [OPTIONS] --in-path <IN_PATH> --out-path <OUT_PATH>

Options:
  -i, --in-path <IN_PATH>        Path to the input GIF file
  -o, --out-path <OUT_PATH>      Path to the output GIF file
  -b, --block-size <BLOCK_SIZE>  Size of the blocks to cluster pixels into [default: 8]
  -p, --padding <PADDING>        How much padding to add between the circles [default: 2]
  -r, --radius <RADIUS>          Maximum radius of the circles [default: 8]
  -d, --delay <DELAY>            Delay of the frames in the output GIF [default: 5]
  -h, --help                     Print help
  -V, --version                  Print version
```

The most interesting option here is `block-size` which allows you to make the gif as detailed as you want it to.

<img src="https://github.com/user-attachments/assets/89ab3600-53f4-4ac9-9706-9fb17176e886" width="200" height="200">
<img src="https://github.com/user-attachments/assets/6f850e7b-b1a4-4e9c-8456-fe9703d13843" width="200" height="200">
