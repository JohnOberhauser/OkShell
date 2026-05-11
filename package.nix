{
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  clang,
  llvmPackages,
  makeWrapper,
  # Runtime libraries (mirrors PKGBUILD `depends`)
  alsa-lib,
  bluez,
  glib,
  gtk4,
  gtk4-layer-shell,
  libnotify,
  pipewire,
  libpulseaudio,
  pam,
  systemd,
  # Runtime tools invoked by okshell at runtime
  hyprland,
  hyprpicker,
  matugen,
  networkmanager,
  power-profiles-daemon,
  upower,
  wf-recorder,
}:

rustPlatform.buildRustPackage rec {
  pname = "okshell";
  version = "0.10.1";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    # If any dependencies in Cargo.lock come from git rather than crates.io,
    # uncomment and fill in hashes here. Use lib.fakeHash first, then replace
    # with the value the build error prints.
    # outputHashes = {
    #   "some-crate-0.1.0" = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    # };
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
    clang
    makeWrapper
  ];

  buildInputs = [
    alsa-lib
    bluez
    glib
    gtk4
    gtk4-layer-shell
    libnotify
    llvmPackages.libclang.lib
    pipewire
    libpulseaudio
    pam
    systemd
  ];

  # bindgen (used transitively by pam-sys and others) needs to locate
  # libclang.so. clang-sys reads LIBCLANG_PATH at build time.
  env = {
    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
    # Match the Arch PKGBUILD's `options=(!lto)`.
    CARGO_PROFILE_RELEASE_LTO = "false";
  };

  # Runtime tools that okshell shells out to. Injecting them onto the wrapped
  # binary's PATH keeps the package self-contained on NixOS, where these
  # aren't available globally by default.
  runtimeDeps = [
    hyprland
    hyprpicker
    matugen
    networkmanager
    power-profiles-daemon
    upower
    wf-recorder
  ];

  # okshell-gamma's doctests fail to link glib in the Nix test sandbox.
  # The library and integration tests still run; doctests are documentation
  # examples, not behavioral coverage.
  cargoTestFlags = [ "--workspace" "--lib" "--bins" "--tests" ];

  postInstall = ''
    mkdir -p $out/share/icons
    cp -r icons/OkMaterial $out/share/icons/
    cp -r icons/OkPhosphor $out/share/icons/
  '';

  preFixup = ''
    gappsWrapperArgs+=(
      --prefix PATH : "${lib.makeBinPath runtimeDeps}"
    )
  '';

  meta = {
    description = "A customizable shell for Hyprland";
    homepage = "https://github.com/JohnOberhauser/OkShell";
    license = lib.licenses.gpl3Only;
    platforms = lib.platforms.linux;
    mainProgram = "okshell";
    maintainers = [ ];
  };
}
