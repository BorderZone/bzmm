{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"; # Use unstable for newer packages
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      libraries = with pkgs; [
        webkitgtk_4_1
        gtk3
        cairo
        gdk-pixbuf
        glib
        dbus
        openssl
        librsvg
      ];

      packages = with pkgs; [
        # Use Node.js 18.x which is still supported
        nodejs_18
        yarn
        
        # System dependencies
        curl
        wget
        pkg-config
        dbus
        openssl
        glib
        gtk3
        libsoup_3
        webkitgtk_4_1
        librsvg
        act
        
        # Rust ecosystem
        rustc
        cargo
        rustfmt
        clippy
      ];

      env = {
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}";
        # Make sure Rust can find OpenSSL
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        # Tell Node.js to work with the available OpenSSL
        NODE_OPTIONS = "--openssl-legacy-provider";
      };
    in {
      devShell = pkgs.mkShell {
        buildInputs = packages;
        
        shellHook = ''
          # Set up environment variables
          export LD_LIBRARY_PATH=${env.LD_LIBRARY_PATH}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
          export XDG_DATA_DIRS=${env.XDG_DATA_DIRS}''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}
          export OPENSSL_DIR=${env.OPENSSL_DIR}
          export OPENSSL_LIB_DIR=${env.OPENSSL_LIB_DIR}
          export OPENSSL_INCLUDE_DIR=${env.OPENSSL_INCLUDE_DIR}
          export NODE_OPTIONS=${env.NODE_OPTIONS}
          
          # For tauri development
          export PATH=$PATH:$PWD/node_modules/.bin
          
          # Show environment info
          echo "Development environment ready:"
          echo "- Node.js: $(node --version)"
          echo "- Yarn: $(yarn --version)"
          echo "- Rust: $(rustc --version)"
          echo "- Cargo: $(cargo --version)"
          echo "- OpenSSL: $(openssl version)"
        '';
      };
    });
}
