gcc stdlib.c -c -o stdlib.o
cargo run -- examples/beatsaber.beatsaber
gcc examples/beatsaber.o stdlib.o -o beatsaber