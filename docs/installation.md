# Installation

## Arch

### Install via the AUR

```
yay -Syu okshell
```

### From source

#### Install Dependencies

```
pacman -Syu \
bluez \
gtk4 \
gtk4-layer-shell \
hyprland \
hyprpicker \
libnotify \
libpipewire \
libpulse \
matugen \
networkmanager \
pam \
pipewire-alsa \
power-profiles-daemon \
upower \
wf-recorder
```

#### Install Make Dependencies

```
pacman -Syu \
clang \
rust \
```

#### Install Script Dependencies

```
pacman -Syu rsync
```

### Set up the rust

```
rustup stable default
```

### Add ~/.cargo/bin to your PATH

```
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.profile
```

#### Clone the repo and install

```
git clone git@github.com:JohnOberhauser/OkShell.git
cd OkShell
./install.sh
```

## Nix

OkShell ships a flake with a package, dev shell, and home-manager / NixOS modules.

### Try it without installing

```
nix run github:JohnOberhauser/OkShell
```

### Install via flake input

Add OkShell to your flake inputs:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    okshell.url = "github:JohnOberhauser/OkShell";
    okshell.inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

#### With home-manager (recommended)

```nix
{ inputs, ... }: {
  imports = [ inputs.okshell.homeManagerModules.default ];

  programs.okshell.enable = true;
}
```

#### With NixOS

```nix
{ inputs, ... }: {
  imports = [ inputs.okshell.nixosModules.default ];

  programs.okshell.enable = true;
}
```

#### Or just add it to packages directly

```nix
environment.systemPackages = [
  inputs.okshell.packages.${pkgs.system}.default
];
```

### From source

Make sure flakes are enabled (add `experimental-features = nix-command flakes` to `~/.config/nix/nix.conf` or `/etc/nix/nix.conf`).

```
git clone git@github.com:JohnOberhauser/OkShell.git
cd OkShell
nix build
./result/bin/okshell
```

### Hack on it

```
nix develop
cargo build
```
If you have direnv installed, `direnv allow` once and the dev shell will load automatically when you `cd` into the repo.
