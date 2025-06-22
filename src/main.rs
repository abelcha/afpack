use applesauce::compressor::Kind;
use applesauce::progress::{Progress, Task};
use applesauce::FileCompressor;
use clap::Parser;
use std::path::Path;

// NoProgress implementation for applesauce
struct NoProgress;

impl Task for NoProgress {
    fn increment(&self, _amt: u64) {}
    fn error(&self, _message: &str) {}
}

impl Progress for NoProgress {
    type Task = NoProgress;

    fn error(&self, _path: &Path, _message: &str) {
        eprintln!("Error at {}: {}", _path.display(), _message);
    }

    fn file_task(&self, _path: &Path, _size: u64) -> Self::Task {
        NoProgress
    }
}
use std::process::{exit, Command};
use std::sync::OnceLock;

pub mod diskimage;
use diskimage::{AttachOptions, CreateFromOptions, DiskImage, DiskImageError, ResizeOptions};

// Global flags
static DRY_RUN: OnceLock<bool> = OnceLock::new();
static VERBOSE: OnceLock<bool> = OnceLock::new();

fn is_dry_run() -> bool {
    *DRY_RUN.get().unwrap_or(&false)
}

// Verbose logging utility
fn vlog(msg: &str) {
    if *VERBOSE.get().unwrap_or(&false) {
        println!("{}", msg);
    }
}

#[derive(Parser)]
#[command(name = "afpack")]
#[command(about = "CLI tool for managing large dependency folders using ASIF")]
#[command(version = "0.1.0")]
struct Cli {
    /// Artifact directory (node_modules, target, .build, etc.)
    /// If not specified, will auto-detect common directories
    afdir: Option<String>,

    /// Compression algorithm
    #[arg(long, default_value = "none")]
    #[arg(help = "Compression algorithm: none, lzfse, lzvn, zlib")]
    compress: String,

    /// Maximum ASIF size
    #[arg(long, default_value = "10G")]
    maxsize: String,

    /// Show what would be done without actually doing it
    #[arg(long)]
    dry_run: bool,

    /// Enable verbose output
    #[arg(long, short)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    DRY_RUN.set(cli.dry_run).unwrap();
    VERBOSE.set(cli.verbose).unwrap();

    if !check_macos_compatibility() {
        eprintln!("ASIF creation requires macOS 26 Tahoe or later");
        exit(1);
    }
    // Get artifact directory (must be specified)
    let Some(afdir) = cli.afdir else {
        eprintln!("Error: Artifact directory must be specified.");
        exit(1);
    };
    vlog(&format!(
        "Options:\n\tArtifact directory: {}\n\tCompression: {}\n\tMax size: {}\n\tDry run: {}",
        afdir, cli.compress, cli.maxsize, cli.dry_run
    ));
    // Check macOS version compatibility
    let asif_path = format!("{}.asif", afdir);

    if !Path::new(&asif_path).exists() {
        if let Err(e) = create_asif_image(&afdir, &asif_path, &cli.maxsize, &cli.compress) {
            eprintln!("error create image: {}", e);
            exit(1);
        }
        if cli.dry_run {
            println!("[DRY RUN] removing {}", afdir);
        } else {
            trash::delete(&afdir).unwrap();
        }
    }

    if let Err(e) = DiskImage::attach(
        &asif_path,
        AttachOptions::new()
            .with_dry_run(cli.dry_run)
            .with_verbose(cli.verbose)
            .with_mount_point(&afdir),
    ) {
        eprintln!("Error attaching ASIF: {}", e);
        exit(1);
    }
    vlog(&format!("attached {} -> {}", asif_path, afdir));

    FileCompressor::new().recursive_compress(
        std::iter::once(Path::new(&asif_path)),
        applesauce::compressor::Kind::Lzfse,
        1.0,
        2,
        &NoProgress,
        true,
    );
}

fn check_macos_compatibility() -> bool {
    // Simple check - in a real implementation, you'd parse the actual macOS version
    let output = Command::new("sw_vers").arg("-productVersion").output();

    if let Err(_) = output {
        return false;
    }

    let output = output.unwrap();
    let version = String::from_utf8_lossy(&output.stdout);
    // This is a simplified check - real implementation would parse version properly
    !version.trim().is_empty()
}

fn apply_compression(compress: &str, path: &str) {
    let compression_kind = match compress {
        "lzfse" => Kind::Lzfse,
        "lzvn" => Kind::Lzvn,
        "zlib" => Kind::Zlib,
        _ => {
            eprintln!(
                "Warning: Unknown compression type '{}', using default",
                compress
            );
            Kind::default()
        }
    };

    let mut compressor = FileCompressor::new();
    compressor.recursive_compress(
        std::iter::once(Path::new(path)),
        compression_kind,
        1.0,
        2,
        &NoProgress,
        true,
    );
}

fn create_asif_image(
    afdir: &str,
    asif_path: &str,
    maxsize: &str,
    compress: &str,
) -> Result<(), DiskImageError> {
    let dry_run = is_dry_run();

    // Check if source directory exists
    if !Path::new(afdir).exists() {
        // Create blank disk image with size if directory doesn't exist
        vlog("resizing blank image");

        let create_options = diskimage::CreateBlankOptions::new(
            maxsize,
            diskimage::FileSystem::APFS,
            diskimage::Format::ASIF,
        )
        .with_dry_run(dry_run)
        .with_verbose(*VERBOSE.get().unwrap_or(&false));
        DiskImage::create_blank(asif_path, create_options)?;
    } else {
        // Create disk image from existing directory
        let create_options = CreateFromOptions::new(diskimage::Format::ASIF)
            .with_dry_run(dry_run)
            .with_verbose(*VERBOSE.get().unwrap_or(&false));
        vlog("creating disk image from existing directory");
        DiskImage::create_from(afdir, asif_path, create_options)?;

        if compress != "none" {
            vlog("Applying compression");
            apply_compression(compress, afdir);
        }
        // sleep 3 sec:
        std::thread::sleep(std::time::Duration::from_secs(3));
        // Only resize when creating from existing directory
        let resize_options = ResizeOptions::new(maxsize)
            .with_dry_run(dry_run)
            .with_verbose(*VERBOSE.get().unwrap_or(&false));
        vlog("resizing disk image");
        DiskImage::resize(asif_path, resize_options)?;
    }
    Ok(())
}
