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