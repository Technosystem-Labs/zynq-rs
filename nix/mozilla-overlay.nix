let
  pkgs = import <nixpkgs> {};
  overlay = pkgs.fetchFromGitHub {
    owner = "mozilla";
    repo = "nixpkgs-mozilla";
    rev = "b1001ed670666ca4ce1c1b064481f88694315c1d";
    sha256 = "1hpig8z4pzdwc2vazr6hg7qyxllbgznsaivaigjnmrdszlxz55zz";
  };
in
  import overlay
