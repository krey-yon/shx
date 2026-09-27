//! Detection of the operating system and, on Linux, the distribution family.
//!
//! Detection is a runtime branch, not a `#[cfg]` branch. `shx` is one binary
//! that reports the right package manager on every machine, and the answer on
//! Linux comes from reading `/etc/os-release` at runtime rather than from what
//! the compiler saw when it was built.

pub mod distribution;
pub mod os_release;

pub use distribution::Distribution;

use std::fmt;
use std::sync::LazyLock;

use crate::error::{Result, ShxError};
use crate::platform::os_release::OsRelease;

/// The operating system, with the Linux distribution resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    /// The kernel-reported system name, lowercased: `linux`, `macos`,
    /// `windows`, or whatever else.
    pub system: String,
    /// The distribution, when [`is_linux`](Self::is_linux).
    pub distribution: Distribution,
    /// The `os-release` `ID`, for error messages that should name the exact
    /// distribution rather than the family.
    pub distribution_id: String,
    /// The `os-release` `NAME`, for the same reason.
    pub distribution_name: String,
}

impl Platform {
    /// Detect the current platform.
    ///
    /// Detection is runtime, not `#[cfg]`, so one binary reports correctly on
    /// every machine. `std::env::consts::OS` is compile-time, which is why this
    /// calls `uname`-equivalent detection through `std::process::Command`
    /// instead: a cross-compiled or containerised binary needs the answer from
    /// the machine it is running on.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedPlatform`](crate::error::ShxError::UnsupportedPlatform)
    /// for a system we do not recognise. Everything else, including a Linux
    /// distribution we have no package-manager rules for, is a success with
    /// [`Distribution::Unknown`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::platform::Platform;
    ///
    /// let platform = Platform::detect().expect("this platform is supported");
    /// println!("{} / {}", platform.system, platform.display_name());
    /// ```
    pub fn detect() -> Result<Self> {
        let system = detect_system()?;

        if system == "linux" {
            return Ok(Self::linux());
        }

        Ok(Self {
            system,
            distribution: Distribution::Unknown,
            distribution_id: String::new(),
            distribution_name: String::new(),
        })
    }

    /// The Linux platform, resolved from `/etc/os-release`.
    ///
    /// A missing or unreadable file is not an error: a Linux machine without
    /// `os-release` is unusual but real, in a minimal container for instance,
    /// and refusing to start would be worse than having no package-manager
    /// suggestions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::platform::Platform;
    ///
    /// let platform = Platform::linux();
    /// println!("{}", platform.display_name());
    /// ```
    #[must_use]
    pub fn linux() -> Self {
        let release = OsRelease::read_system().ok();

        let distribution_id = release
            .as_ref()
            .and_then(OsRelease::id)
            .unwrap_or_default()
            .to_owned();
        let distribution_name = release
            .as_ref()
            .and_then(OsRelease::name)
            .unwrap_or_default()
            .to_owned();

        let distribution = release.as_ref().map_or(Distribution::Unknown, |release| {
            Distribution::classify(release.id().unwrap_or_default(), release.get("id_like"))
        });

        Self {
            system: "linux".to_owned(),
            distribution,
            distribution_id,
            distribution_name,
        }
    }

    /// macOS.
    #[must_use]
    pub fn macos() -> Self {
        Self {
            system: "macos".to_owned(),
            distribution: Distribution::Unknown,
            distribution_id: String::new(),
            distribution_name: String::new(),
        }
    }

    /// Windows, or anything else that is neither Linux nor macOS.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedPlatform`](crate::error::ShxError::UnsupportedPlatform)
    /// for a system that is not one of the three.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::Platform;
    ///
    /// let windows = Platform::for_system("windows").expect("windows is supported");
    /// assert!(!windows.is_linux());
    /// ```
    pub fn for_system(system: &str) -> Result<Self> {
        match system.trim().to_ascii_lowercase().as_str() {
            "linux" => Ok(Self::linux()),
            "macos" | "darwin" => Ok(Self::macos()),
            "windows" => Ok(Self {
                system: "windows".to_owned(),
                distribution: Distribution::Unknown,
                distribution_id: String::new(),
                distribution_name: String::new(),
            }),
            other => Err(ShxError::UnsupportedPlatform(other.to_owned())),
        }
    }

    /// Whether this is Linux.
    #[must_use]
    pub fn is_linux(&self) -> bool {
        self.system == "linux"
    }

    /// Whether this is macOS.
    #[must_use]
    pub fn is_macos(&self) -> bool {
        self.system == "macos"
    }

    /// The distribution, for package-manager commands.
    #[must_use]
    pub const fn distribution(&self) -> Distribution {
        self.distribution
    }

    /// A one-line description of the machine, for the prompt to send as context
    /// to a model.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::platform::Platform;
    ///
    /// let platform = Platform::detect().expect("supported");
    /// println!("{}", platform.prompt_context());
    /// ```
    #[must_use]
    pub fn prompt_context(&self) -> String {
        if self.is_linux() {
            let name = if self.distribution_name.is_empty() {
                self.distribution.to_string()
            } else {
                self.distribution_name.clone()
            };
            format!(
                "Operating system: Linux, distribution: {name}, package manager: {}",
                self.distribution.package_manager()
            )
        } else {
            format!("Operating system: {}", self.system)
        }
    }

    /// A human-readable name for the machine.
    #[must_use]
    pub fn display_name(&self) -> String {
        if self.is_linux() && !self.distribution_name.is_empty() {
            return self.distribution_name.clone();
        }
        self.system.clone()
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.display_name(), self.system)
    }
}

/// The detected platform for this process, computed once.
///
/// `LazyLock` rather than a dependency: this is stdlib, and it is the only
/// global in the crate.
static CURRENT: LazyLock<Platform> = LazyLock::new(|| {
    Platform::detect().unwrap_or_else(|_| {
        // Detection failing means an OS we have never heard of. Falling back to the
        // compile-time target keeps the tool usable rather than panicking, and the
        // package-manager suggestions simply stop.
        Platform::for_system(std::env::consts::OS).unwrap_or_else(|_| Platform {
            system: std::env::consts::OS.to_owned(),
            distribution: Distribution::Unknown,
            distribution_id: String::new(),
            distribution_name: String::new(),
        })
    })
});

/// The detected platform, computed on first use and cached.
///
/// # Examples
///
/// ```no_run
/// use shx::platform::current_platform;
///
/// println!("{}", current_platform().prompt_context());
/// ```
#[must_use]
pub fn current_platform() -> &'static Platform {
    &CURRENT
}

/// Ask the machine what it is, rather than trusting the compile-time target.
///
/// `std::env::consts::OS` is what the compiler saw. A binary built on Linux and
/// run under emulation, or in a container with a different userland, can
/// disagree with the kernel, and the kernel is the right answer.
fn detect_system() -> Result<String> {
    if let Ok(output) = std::process::Command::new("uname").arg("-s").output()
        && output.status.success()
    {
        let name = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_ascii_lowercase();
        let name = name.strip_prefix("linux").map_or_else(
            || name.clone(),
            |_| {
                if name.contains("darwin") {
                    "macos".to_owned()
                } else {
                    "linux".to_owned()
                }
            },
        );
        if name == "linux" || name == "macos" {
            return Ok(name);
        }
    }

    let fallback = std::env::consts::OS.to_ascii_lowercase();
    Platform::for_system(&fallback)?;
    Ok(match fallback.as_str() {
        "macos" => "macos".to_owned(),
        other => other.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::{Platform, current_platform};
    use crate::error::ShxError;
    use crate::platform::distribution::Distribution;

    #[test]
    fn detection_succeeds_on_the_machine_running_the_tests() {
        // Not portable in the sense of a fixed answer, but it must never fail:
        // if this breaks, every user on that platform gets a panic at startup.
        let platform = Platform::detect().expect("this platform is supported");
        assert!(
            platform.is_linux() || platform.is_macos() || platform.system == "windows",
            "unexpected system {}",
            platform.system
        );
    }

    #[test]
    fn the_cached_platform_matches_a_fresh_detection() {
        assert_eq!(
            current_platform().system,
            Platform::detect().expect("detection").system
        );
    }

    #[test]
    fn linux_resolves_its_distribution() {
        let platform = Platform::linux();
        assert!(platform.is_linux());
        // Whether the machine has os-release or not, the field exists and the
        // accessors work.
        let _ = platform.display_name();
    }

    #[test]
    fn macos_has_no_distribution() {
        let platform = Platform::macos();
        assert!(platform.is_macos());
        assert!(!platform.is_linux());
        assert_eq!(platform.distribution(), Distribution::Unknown);
    }

    #[test]
    fn darwin_is_accepted_as_a_spelling_of_macos() {
        assert_eq!(
            Platform::for_system("Darwin").expect("ok"),
            Platform::macos()
        );
    }

    #[test]
    fn an_unknown_system_is_an_error_naming_it() {
        let error = Platform::for_system("plan9").expect_err("plan9 is not supported");
        match &error {
            ShxError::UnsupportedPlatform(name) => assert_eq!(name, "plan9"),
            other => panic!("expected UnsupportedPlatform, got {other:?}"),
        }
    }

    #[test]
    fn the_prompt_context_names_the_package_manager() {
        let platform = Platform::linux();
        let context = platform.prompt_context();
        assert!(context.contains("Linux"));
        assert!(context.contains("package manager"));
    }

    #[test]
    fn a_macos_context_does_not_claim_a_package_manager() {
        // dwarp's os_info reports brew for macOS, which is right, but reporting
        // it as a "distribution" is not.
        let context = Platform::macos().prompt_context();
        assert!(context.contains("macos"));
        assert!(!context.contains("package manager"));
    }

    #[test]
    fn display_falls_back_to_the_system_name() {
        let windows = Platform::for_system("windows").expect("supported");
        assert_eq!(windows.display_name(), "windows");
    }

    #[test]
    fn display_uses_the_distribution_name_when_there_is_one() {
        let platform = Platform {
            system: "linux".to_owned(),
            distribution: Distribution::Debian,
            distribution_id: "ubuntu".to_owned(),
            distribution_name: "Ubuntu".to_owned(),
        };
        assert_eq!(platform.display_name(), "Ubuntu");
    }
}
