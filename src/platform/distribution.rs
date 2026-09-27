//! The distribution families `shx` knows how to generate commands for, and the
//! package-manager commands each one uses.

use std::fmt;

use crate::error::{Result, ShxError};
use crate::platform::package_commands;

/// A Linux distribution, grouped by the package manager it uses.
///
/// The grouping is the point. `ubuntu`, `debian`, `mint` and `pop` are four
/// distributions with four names and one answer, so they share a variant rather
/// than repeating the same four command strings.
///
/// # Examples
///
/// ```
/// use shx::platform::distribution::Distribution;
///
/// let debian = Distribution::from_id("ubuntu").expect("a known distribution");
/// assert_eq!(debian.install("ripgrep"), "sudo apt install ripgrep");
///
/// let arch = Distribution::from_id("arch").expect("a known distribution");
/// assert_eq!(arch.install("ripgrep"), "sudo pacman -S ripgrep");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Distribution {
    /// apt: Debian, Ubuntu, Mint, Pop!_OS, Elementary, Kali, Raspbian.
    Debian,
    /// dnf: Fedora, RHEL, CentOS Stream, Rocky, AlmaLinux.
    Dnf,
    /// yum: the same family as dnf, on releases whose `yum` is not a symlink to
    /// it. CentOS 7 in particular.
    Yum,
    /// pacman: Arch, Manjaro, EndeavourOS, Artix.
    Pacman,
    /// zypper: openSUSE, SLES, SUSE Linux Enterprise.
    Zypper,
    /// emerge: Gentoo.
    Emerge,
    /// nix-env: NixOS and NixOS variants.
    Nix,
    /// xbps-install: Void Linux.
    Xbps,
    /// pkg: FreeBSD, OpenBSD, NetBSD.
    Pkg,
    /// pacman derivatives of the BSD family: nothing standard, so no.
    Alpine,
    /// A Linux distribution we can name but have no rules for.
    ///
    /// Not an error, because a plain shell still works and refusing to start
    /// would be worse than losing package-manager suggestions.
    Unknown,
}

impl Distribution {
    /// Classify a distribution by its `os-release` `ID`, with `ID_LIKE` as the
    /// fallback.
    ///
    /// Matching is by exact id first, then by `id_like` ancestry, which is how
    /// a derivative like Ubuntu reports `ID_LIKE=debian` and a smaller distro
    /// like elementary OS gets the right commands without a table entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// assert_eq!(Distribution::classify("ubuntu", None), Distribution::Debian);
    /// assert_eq!(Distribution::classify("nixos", None), Distribution::Nix);
    ///
    /// // A derivative with no entry of its own inherits from its parent.
    /// assert_eq!(
    ///     Distribution::classify("elementary", Some("ubuntu")),
    ///     Distribution::Debian
    /// );
    /// ```
    #[must_use]
    pub fn classify(id: &str, id_like: Option<&str>) -> Self {
        let id = id.trim().to_ascii_lowercase();

        if let Some(direct) = Self::from_id(&id) {
            return direct;
        }

        if let Some(parent) = id_like {
            for ancestor in parent.split_whitespace() {
                if let Some(inherited) = Self::from_id(&ancestor.to_ascii_lowercase()) {
                    return inherited;
                }
            }
        }

        Self::Unknown
    }

    /// The variant for a distribution id, or [`Unknown`](Self::Unknown) if the id
    /// is not one we have rules for.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// assert_eq!(Distribution::from_id("ubuntu"), Some(Distribution::Debian));
    /// assert_eq!(Distribution::from_id("plan9"), None);
    /// ```
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.trim().to_ascii_lowercase().as_str() {
            "debian" | "ubuntu" | "linuxmint" | "mint" | "pop" | "pop_os" | "elementary"
            | "kali" | "raspbian" | "devuan" | "deepin" | "neon" | "zorin" | "parrot" => {
                Self::Debian
            }

            "fedora" | "rhel" | "rhel9" | "redhat" | "rocky" | "almalinux" | "alma" | "centos"
            | "ol" | "oracle" | "amzn" | "amazon" | "nobara" => Self::Dnf,

            "centos7" | "rhel7" => Self::Yum,

            "arch" | "archarm" | "manjaro" | "endeavouros" | "endeavour" | "artix" | "cachyos" => {
                Self::Pacman
            }

            "opensuse"
            | "opensuse-leap"
            | "opensuse-tumbleweed"
            | "sles"
            | "sled"
            | "opensuse-microos" => Self::Zypper,

            "gentoo" | "funtoo" | "calculate" => Self::Emerge,
            "nixos" => Self::Nix,
            "void" => Self::Xbps,
            "freebsd" | "openbsd" | "netbsd" | "dragonfly" | "midnightbsd" => Self::Pkg,
            "alpine" | "postmarketos" => Self::Alpine,

            _ => return None,
        })
    }

    /// The command that installs a package.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// assert_eq!(
    ///     Distribution::from_id("debian").expect("known").install("curl"),
    ///     "sudo apt install curl"
    /// );
    /// ```
    #[must_use]
    pub fn install(self, package: &str) -> String {
        format!("{} {package}", package_commands::install_prefix(self))
    }

    /// The command that updates the package index.
    #[must_use]
    pub fn update(self) -> String {
        package_commands::update_command(self).to_owned()
    }

    /// The command that upgrades installed packages.
    #[must_use]
    pub fn upgrade(self) -> String {
        package_commands::upgrade_command(self).to_owned()
    }

    /// The command that removes a package.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// assert_eq!(
    ///     Distribution::from_id("arch").expect("known").remove("vim"),
    ///     "sudo pacman -R vim"
    /// );
    /// ```
    #[must_use]
    pub fn remove(self, package: &str) -> String {
        format!("{} {package}", package_commands::remove_prefix(self))
    }

    /// The executable name, for display and for looking it up on `PATH`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// assert_eq!(Distribution::Debian.package_manager(), "apt");
    /// assert_eq!(Distribution::Unknown.package_manager(), "unknown");
    /// ```
    #[must_use]
    pub const fn package_manager(self) -> &'static str {
        match self {
            Self::Debian => "apt",
            Self::Dnf => "dnf",
            Self::Yum => "yum",
            Self::Pacman => "pacman",
            Self::Zypper => "zypper",
            Self::Emerge => "emerge",
            Self::Nix => "nix-env",
            Self::Xbps => "xbps-install",
            Self::Pkg => "pkg",
            Self::Alpine => "apk",
            Self::Unknown => "unknown",
        }
    }

    /// The name of the tool, for the prompt and for error messages.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Debian => "Debian family",
            Self::Dnf => "Fedora family",
            Self::Yum => "Yum",
            Self::Pacman => "Arch family",
            Self::Zypper => "openSUSE family",
            Self::Emerge => "Gentoo",
            Self::Nix => "NixOS",
            Self::Xbps => "Void Linux",
            Self::Pkg => "BSD",
            Self::Alpine => "Alpine family",
            Self::Unknown => "unknown",
        }
    }

    /// Whether we have real commands for this distribution.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// assert!(Distribution::Debian.is_supported());
    /// assert!(!Distribution::Unknown.is_supported());
    /// ```
    #[must_use]
    pub const fn is_supported(self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// The commands to use for a package operation, or an explanation of why we
    /// cannot.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedDistribution`](crate::error::ShxError::UnsupportedDistribution)
    /// for [`Unknown`](Self::Unknown), naming the distro so the user can file a
    /// useful issue or set the family by hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::distribution::Distribution;
    ///
    /// let error = Distribution::Unknown
    ///     .require_supported("plan9", "Plan 9")
    ///     .expect_err("unknown has no rules");
    /// assert!(error.to_string().contains("plan9"));
    /// ```
    pub fn require_supported(self, id: &str, name: &str) -> Result<Self> {
        if self.is_supported() {
            return Ok(self);
        }
        Err(ShxError::UnsupportedDistribution {
            id: id.to_owned(),
            distro: name.to_owned(),
        })
    }
}

impl fmt::Display for Distribution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::Distribution;
    use crate::error::ShxError;

    #[test]
    fn debian_derivatives_share_one_variant() {
        for id in ["ubuntu", "debian", "linuxmint", "pop", "elementary", "kali"] {
            assert_eq!(
                Distribution::from_id(id),
                Some(Distribution::Debian),
                "failed for {id}"
            );
        }
    }

    #[test]
    fn case_and_padding_do_not_matter() {
        assert_eq!(
            Distribution::from_id("  UBUNTU  "),
            Some(Distribution::Debian)
        );
    }

    #[test]
    fn every_family_has_a_distinct_install_prefix() {
        let families = [
            Distribution::Debian,
            Distribution::Dnf,
            Distribution::Yum,
            Distribution::Pacman,
            Distribution::Zypper,
            Distribution::Emerge,
            Distribution::Nix,
            Distribution::Xbps,
            Distribution::Pkg,
            Distribution::Alpine,
        ];

        let mut prefixes: Vec<String> = families.iter().map(|d| d.install("x")).collect();
        prefixes.sort();
        let count = prefixes.len();
        prefixes.dedup();

        assert_eq!(
            prefixes.len(),
            count,
            "two families share an install command"
        );
    }

    #[test]
    fn install_names_the_package() {
        assert_eq!(
            Distribution::Debian.install("ripgrep"),
            "sudo apt install ripgrep"
        );
        assert_eq!(
            Distribution::Pacman.install("ripgrep"),
            "sudo pacman -S ripgrep"
        );
        assert_eq!(
            Distribution::Alpine.install("ripgrep"),
            "sudo apk add ripgrep"
        );
    }

    #[test]
    fn an_unknown_distribution_says_so_rather_than_guessing() {
        let unknown = Distribution::Unknown;
        assert!(!unknown.is_supported());
        assert_eq!(unknown.package_manager(), "unknown");
    }

    #[test]
    fn require_supported_names_the_distro() {
        let error = Distribution::Unknown
            .require_supported("solaris", "Solaris")
            .expect_err("unknown has no rules");

        match &error {
            ShxError::UnsupportedDistribution { id, distro } => {
                assert_eq!(id, "solaris");
                assert_eq!(distro, "Solaris");
            }
            other => panic!("expected UnsupportedDistribution, got {other:?}"),
        }
    }

    #[test]
    fn require_supported_passes_a_known_family_through() {
        assert_eq!(
            Distribution::Debian
                .require_supported("ubuntu", "Ubuntu")
                .expect("debian is supported"),
            Distribution::Debian
        );
    }

    #[test]
    fn id_like_inheritance_covers_derivatives_with_no_entry() {
        for (id, parent, expected) in [
            ("steamos", "arch", Distribution::Pacman),
            ("pop", "ubuntu", Distribution::Debian),
            ("rocky", "rhel centos fedora", Distribution::Dnf),
            ("endeavouros", "arch", Distribution::Pacman),
        ] {
            assert_eq!(
                Distribution::classify(id, Some(parent)),
                expected,
                "failed for {id} like {parent}"
            );
        }
    }

    #[test]
    fn a_direct_id_beats_id_like() {
        assert_eq!(
            Distribution::classify("alpine", Some("arch")),
            Distribution::Alpine
        );
    }

    #[test]
    fn an_unknown_id_with_no_lineage_is_unknown() {
        assert_eq!(Distribution::classify("plan9", None), Distribution::Unknown);
        assert_eq!(
            Distribution::classify("weird", Some("alsoweird")),
            Distribution::Unknown
        );
    }

    #[test]
    fn update_and_upgrade_differ_for_arch() {
        assert_eq!(Distribution::Pacman.update(), "sudo pacman -Sy");
        assert_eq!(Distribution::Pacman.upgrade(), "sudo pacman -Syu");
    }

    #[test]
    fn display_names_are_distinct() {
        let mut names: Vec<&str> = [
            Distribution::Debian,
            Distribution::Dnf,
            Distribution::Yum,
            Distribution::Pacman,
            Distribution::Zypper,
            Distribution::Emerge,
            Distribution::Nix,
            Distribution::Xbps,
            Distribution::Pkg,
            Distribution::Alpine,
            Distribution::Unknown,
        ]
        .iter()
        .map(|distribution| distribution.display_name())
        .collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
