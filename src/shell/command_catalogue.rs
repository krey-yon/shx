//! The table of commands `shx` knows by name.
//!
//! A const array of string slices rather than a `HashSet`, for three reasons:
//! there is no build step to generate a set from, the data is a few kilobytes
//! that stay in the binary's read-only data, and iteration order is stable,
//! which makes a failure reproducible.
//!
//! Membership is answered by binary search over a sorted table. The table is
//! sorted by hand and a test asserts it, because a table that is accidentally
//! unsorted is a silently wrong lookup.

/// What a command is for, used for grouping in `--help` output and in
/// completions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Listing, moving, copying and deleting files.
    FileAndDirectory,
    /// Reading and editing file contents.
    FileViewing,
    /// Searching and filtering.
    Searching,
    /// Changing permissions and ownership.
    Permissions,
    /// System information and process control.
    SystemInformation,
    /// Network tools.
    Networking,
    /// Archives and compression.
    Archiving,
    /// Package managers, which vary by platform.
    PackageManagement,
    /// Language runtimes and build tools.
    Development,
    /// Users, sessions and privilege.
    UserManagement,
    /// Shell builtins.
    ShellBuiltins,
    /// Text processing.
    TextProcessing,
    /// Disk and storage.
    DiskAndStorage,
    /// Security tooling.
    Security,
    /// Logs and monitoring.
    Monitoring,
    /// Containers and orchestration.
    Containers,
    /// Cloud and infrastructure.
    Cloud,
    /// Clipboard tools, which are platform-specific.
    Clipboard,
    /// Scheduling and backgrounding.
    JobControl,
}

impl Category {
    /// The name shown in help output.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::FileAndDirectory => "files and directories",
            Self::FileViewing => "viewing and editing files",
            Self::Searching => "searching and filtering",
            Self::Permissions => "permissions",
            Self::SystemInformation => "system information",
            Self::Networking => "networking",
            Self::Archiving => "archiving",
            Self::PackageManagement => "package management",
            Self::Development => "development tools",
            Self::UserManagement => "users and sessions",
            Self::ShellBuiltins => "shell builtins",
            Self::TextProcessing => "text processing",
            Self::DiskAndStorage => "disk and storage",
            Self::Security => "security",
            Self::Monitoring => "logs and monitoring",
            Self::Containers => "containers",
            Self::Cloud => "cloud and infrastructure",
            Self::Clipboard => "clipboard",
            Self::JobControl => "job control",
        }
    }
}

/// One entry: a command and what it is for.
#[derive(Debug, Clone, Copy)]
pub struct CatalogueEntry {
    /// The command name, as typed.
    pub name: &'static str,
    /// What it is for.
    pub category: Category,
}

/// Expands a flat list of `(command, category)` pairs into a sorted slice.
///
/// One repetition level, not two. A macro that groups commands by category
/// looks tidier at the call site but needs a nested repetition, and referencing
/// the inner metavariable from inside a struct literal of the outer one is
/// ambiguous to the parser.
///
/// The emitted order is the call order, so the list below is sorted by name:
/// `is_known_command` binary searches it. `the_table_is_sorted` enforces that.
macro_rules! catalogue {
    ( $( ( $command:literal, $category:ident ) ),* $(,)? ) => {
        &[ $( CatalogueEntry { name: $command, category: Category::$category } ),* ]
    };
}

/// Every command name `shx` recognises, sorted by name for binary search.
///
/// Sorted with `LC_ALL=C sort -u`: byte order, deduplicated, one entry per
/// name. `duplicate_entries_are_absent` and `the_table_is_sorted` enforce both
/// properties.
///
/// A duplicate is not harmless. It inflates the binary, and — worse — it hides
/// the case where a command is filed under two categories and the first one
/// wins unpredictably.
pub const COMMANDS: &[CatalogueEntry] = catalogue! {
    ("ack", Searching), ("adduser", UserManagement), ("ag", Searching), ("age", Security),
    ("alias", ShellBuiltins), ("ansible", Cloud), ("apk", PackageManagement),
    ("apt", PackageManagement), ("apt-get", PackageManagement), ("arch", SystemInformation),
    ("arp", Networking), ("at", Monitoring), ("awk", TextProcessing), ("aws", Cloud),
    ("az", Cloud), ("badblocks", DiskAndStorage), ("basename", FileAndDirectory),
    ("bg", JobControl), ("blkid", DiskAndStorage), ("brew", PackageManagement),
    ("brotli", Archiving), ("buildah", Containers), ("bundle", Development),
    ("bunzip2", Archiving), ("bzip2", Archiving), ("cabal", Development),
    ("cargo", Development), ("cat", FileViewing), ("cd", FileAndDirectory),
    ("certbot", Security), ("chattr", Permissions), ("chef", Cloud), ("chfn", UserManagement),
    ("chgrp", FileAndDirectory), ("chmod", FileAndDirectory), ("choco", PackageManagement),
    ("chown", FileAndDirectory), ("chsh", UserManagement), ("clang", Development),
    ("clear", SystemInformation), ("cloud-init", Cloud), ("cls", SystemInformation),
    ("cmake", Development), ("code", FileViewing), ("column", TextProcessing),
    ("composer", Development), ("cp", FileAndDirectory), ("crontab", Monitoring),
    ("curl", Networking), ("cut", TextProcessing), ("date", SystemInformation),
    ("dd", DiskAndStorage), ("ddrescue", DiskAndStorage), ("deno", Development),
    ("df", SystemInformation), ("diff", TextProcessing), ("dig", Networking),
    ("dirname", FileAndDirectory), ("disown", JobControl), ("dmesg", SystemInformation),
    ("dnf", PackageManagement), ("docker", Containers), ("docker-compose", Containers),
    ("dotnet", Development), ("dpkg", PackageManagement), ("du", SystemInformation),
    ("echo", ShellBuiltins), ("egrep", Searching), ("emacs", FileViewing),
    ("emerge", PackageManagement), ("env", SystemInformation), ("eval", ShellBuiltins),
    ("exec", ShellBuiltins), ("exit", UserManagement), ("expand", TextProcessing),
    ("export", ShellBuiltins), ("fail2ban", Monitoring), ("false", ShellBuiltins),
    ("fd", Searching), ("fdisk", DiskAndStorage), ("fg", JobControl), ("fgrep", Searching),
    ("find", Searching), ("flatpak", PackageManagement), ("fmt", TextProcessing),
    ("fold", TextProcessing), ("free", SystemInformation), ("fsck", DiskAndStorage),
    ("ftp", Networking), ("fzf", Searching), ("gcc", Development), ("gcloud", Cloud),
    ("gem", Development), ("getfacl", Permissions), ("git", Development), ("go", Development),
    ("gpg", Security), ("gpg2", Security), ("gradle", Development), ("grep", Searching),
    ("groups", UserManagement), ("gunzip", Archiving), ("gzip", Archiving),
    ("hdparm", DiskAndStorage), ("hdparq", DiskAndStorage), ("head", FileViewing),
    ("helm", Containers), ("hostname", SystemInformation), ("htop", SystemInformation),
    ("ifconfig", Networking), ("iostat", SystemInformation), ("ip", Networking),
    ("iperf3", Networking), ("java", Development), ("javac", Development),
    ("jobs", JobControl), ("join", TextProcessing), ("journalctl", Monitoring),
    ("jq", Searching), ("julia", Development), ("k9s", Containers),
    ("kill", SystemInformation), ("killall", SystemInformation), ("kind", Containers),
    ("kubectl", Containers), ("kustomize", Containers), ("last", UserManagement),
    ("lens", Containers), ("less", FileViewing), ("letsencrypt", Security),
    ("link", FileAndDirectory), ("ln", FileAndDirectory), ("local", ShellBuiltins),
    ("locate", Searching), ("login", UserManagement), ("logout", UserManagement),
    ("logrotate", Monitoring), ("logwatch", Monitoring), ("ls", FileAndDirectory),
    ("lsattr", Permissions), ("lsblk", SystemInformation), ("lscpu", SystemInformation),
    ("lsof", SystemInformation), ("make", Development), ("micro", FileViewing),
    ("minikube", Containers), ("mkdir", FileAndDirectory), ("mkfs", DiskAndStorage),
    ("more", FileViewing), ("mount", SystemInformation), ("mtr", Networking),
    ("mv", FileAndDirectory), ("mvn", Development), ("nano", FileViewing), ("nc", Networking),
    ("nerdctl", Containers), ("netcat", Networking), ("netstat", Networking),
    ("nix-env", PackageManagement), ("nl", TextProcessing), ("nmap", Networking),
    ("node", Development), ("nohup", JobControl), ("npm", PackageManagement),
    ("npx", Development), ("nslookup", Networking), ("opam", Development),
    ("openssl", Security), ("packer", Cloud), ("pacman", PackageManagement),
    ("parted", DiskAndStorage), ("pass", Security), ("passwd", UserManagement),
    ("paste", TextProcessing), ("patch", TextProcessing), ("pbcopy", Clipboard),
    ("pbpaste", Clipboard), ("perl", Development), ("php", Development), ("ping", Networking),
    ("pip", PackageManagement), ("pip3", PackageManagement), ("pnpm", PackageManagement),
    ("podman", Containers), ("port", PackageManagement), ("pr", TextProcessing),
    ("printenv", SystemInformation), ("printf", ShellBuiltins), ("ps", SystemInformation),
    ("pulumi", Cloud), ("puppet", Cloud), ("pv", DiskAndStorage), ("pwd", FileAndDirectory),
    ("python", Development), ("python3", Development), ("quit", ShellBuiltins),
    ("rancher", Containers), ("read", ShellBuiltins), ("readlink", FileAndDirectory),
    ("realpath", FileAndDirectory), ("rg", Searching), ("ripgrep", Searching),
    ("rm", FileAndDirectory), ("rmdir", FileAndDirectory), ("route", Networking),
    ("rsync", Networking), ("ruby", Development), ("rustc", Development),
    ("sar", SystemInformation), ("say", Clipboard), ("sbt", Development), ("scp", Networking),
    ("screen", JobControl), ("script", SystemInformation), ("sed", TextProcessing),
    ("set", ShellBuiltins), ("setfacl", Permissions), ("setsid", JobControl),
    ("sftp", Networking), ("skaffold", Containers), ("skopeo", Containers),
    ("sleep", JobControl), ("smartctl", DiskAndStorage), ("snap", PackageManagement),
    ("socat", Networking), ("softwareupdate", PackageManagement), ("sort", FileAndDirectory),
    ("source", ShellBuiltins), ("split", TextProcessing), ("ss", Networking),
    ("ssh", Networking), ("ssh-add", Security), ("ssh-agent", Security),
    ("ssh-keygen", Security), ("stack", Development), ("stat", FileAndDirectory),
    ("strace", SystemInformation), ("su", UserManagement), ("sudo", UserManagement),
    ("swift", Development), ("sysctl", SystemInformation), ("systemctl", Monitoring),
    ("tac", TextProcessing), ("tail", FileViewing), ("tar", Archiving),
    ("tee", SystemInformation), ("telnet", Networking), ("terraform", Cloud),
    ("test", ShellBuiltins), ("time", SystemInformation), ("timeout", JobControl),
    ("tmux", JobControl), ("top", SystemInformation), ("touch", FileAndDirectory),
    ("tr", TextProcessing), ("tracepath", SystemInformation), ("traceroute", Networking),
    ("tree", FileAndDirectory), ("true", ShellBuiltins), ("tsc", Development),
    ("type", ShellBuiltins), ("umask", FileAndDirectory), ("umount", SystemInformation),
    ("unalias", ShellBuiltins), ("uname", SystemInformation), ("unexpand", TextProcessing),
    ("uniq", FileAndDirectory), ("unset", ShellBuiltins), ("unxz", Archiving),
    ("unzip", Archiving), ("uptime", SystemInformation), ("useradd", UserManagement),
    ("users", UserManagement), ("vagrant", Cloud), ("vcpkg", Development),
    ("view", FileViewing), ("vim", FileViewing), ("vm_stat", SystemInformation),
    ("vmstat", SystemInformation), ("w", UserManagement), ("wait", ShellBuiltins),
    ("watch", SystemInformation), ("wc", FileAndDirectory), ("wget", Networking),
    ("whereis", Searching), ("which", Searching), ("who", UserManagement),
    ("whoami", SystemInformation), ("whois", Networking), ("winget", PackageManagement),
    ("xargs", Searching), ("xbps-install", PackageManagement), ("xz", Archiving),
    ("yarn", Development), ("yq", Searching), ("yum", PackageManagement), ("zip", Archiving),
    ("zstd", Archiving), ("zypper", PackageManagement),
};

/// Every command name, sorted, for a binary search.
///
/// # Examples
///
/// ```
/// use shx::shell::is_known_command;
///
/// assert!(is_known_command("git"));
/// assert!(!is_known_command("frobnicate"));
/// ```
#[must_use]
pub fn is_known_command(name: &str) -> bool {
    let needle = name.trim();
    COMMANDS
        .binary_search_by(|entry| entry.name.cmp(needle))
        .is_ok()
}

/// The category a command belongs to, or `None` if it is not in the table.
///
/// # Examples
///
/// ```
/// use shx::shell::command_category;
/// use shx::shell::command_catalogue::Category;
///
/// assert_eq!(command_category("git"), Some(Category::Development));
/// assert_eq!(command_category("frobnicate"), None);
/// ```
#[must_use]
pub fn command_category(name: &str) -> Option<Category> {
    let needle = name.trim();
    COMMANDS
        .binary_search_by(|entry| entry.name.cmp(needle))
        .ok()
        .map(|index| COMMANDS[index].category)
}

/// Every category name, in a stable order, for help output.
///
/// # Examples
///
/// ```
/// use shx::shell::category_names;
///
/// assert!(category_names().contains(&"development tools"));
/// ```
#[must_use]
pub fn category_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = [
        Category::FileAndDirectory,
        Category::FileViewing,
        Category::Searching,
        Category::Permissions,
        Category::SystemInformation,
        Category::Networking,
        Category::Archiving,
        Category::PackageManagement,
        Category::Development,
        Category::UserManagement,
        Category::ShellBuiltins,
        Category::TextProcessing,
        Category::DiskAndStorage,
        Category::Security,
        Category::Monitoring,
        Category::Containers,
        Category::Cloud,
        Category::Clipboard,
        Category::JobControl,
    ]
    .iter()
    .map(|category| category.display_name())
    .collect();
    names.sort_unstable();
    names
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{COMMANDS, Category, command_category, is_known_command};

    #[test]
    fn the_table_is_sorted_by_name() {
        for pair in COMMANDS.windows(2) {
            assert!(
                pair[0].name < pair[1].name,
                "catalogue is not sorted: {:?} then {:?}",
                pair[0].name,
                pair[1].name
            );
        }
    }

    #[test]
    fn duplicate_entries_are_absent() {
        let mut seen = HashSet::new();
        for entry in COMMANDS {
            assert!(
                seen.insert(entry.name),
                "duplicate entry for {}: a command must be filed under exactly one category",
                entry.name
            );
        }
    }

    #[test]
    fn the_table_is_not_empty_and_not_absurd() {
        assert!(
            COMMANDS.len() > 200,
            "expected a real table, got {}",
            COMMANDS.len()
        );
        assert!(
            COMMANDS.len() < 600,
            "the table has probably picked up junk"
        );
    }

    #[test]
    fn every_entry_has_a_real_category_name() {
        for entry in COMMANDS {
            assert!(
                !entry.category.display_name().is_empty(),
                "{} has no category name",
                entry.name
            );
        }
    }

    #[test]
    fn common_commands_are_present() {
        for name in [
            "ls", "cd", "git", "cargo", "docker", "kubectl", "grep", "rm", "sudo", "python3",
            "npm", "make", "curl", "find", "chmod", "kill", "tmux",
        ] {
            assert!(is_known_command(name), "{name} should be known");
        }
    }

    #[test]
    fn words_are_not_commands() {
        for name in [
            "install", "please", "explain", "how", "what", "can", "you", "the", "a", "is",
        ] {
            assert!(!is_known_command(name), "{name} should not be a command");
        }
    }

    #[test]
    fn lookups_ignore_surrounding_whitespace() {
        assert!(is_known_command("  git  "));
    }

    #[test]
    fn lookup_is_case_sensitive_because_commands_are() {
        assert!(is_known_command("git"));
        assert!(!is_known_command("GIT"));
    }

    #[test]
    fn a_category_lookups_agree_with_membership() {
        for entry in COMMANDS {
            assert_eq!(command_category(entry.name), Some(entry.category));
        }
        assert_eq!(command_category("definitely-not-a-command"), None);
    }

    #[test]
    fn git_is_filed_under_development_not_containers() {
        assert_eq!(command_category("git"), Some(Category::Development));
    }

    #[test]
    fn the_category_list_has_no_duplicates() {
        let names = super::category_names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), before, "duplicate category display name");
    }
}
