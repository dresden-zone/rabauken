{ buildPackage, lib, pkg-config, openssl }:
buildPackage {
  pname = "chef";
  version = "0.1.0";

  src = ../.;
  cargoBuildOptions = (x: x ++ [ "-p chef" ]);
  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ ];

  meta = {
    description = "";
    homepage = "https://github.com/dresden-zone/rabauken";
  };
}
