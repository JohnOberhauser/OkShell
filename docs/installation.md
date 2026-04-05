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
alsa-lib \
bluez \
cairo \
gdk-pixbuf2 \
glib2 \
graphene \
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
pango \
power-profiles-daemon \
systemd-libs \
upower \
wf-recorder \
wireplumber
```

#### Install Make Dependencies

```
pacman -Syu \
clang \
rust \
```

#### Clone the repo and install

```
git clone git@github.com:JohnOberhauser/OkShell.git
cd OkShell
./install.sh
```