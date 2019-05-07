with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "may-env";
  buildInputs = [ pkgconfig openssl rustup ]; # gnuplot linuxPackages.perf ];
}
