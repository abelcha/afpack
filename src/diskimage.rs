use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    RAW,
    ASIF,
    UDSB,
}

impl Default for Format {
    fn default() -> Self {
        Format::ASIF
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::RAW => write!(f, "RAW"),
            Format::ASIF => write!(f, "ASIF"),
            Format::UDSB => write!(f, "UDSB"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileSystem {
    APFS,
    ExFAT,
    MSDOS,
    None,
}

impl Default for FileSystem {
    fn default() -> Self {
        FileSystem::APFS
    }
}

impl std::fmt::Display for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSystem::APFS => write!(f, "APFS"),
            FileSystem::ExFAT => write!(f, "ExFAT"),
            FileSystem::MSDOS => write!(f, "MS-DOS"),
            FileSystem::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttachOptions {
    pub mount_point: Option<String>,
    pub readonly: bool,
    pub nobrowse: bool,
    pub verbose: bool,
    pub dry_run: bool,
}

impl AttachOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mount_point(mut self, mount_point: impl Into<String>) -> Self {
        self.mount_point = Some(mount_point.into());
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn nobrowse(mut self) -> Self {
        self.nobrowse = true;
        self
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

impl Default for AttachOptions {
    fn default() -> Self {
        Self {
            mount_point: None,
            readonly: false,
            nobrowse: false,
            verbose: false,
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateBlankOptions {
    pub size: String,
    pub fs: FileSystem,
    pub format: Format,
    pub dry_run: bool,
    pub verbose: bool,
}

impl CreateBlankOptions {
    pub fn new(size: impl Into<String>, fs: FileSystem, format: Format) -> Self {
        Self {
            size: size.into(),
            fs,
            format,
            dry_run: false,
            verbose: false,
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CreateFromOptions {
    pub format: Format,
    pub dry_run: bool,
    pub verbose: bool,
}

impl CreateFromOptions {
    pub fn new(format: Format) -> Self {
        Self {
            format,
            dry_run: false,
            verbose: false,
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl Default for CreateBlankOptions {
    fn default() -> Self {
        Self {
            size: "1GB".to_string(),
            fs: FileSystem::None,
            format: Format::default(),
            dry_run: false,
            verbose: false,
        }
    }
}

impl Default for CreateFromOptions {
    fn default() -> Self {
        Self {
            format: Format::default(),
            dry_run: false,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResizeOptions {
    pub size: String,
    pub dry_run: bool,
    pub verbose: bool,
}

impl ResizeOptions {
    pub fn new(size: impl Into<String>) -> Self {
        Self {
            size: size.into(),
            dry_run: false,
            verbose: false,
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[derive(Debug)]
pub enum DiskImageError {
    CommandFailed(String),
    InvalidPath(String),
    InvalidSize(String),
    DiskutilNotFound,
}

impl std::fmt::Display for DiskImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskImageError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            DiskImageError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            DiskImageError::InvalidSize(size) => write!(f, "Invalid size: {}", size),
            DiskImageError::DiskutilNotFound => write!(f, "diskutil command not found"),
        }
    }
}

impl std::error::Error for DiskImageError {}

pub type Result<T> = std::result::Result<T, DiskImageError>;

pub struct DiskImage;

impl DiskImage {
    /// Attach a disk image
    pub fn attach<P: AsRef<Path>>(image_path: P, options: AttachOptions) -> Result<String> {
        let path = image_path.as_ref();
        // if !path.exists() {
        //     return Err(DiskImageError::InvalidPath(path.display().to_string()));
        // }

        let mut cmd = Command::new("diskutil");
        cmd.arg("image").arg("attach");

        if let Some(mount_point) = &options.mount_point {
            // Only create directory if it doesn't exist and not in dry run
            if !Path::new(mount_point).exists() {
                if !options.dry_run {
                    std::fs::create_dir_all(Path::new(mount_point))
                        .map_err(|e| DiskImageError::CommandFailed(e.to_string()))?;
                }
            }
            cmd.arg("--mountPoint").arg(mount_point);
        }

        if options.verbose {
            cmd.arg("--verbose");
        }

        cmd.arg(path);

        if options.dry_run {
            let cmd_str = format!("{:?}", cmd);
            println!("[DRY RUN] Would execute: {}", cmd_str);
            return Ok(format!("[DRY RUN] Command: {}", cmd_str));
        }

        if options.verbose {
            let cmd_str = format!("{:?}", cmd);
            println!("[VERBOSE] Executing: {}", cmd_str);
        }

        let output = cmd.output().map_err(|_| DiskImageError::DiskutilNotFound)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(DiskImageError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Create a blank disk image
    /// diskutil image create blank --fs none --format ASIF --size 2GB ./node_modules.asif
    pub fn create_blank<P: AsRef<Path>>(
        image_path: P,
        options: CreateBlankOptions,
    ) -> Result<String> {
        let path = image_path.as_ref();

        // Validate size format (basic check)
        if !Self::is_valid_size(&options.size) {
            return Err(DiskImageError::InvalidSize(options.size));
        }

        if options.dry_run {
            let cmd_str = format!(
                "diskutil image create blank --fs {} --format {} --size {} {}",
                options.fs.to_string().to_lowercase(),
                options.format,
                options.size,
                path.display()
            );
            println!("[DRY RUN] Would execute: {}", cmd_str);
            return Ok(format!("[DRY RUN] Command: {}", cmd_str));
        }

        let mut cmd = Command::new("diskutil");
        cmd.arg("image").arg("create").arg("blank");

        cmd.arg("--fs").arg(options.fs.to_string().to_lowercase());
        cmd.arg("--format").arg(options.format.to_string());
        cmd.arg("--size").arg(&options.size);
        cmd.arg(path);

        if options.verbose {
            let cmd_str = format!("{:?}", cmd);
            println!("[VERBOSE] Executing: {}", cmd_str);
        }

        let output = cmd.output().map_err(|_| DiskImageError::DiskutilNotFound)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(DiskImageError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Create disk image from existing image
    /// diskutil image create from atuin.dmg image.asif
    pub fn create_from<P: AsRef<Path>, Q: AsRef<Path>>(
        source_path: P,
        dest_path: Q,
        options: CreateFromOptions,
    ) -> Result<String> {
        let source = source_path.as_ref();
        let dest = dest_path.as_ref();

        if options.dry_run {
            let cmd_str = format!(
                "diskutil image create from --format {} {} {}",
                options.format,
                source.display(),
                dest.display()
            );
            println!("[DRY RUN] Would execute: {}", cmd_str);
            return Ok(format!("[DRY RUN] Command: {}", cmd_str));
        }

        if !source.exists() {
            return Err(DiskImageError::InvalidPath(source.display().to_string()));
        }

        let mut cmd = Command::new("diskutil");
        cmd.arg("image").arg("create").arg("from");
        cmd.arg("--format").arg(options.format.to_string());
        cmd.arg(source);
        cmd.arg(dest);

        if options.verbose {
            let cmd_str = format!("{:?}", cmd);
            println!("[VERBOSE] Executing: {}", cmd_str);
        }

        let output = cmd.output().map_err(|_| DiskImageError::DiskutilNotFound)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(DiskImageError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Resize a disk image
    pub fn resize<P: AsRef<Path>>(image_path: P, options: ResizeOptions) -> Result<String> {
        let path = image_path.as_ref();

        if !Self::is_valid_size(&options.size) {
            return Err(DiskImageError::InvalidSize(options.size));
        }

        if options.dry_run {
            let cmd_str = format!(
                "diskutil image resize --size {} {}",
                options.size,
                path.display()
            );
            println!("[DRY RUN] Would execute: {}", cmd_str);
            return Ok(format!("[DRY RUN] Command: {}", cmd_str));
        }

        if !path.exists() {
            return Err(DiskImageError::InvalidPath(path.display().to_string()));
        }

        let mut cmd = Command::new("diskutil");
        cmd.arg("image").arg("resize");
        cmd.arg("--size").arg(&options.size);
        cmd.arg(path);

        if options.verbose {
            let cmd_str = format!("{:?}", cmd);
            println!("[VERBOSE] Executing: {}", cmd_str);
        }

        let output = cmd.output().map_err(|_| DiskImageError::DiskutilNotFound)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(DiskImageError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Detach a disk image
    pub fn detach<P: AsRef<Path>>(mount_point: P) -> Result<String> {
        let path = mount_point.as_ref();

        let mut cmd = Command::new("diskutil");
        cmd.arg("unmount").arg(path);

        let output = cmd.output().map_err(|_| DiskImageError::DiskutilNotFound)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(DiskImageError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Check if size format is valid (basic validation)
    fn is_valid_size(size: &str) -> bool {
        let size_lower = size.to_lowercase();
        size_lower.ends_with("b")
            || size_lower.ends_with("kb")
            || size_lower.ends_with("mb")
            || size_lower.ends_with("gb")
            || size_lower.ends_with("tb")
            || size_lower.ends_with("k")
            || size_lower.ends_with("m")
            || size_lower.ends_with("g")
            || size_lower.ends_with("t")
            || size.chars().all(|c| c.is_ascii_digit())
    }
}

// Convenience functions for easier usage
pub mod diskimage {
    use super::*;

    /// Attach a disk image with options
    pub fn attach<P: AsRef<Path>>(image_path: P, options: AttachOptions) -> Result<String> {
        DiskImage::attach(image_path, options)
    }

    /// Create a blank disk image with options
    pub fn create_blank<P: AsRef<Path>>(
        image_path: P,
        options: CreateBlankOptions,
    ) -> Result<String> {
        DiskImage::create_blank(image_path, options)
    }

    /// Create disk image from existing image
    pub fn create_from<P: AsRef<Path>, Q: AsRef<Path>>(
        source_path: P,
        dest_path: Q,
        options: CreateFromOptions,
    ) -> Result<String> {
        DiskImage::create_from(source_path, dest_path, options)
    }

    /// Resize a disk image
    pub fn resize<P: AsRef<Path>>(image_path: P, options: ResizeOptions) -> Result<String> {
        DiskImage::resize(image_path, options)
    }

    /// Detach/unmount a disk image
    pub fn detach<P: AsRef<Path>>(mount_point: P) -> Result<String> {
        DiskImage::detach(mount_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_display() {
        assert_eq!(FileSystem::APFS.to_string(), "APFS");
        assert_eq!(FileSystem::ExFAT.to_string(), "ExFAT");
        assert_eq!(FileSystem::MSDOS.to_string(), "MS-DOS");
        assert_eq!(FileSystem::None.to_string(), "None");
    }

    #[test]
    fn test_size_validation() {
        assert!(DiskImage::is_valid_size("5GB"));
        assert!(DiskImage::is_valid_size("100MB"));
        assert!(DiskImage::is_valid_size("1024"));
        assert!(!DiskImage::is_valid_size("invalid"));
    }

    #[test]
    fn test_default_options() {
        let attach_opts = AttachOptions::default();
        assert_eq!(attach_opts.mount_point, None);
        assert!(!attach_opts.readonly);
        assert!(!attach_opts.verbose);
        assert!(!attach_opts.dry_run);

        let create_blank_opts = CreateBlankOptions::default();
        assert_eq!(create_blank_opts.size, "1GB");
        assert_eq!(create_blank_opts.fs, FileSystem::None);
        assert_eq!(create_blank_opts.format, Format::ASIF);

        let create_from_opts = CreateFromOptions::default();
        assert_eq!(create_from_opts.format, Format::ASIF);
    }

    #[test]
    fn test_format_display() {
        assert_eq!(Format::RAW.to_string(), "RAW");
        assert_eq!(Format::ASIF.to_string(), "ASIF");
        assert_eq!(Format::UDSB.to_string(), "UDSB");
    }
}
