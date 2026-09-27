//! The literal command strings for each distribution family.

use crate::platform::distribution::Distribution;

pub(crate) fn install_prefix(d: Distribution) -> &'static str {
    match d {
        Distribution::Debian => "sudo apt install",
        Distribution::Dnf => "sudo dnf install",
        Distribution::Yum => "sudo yum install",
        Distribution::Pacman => "sudo pacman -S",
        Distribution::Zypper => "sudo zypper install",
        Distribution::Emerge => "sudo emerge",
        Distribution::Nix => "nix-env -iA",
        Distribution::Xbps => "sudo xbps-install -S",
        Distribution::Pkg => "sudo pkg install",
        Distribution::Alpine => "sudo apk add",
        Distribution::Unknown => "unknown",
    }
}

pub(crate) fn update_command(d: Distribution) -> &'static str {
    match d {
        Distribution::Debian => "sudo apt update",
        Distribution::Dnf => "sudo dnf check-update",
        Distribution::Yum => "sudo yum check-update",
        Distribution::Pacman => "sudo pacman -Sy",
        Distribution::Zypper => "sudo zypper refresh",
        Distribution::Emerge => "sudo emerge --update --deep",
        Distribution::Nix => "nix-channel --update",
        Distribution::Xbps => "sudo xbps-install -S",
        Distribution::Pkg => "sudo pkg update",
        Distribution::Alpine => "sudo apk update",
        Distribution::Unknown => "unknown",
    }
}

pub(crate) fn upgrade_command(d: Distribution) -> &'static str {
    match d {
        Distribution::Debian => "sudo apt upgrade",
        Distribution::Dnf => "sudo dnf upgrade",
        Distribution::Yum => "sudo yum update",
        Distribution::Pacman => "sudo pacman -Syu",
        Distribution::Zypper => "sudo zypper update",
        Distribution::Emerge => "sudo emerge --update --deep @world",
        Distribution::Nix => "nix-env -u",
        Distribution::Xbps => "sudo xbps-install -Syu",
        Distribution::Pkg => "sudo pkg upgrade",
        Distribution::Alpine => "sudo apk upgrade",
        Distribution::Unknown => "unknown",
    }
}

pub(crate) fn remove_prefix(d: Distribution) -> &'static str {
    match d {
        Distribution::Debian => "sudo apt remove",
        Distribution::Dnf => "sudo dnf remove",
        Distribution::Yum => "sudo yum remove",
        Distribution::Pacman => "sudo pacman -R",
        Distribution::Zypper => "sudo zypper remove",
        Distribution::Emerge => "sudo emerge --unmerge",
        Distribution::Nix => "nix-env -e",
        Distribution::Xbps => "sudo xbps-remove",
        Distribution::Pkg => "sudo pkg delete",
        Distribution::Alpine => "sudo apk del",
        Distribution::Unknown => "unknown",
    }
}
