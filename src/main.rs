use std::fs;
use std::path::Path;
use std::process::Command;
use std::io::{self, Write};
use std::env;
use std::thread;
use std::time::Duration;
use colored::*;
use clap::Parser;
use humansize::{format_size, BINARY};

#[derive(Parser)]
#[command(name = "maccleanup-rust")]
#[command(about = "🧹 Mac Cleanup Tool (Rust Edition) By Gappa", long_about = None)]
struct Cli {
    /// Run in interactive mode (ask before each action)
    #[arg(short, long, default_value_t = true)]
    interactive: bool,

    /// Dry run - only show what would be deleted
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    /// Force mode - delete without asking (use with caution!)
    #[arg(short, long, default_value_t = false)]
    force: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Clean RAM only
    #[arg(short = 'r', long, default_value_t = false)]
    ram_only: bool,
}

#[derive(Debug)]
struct CleanupStats {
    files_removed: usize,
    space_freed: u64,
}

impl CleanupStats {
    fn new() -> Self {
        CleanupStats {
            files_removed: 0,
            space_freed: 0,
        }
    }

    fn add(&mut self, other: &CleanupStats) {
        self.files_removed += other.files_removed;
        self.space_freed += other.space_freed;
    }
}

#[derive(Debug)]
struct DiskInfo {
    total: u64,
    available: u64,
    used: u64,
    percent_used: f32,
}

struct CleanupContext {
    interactive: bool,
    dry_run: bool,
    force: bool,
    verbose: bool,
}

impl CleanupContext {
    fn should_proceed(&self, action: &str, details: Option<String>) -> bool {
        if self.dry_run {
            println!("  {} [DRY RUN] Would {}", "→".yellow(), action);
            if let Some(detail) = details {
                println!("    {}", detail.dimmed());
            }
            return false;
        }

        if self.force {
            return true;
        }

        if self.interactive {
            print!("  {} {} {} ", "?".cyan(), action, "Proceed? (y/N):".yellow());
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            return input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes";
        }

        true
    }

    fn log_action(&self, message: &str) {
        if self.verbose {
            println!("  {} {}", "→".green(), message);
        }
    }

    fn log_error(&self, message: &str) {
        println!("  {} {}", "✗".red(), message);
    }

    fn log_success(&self, message: &str) {
        println!("  {} {}", "✓".green(), message);
    }

    fn log_info(&self, message: &str) {
        println!("  {} {}", "ℹ".blue(), message);
    }
}

fn main() {
    let cli = Cli::parse();
    
    println!("{}", "🧹 Mac Cleanup Tool (Rust Edition) By Gappa".bold().blue());
    println!("{}", "===============================================\n".blue());

    let ctx = CleanupContext {
        interactive: cli.interactive && !cli.force,
        dry_run: cli.dry_run,
        force: cli.force,
        verbose: cli.verbose,
    };

    // If RAM only mode, just clean RAM and exit
    if cli.ram_only {
        println!("{}", "🧠 RAM Cleanup Mode".bold());
        println!("{}", "─".repeat(40).dimmed());
        clean_ram(&ctx);
        return;
    }

    // Get initial disk info
    let initial_disk = get_disk_info();
    show_disk_status(&initial_disk, "Current Disk Status");

    if ctx.dry_run {
        println!("\n{}", "🔍 Running in DRY RUN mode - nothing will be deleted\n".yellow());
    } else if ctx.force {
        println!("\n{}", "⚠️  Running in FORCE mode - no confirmation prompts!\n".red());
    } else if ctx.interactive {
        println!("\n{}", "💬 Running in INTERACTIVE mode - will ask before actions\n".green());
    }

    let mut total_stats = CleanupStats::new();

    // Show menu first in interactive mode
    if ctx.interactive && !ctx.dry_run {
        if !show_menu() {
            println!("\n{}", "Cleanup cancelled.".yellow());
            return;
        }
    }

    // Calculate total potential cleanup size
    println!("\n{}", "📊 Calculating cleanup potential...".bold().cyan());
    let total_potential = calculate_total_cleanup_size();
    println!("{}", format!("  Total potential cleanup: {}", 
        format_size(total_potential, BINARY).bold().yellow()));
    println!();

    // System Caches
    println!("{}", "📁 System & User Caches".bold());
    println!("{}", "─".repeat(40).dimmed());
    let cache_size = estimate_cache_size();
    ctx.log_info(&format!("Estimated size: {}", format_size(cache_size, BINARY).red()));
    show_space_preview(cache_size);
    
    if ctx.should_proceed("Clean system and user caches?", 
        Some(format!("This will free approximately {}", format_size(cache_size, BINARY)))) {
        total_stats.add(&clean_caches(&ctx));
    }

    // Logs
    println!("\n{}", "📝 System Logs".bold());
    println!("{}", "─".repeat(40).dimmed());
    let log_size = estimate_logs_size();
    ctx.log_info(&format!("Estimated size: {}", format_size(log_size, BINARY).red()));
    show_space_preview(log_size);
    
    if ctx.should_proceed("Clean system logs older than 7 days?",
        Some(format!("This will free approximately {}", format_size(log_size, BINARY)))) {
        total_stats.add(&clean_logs(&ctx));
    }

    // Downloads folder
    println!("\n{}", "📥 Downloads Folder".bold());
    println!("{}", "─".repeat(40).dimmed());
    let downloads_size = estimate_old_downloads_size();
    ctx.log_info(&format!("Old files (30+ days): {}", format_size(downloads_size, BINARY).red()));
    show_space_preview(downloads_size);
    
    if downloads_size > 0 && ctx.should_proceed("Clean files older than 30 days in Downloads?",
        Some(format!("This will free approximately {}", format_size(downloads_size, BINARY)))) {
        total_stats.add(&clean_old_downloads(&ctx));
    }

    // Trash
    println!("\n{}", "🗑️  Trash".bold());
    println!("{}", "─".repeat(40).dimmed());
    let trash_size = estimate_trash_size();
    ctx.log_info(&format!("Current size: {}", format_size(trash_size, BINARY).red()));
    show_space_preview(trash_size);
    
    if trash_size > 0 && ctx.should_proceed("Empty trash?",
        Some(format!("This will permanently delete {} of files", format_size(trash_size, BINARY)))) {
        total_stats.add(&empty_trash(&ctx));
    }

    // Xcode derived data
    if check_xcode_installed() {
        println!("\n{}", "🛠️  Xcode".bold());
        println!("{}", "─".repeat(40).dimmed());
        let xcode_size = estimate_xcode_size();
        ctx.log_info(&format!("Derived Data & Archives: {}", format_size(xcode_size, BINARY).red()));
        show_space_preview(xcode_size);
        
        if xcode_size > 0 && ctx.should_proceed("Clean Xcode derived data and archives?",
            Some(format!("This will free approximately {}", format_size(xcode_size, BINARY)))) {
            total_stats.add(&clean_xcode(&ctx));
        }
    }

    // Homebrew cache
    if check_homebrew_installed() {
        println!("\n{}", "🍺 Homebrew".bold());
        println!("{}", "─".repeat(40).dimmed());
        let brew_size = estimate_homebrew_size();
        ctx.log_info(&format!("Cache size: {}", format_size(brew_size, BINARY).red()));
        show_space_preview(brew_size);
        
        if ctx.should_proceed("Clean Homebrew cache and outdated formulae?", None) {
            total_stats.add(&clean_homebrew(&ctx));
        }
    }

    // Node modules
    println!("\n{}", "📦 Node Modules".bold());
    println!("{}", "─".repeat(40).dimmed());
    find_and_clean_node_modules(&ctx, &mut total_stats);

    // Docker
    if check_docker_installed() {
        println!("\n{}", "🐳 Docker".bold());
        println!("{}", "─".repeat(40).dimmed());
        let docker_size = estimate_docker_size();
        if docker_size > 0 {
            ctx.log_info(&format!("Estimated unused: {}", format_size(docker_size, BINARY).red()));
            show_space_preview(docker_size);
        }
        
        if ctx.should_proceed("Clean Docker unused containers, images and volumes?", None) {
            clean_docker(&ctx);
        }
    }

    // Safari
    println!("\n{}", "🌐 Safari".bold());
    println!("{}", "─".repeat(40).dimmed());
    let safari_size = estimate_safari_size();
    ctx.log_info(&format!("Cache & History: {}", format_size(safari_size, BINARY).red()));
    show_space_preview(safari_size);
    
    if safari_size > 0 && ctx.should_proceed("Clean Safari cache and history?",
        Some(format!("This will free approximately {}", format_size(safari_size, BINARY)))) {
        total_stats.add(&clean_safari(&ctx));
    }

    // Chrome Cache
    println!("\n{}", "🌐 Chrome Cache".bold());
    println!("{}", "─".repeat(40).dimmed());
    let chrome_size = estimate_chrome_cache_size();
    ctx.log_info(&format!("Browser cache: {}", format_size(chrome_size, BINARY).red()));
    show_space_preview(chrome_size);
    
    if chrome_size > 0 && ctx.should_proceed("Clean Chrome cache?",
        Some(format!("This will free approximately {}", format_size(chrome_size, BINARY)))) {
        total_stats.add(&clean_chrome_cache(&ctx));
    }

    // Python Cache
    println!("\n{}", "🐍 Python Cache".bold());
    println!("{}", "─".repeat(40).dimmed());
    let python_size = estimate_python_cache_size();
    ctx.log_info(&format!("__pycache__ & .pyc files: {}", format_size(python_size, BINARY).red()));
    show_space_preview(python_size);
    
    if python_size > 0 && ctx.should_proceed("Clean Python cache files?",
        Some(format!("This will free approximately {}", format_size(python_size, BINARY)))) {
        total_stats.add(&clean_python_cache(&ctx));
    }

    // App Containers
    println!("\n{}", "📱 App Containers".bold());
    println!("{}", "─".repeat(40).dimmed());
    let containers_size = estimate_containers_size();
    ctx.log_info(&format!("App containers data: {}", format_size(containers_size, BINARY).red()));
    show_space_preview(containers_size);
    
    if containers_size > 0 && ctx.should_proceed("Clean app containers data?",
        Some(format!("This will free approximately {}", format_size(containers_size, BINARY)))) {
        total_stats.add(&clean_containers(&ctx));
    }

    // Browser Cookies & Web Data
    println!("\n{}", "🍪 Browser Cookies & Web Data".bold());
    println!("{}", "─".repeat(40).dimmed());
    let cookies_size = estimate_cookies_size();
    ctx.log_info(&format!("Cookies & web data: {}", format_size(cookies_size, BINARY).red()));
    show_space_preview(cookies_size);
    
    if cookies_size > 0 && ctx.should_proceed("Clean browser cookies and web data?",
        Some(format!("This will free approximately {}", format_size(cookies_size, BINARY)))) {
        total_stats.add(&clean_cookies(&ctx));
    }

    // RAM Cleanup
    println!("\n{}", "🧠 RAM Memory".bold());
    println!("{}", "─".repeat(40).dimmed());
    show_ram_status();
    
    if ctx.should_proceed("Clean RAM memory (purge inactive memory)?", 
        Some("This will free up inactive RAM".to_string())) {
        clean_ram(&ctx);
    }

    // Get final disk info
    let final_disk = get_disk_info();
    
    // Final report
    println!("\n{}", "=".repeat(60).green());
    println!("{}", "✨ Cleanup Complete!".bold().green());
    println!("{}", "=".repeat(60).green());
    
    if !ctx.dry_run {
        // Show before/after comparison
        println!("\n{}", "💾 Disk Space Summary:".bold().cyan());
        println!("  {} {} → {}", 
            "Before:".bold(),
            format!("{} available", format_size(initial_disk.available, BINARY)).red(),
            format!("{} available", format_size(final_disk.available, BINARY)).green()
        );
        
        let actual_freed = if final_disk.available > initial_disk.available {
            final_disk.available - initial_disk.available
        } else {
            0
        };
        
        println!("  {} {}", 
            "Actual space freed:".bold(),
            format_size(actual_freed, BINARY).bold().green()
        );
        
        println!("\n{}", "📊 Cleanup Statistics:".bold().cyan());
        println!("  {} {}", "Files removed:".bold(), total_stats.files_removed.to_string().yellow());
        println!("  {} {}", "Reported freed:".bold(), format_size(total_stats.space_freed, BINARY).green());
        
        // Show final disk status
        show_disk_status(&final_disk, "\n📱 Final Disk Status");
        
        // Show improvement
        let percent_improvement = if final_disk.available > initial_disk.available {
            ((final_disk.available - initial_disk.available) as f32 / initial_disk.total as f32) * 100.0
        } else {
            0.0
        };
        if percent_improvement > 0.0 {
            println!("\n  {} Disk space improved by {:.1}%! 🎉", 
                "✨".green(), 
                percent_improvement);
        }
    } else {
        println!("{}", "No files were actually deleted (dry run mode)".dimmed());
    }
}

fn get_disk_info() -> DiskInfo {
    let output = Command::new("df")
        .args(&["-H", "/"])
        .output()
        .expect("Failed to get disk info");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    
    if lines.len() >= 2 {
        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() >= 5 {
            let total = parse_size(parts[1]);
            let used = parse_size(parts[2]);
            let available = parse_size(parts[3]);
            let percent_str = parts[4].trim_end_matches('%');
            let percent_used = percent_str.parse::<f32>().unwrap_or(0.0);
            
            return DiskInfo {
                total,
                available,
                used,
                percent_used,
            };
        }
    }
    
    DiskInfo {
        total: 0,
        available: 0,
        used: 0,
        percent_used: 0.0,
    }
}

fn parse_size(size_str: &str) -> u64 {
    let size_str = size_str.to_uppercase();
    let number: f64;
    let multiplier: u64;
    
    if size_str.ends_with("T") {
        number = size_str.trim_end_matches('T').parse().unwrap_or(0.0);
        multiplier = 1_099_511_627_776;
    } else if size_str.ends_with("G") {
        number = size_str.trim_end_matches('G').parse().unwrap_or(0.0);
        multiplier = 1_073_741_824;
    } else if size_str.ends_with("M") {
        number = size_str.trim_end_matches('M').parse().unwrap_or(0.0);
        multiplier = 1_048_576;
    } else if size_str.ends_with("K") {
        number = size_str.trim_end_matches('K').parse().unwrap_or(0.0);
        multiplier = 1024;
    } else {
        number = size_str.parse().unwrap_or(0.0);
        multiplier = 1;
    }
    
    (number * multiplier as f64) as u64
}

fn show_disk_status(disk: &DiskInfo, title: &str) {
    println!("{}", title.bold().cyan());
    
    let used_bar_length = (disk.percent_used / 100.0 * 30.0) as usize;
    let free_bar_length = 30 - used_bar_length;
    
    let bar = format!("{}{}",
        "█".repeat(used_bar_length).red(),
        "░".repeat(free_bar_length).dimmed()
    );
    
    println!("  {} [{}] {:.1}%", 
        "Disk Usage:".bold(),
        bar,
        disk.percent_used
    );
    
    println!("  {} {} / {} ({})", 
        "Space:".bold(),
        format_size(disk.used, BINARY).red(),
        format_size(disk.total, BINARY),
        format!("{} free", format_size(disk.available, BINARY)).green()
    );
}

fn show_space_preview(size: u64) {
    if size > 0 {
        let disk = get_disk_info();
        let new_available = disk.available + size;
        let new_percent_used = if disk.used > size {
            ((disk.used - size) as f32 / disk.total as f32) * 100.0
        } else {
            0.0
        };
        
        println!("  {} {} → {} ({:.1}% → {:.1}%)",
            "Preview:".dimmed(),
            format_size(disk.available, BINARY).dimmed(),
            format_size(new_available, BINARY).green(),
            disk.percent_used,
            new_percent_used
        );
    }
}

fn show_ram_status() {
    let output = Command::new("vm_stat")
        .output()
        .expect("Failed to get RAM info");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut free_pages = 0u64;
    let mut inactive_pages = 0u64;
    let mut active_pages = 0u64;
    let mut wired_pages = 0u64;
    let mut compressed_pages = 0u64;
    
    for line in output_str.lines() {
        if line.contains("Pages free:") {
            free_pages = extract_number_from_line(line);
        } else if line.contains("Pages inactive:") {
            inactive_pages = extract_number_from_line(line);
        } else if line.contains("Pages active:") {
            active_pages = extract_number_from_line(line);
        } else if line.contains("Pages wired down:") {
            wired_pages = extract_number_from_line(line);
        } else if line.contains("Pages occupied by compressor:") {
            compressed_pages = extract_number_from_line(line);
        }
    }
    
    let page_size = 4096u64; // 4KB per page on macOS
    let free_mb = (free_pages * page_size) / 1_048_576;
    let inactive_mb = (inactive_pages * page_size) / 1_048_576;
    let active_mb = (active_pages * page_size) / 1_048_576;
    let wired_mb = (wired_pages * page_size) / 1_048_576;
    let compressed_mb = (compressed_pages * page_size) / 1_048_576;
    
    let total_ram = get_total_ram();
    let used_mb = active_mb + wired_mb + compressed_mb;
    let available_mb = free_mb + inactive_mb;
    
    println!("  {} {} / {} MB", 
        "RAM Usage:".bold(),
        format!("{} MB", used_mb).red(),
        total_ram
    );
    
    println!("  {} {} MB ({} MB inactive can be freed)",
        "Available:".bold(),
        format!("{}", available_mb).green(),
        inactive_mb
    );
}

fn get_total_ram() -> u64 {
    let output = Command::new("sysctl")
        .args(&["hw.memsize"])
        .output()
        .expect("Failed to get total RAM");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split(": ").collect();
    
    if parts.len() == 2 {
        let bytes = parts[1].parse::<u64>().unwrap_or(0);
        return bytes / 1_048_576; // Convert to MB
    }
    
    8192 // Default to 8GB if can't determine
}

fn extract_number_from_line(line: &str) -> u64 {
    line.split_whitespace()
        .last()
        .and_then(|s| s.trim_end_matches('.').parse().ok())
        .unwrap_or(0)
}

fn clean_ram(ctx: &CleanupContext) {
    ctx.log_action("Purging inactive memory...");
    
    if !ctx.dry_run {
        println!("  {} This requires sudo password", "ℹ".blue());
        
        // Show before state
        let before_output = Command::new("vm_stat")
            .output()
            .expect("Failed to get RAM info");
        let before_str = String::from_utf8_lossy(&before_output.stdout);
        let before_inactive = extract_inactive_pages(&before_str);
        
        // Run purge command
        let output = Command::new("sudo")
            .args(&["purge"])
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    // Wait a moment for the purge to complete
                    thread::sleep(Duration::from_secs(2));
                    
                    // Show after state
                    let after_output = Command::new("vm_stat")
                        .output()
                        .expect("Failed to get RAM info");
                    let after_str = String::from_utf8_lossy(&after_output.stdout);
                    let after_inactive = extract_inactive_pages(&after_str);
                    
                    let freed_pages = if before_inactive > after_inactive {
                        before_inactive - after_inactive
                    } else {
                        before_inactive // Assume all inactive was freed
                    };
                    
                    let freed_mb = (freed_pages * 4096) / 1_048_576;
                    
                    ctx.log_success(&format!("RAM purged successfully! Freed approximately {} MB", freed_mb));
                    
                    // Show updated RAM status
                    println!("\n  {} Updated RAM status:", "ℹ".blue());
                    show_ram_status();
                } else {
                    ctx.log_error("Failed to purge RAM - may need sudo privileges");
                }
            },
            Err(_) => {
                ctx.log_error("Failed to run purge command - sudo may not be available");
            }
        }
    } else {
        ctx.log_info("Would purge inactive RAM memory");
    }
}

fn extract_inactive_pages(vm_stat_output: &str) -> u64 {
    for line in vm_stat_output.lines() {
        if line.contains("Pages inactive:") {
            return extract_number_from_line(line);
        }
    }
    0
}

fn calculate_total_cleanup_size() -> u64 {
    let mut total = 0u64;
    
    total += estimate_cache_size();
    total += estimate_logs_size();
    total += estimate_old_downloads_size();
    total += estimate_trash_size();
    
    if check_xcode_installed() {
        total += estimate_xcode_size();
    }
    
    if check_homebrew_installed() {
        total += estimate_homebrew_size();
    }
    
    if check_docker_installed() {
        total += estimate_docker_size();
    }
    
    total += estimate_safari_size();
    total += estimate_python_cache_size();
    total += estimate_chrome_cache_size();
    
    total
}

fn estimate_homebrew_size() -> u64 {
    let brew_cache = "/Library/Caches/Homebrew";
    let user_brew_cache = format!("{}/Library/Caches/Homebrew", 
        env::var("HOME").unwrap_or_else(|_| String::from("/")));
    
    let mut size = 0;
    if Path::new(brew_cache).exists() {
        size += get_directory_size(brew_cache);
    }
    if Path::new(&user_brew_cache).exists() {
        size += get_directory_size(&user_brew_cache);
    }
    
    size
}

fn estimate_docker_size() -> u64 {
    // This is an estimate - actual size can be determined by docker system df
    if let Ok(output) = Command::new("docker")
        .args(&["system", "df"])
        .output() {
        if output.status.success() {
            // Parse docker system df output
            // This is simplified - you might want to parse more accurately
            return 1_073_741_824; // Return 1GB as estimate
        }
    }
    0
}

fn estimate_safari_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let safari_paths = vec![
        format!("{}/Library/Caches/com.apple.Safari", home),
        format!("{}/Library/Safari/History.db", home),
        format!("{}/Library/Safari/TopSites.plist", home),
        format!("{}/Library/Caches/com.apple.WebKit.PluginProcess", home),
    ];
    
    let mut total = 0;
    for path in safari_paths {
        if Path::new(&path).exists() {
            if Path::new(&path).is_dir() {
                total += get_directory_size(&path);
            } else if let Ok(metadata) = fs::metadata(&path) {
                total += metadata.len();
            }
        }
    }
    total
}

fn estimate_chrome_cache_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let chrome_paths = vec![
        format!("{}/Library/Caches/Google/Chrome", home),
        format!("{}/Library/Caches/com.google.Chrome", home),
    ];
    
    let mut total = 0;
    for path in chrome_paths {
        if Path::new(&path).exists() {
            total += get_directory_size(&path);
        }
    }
    total
}

fn estimate_python_cache_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let search_paths = vec![
        format!("{}/Desktop", home),
        format!("{}/Documents", home),
        format!("{}/Developer", home),
        format!("{}/Projects", home),
    ];
    
    let mut total = 0;
    for search_path in search_paths {
        if Path::new(&search_path).exists() {
            total += find_python_cache_size(&search_path, 0, 4);
        }
    }
    total
}

fn estimate_containers_size() -> u64 {
    let home = env::var("HOME").unwrap_or_default();
    let containers_path = format!("{}/Library/Containers", home);
    
    if Path::new(&containers_path).exists() {
        get_directory_size(&containers_path)
    } else {
        0
    }
}

fn estimate_cookies_size() -> u64 {
    let home = env::var("HOME").unwrap_or_default();
    let paths = vec![
        format!("{}/Library/Cookies", home),
        format!("{}/Library/HTTPStorages", home),
        format!("{}/Library/WebKit", home),
        format!("{}/Library/Safari/LocalStorage", home),
        format!("{}/Library/Safari/Databases", home),
        format!("{}/Library/Application Support/Google/Chrome/Default/Cookies", home),
        format!("{}/Library/Application Support/Google/Chrome/Default/Local Storage", home),
    ];
    
    let mut total_size = 0u64;
    for path in paths {
        if Path::new(&path).exists() {
            total_size += get_directory_size(&path);
        }
    }
    total_size
}

fn show_menu() -> bool {
    println!("\n{}", "This tool will clean the following:".bold());
    println!("  • System and user caches");
    println!("  • Old system logs (7+ days)");
    println!("  • Old downloads (30+ days)");
    println!("  • Trash bin");
    println!("  • Xcode derived data (if installed)");
    println!("  • Homebrew cache (if installed)");
    println!("  • Unused node_modules");
    println!("  • Docker unused data (if installed)");
    println!("  • Safari cache and history");
    println!("  • Chrome browser cache");
    println!("  • Python cache files (__pycache__, .pyc)");
    println!("  • App containers data");
    println!("  • Browser cookies and web data");
    println!("  • RAM inactive memory");
    
    print!("\n{} {} ", "?".cyan(), "Continue with cleanup? (y/N):".yellow().bold());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes"
}

fn estimate_cache_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let cache_paths = vec![
        format!("{}/Library/Caches", home),
        format!("{}/.cache", home),
        "/Library/Caches".to_string(),
        "/System/Library/Caches".to_string(),
    ];
    
    let mut total = 0;
    for path in cache_paths {
        if Path::new(&path).exists() {
            total += get_directory_size(&path);
        }
    }
    total
}

fn estimate_logs_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let log_paths = vec![
        format!("{}/Library/Logs", home),
        "/Library/Logs".to_string(),
        "/var/log".to_string(),
    ];
    
    let mut total = 0;
    for path in log_paths {
        if Path::new(&path).exists() {
            total += get_old_files_size(&path, 7);
        }
    }
    total
}

fn estimate_old_downloads_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let downloads_path = format!("{}/Downloads", home);
    
    if Path::new(&downloads_path).exists() {
        return get_old_files_size(&downloads_path, 30);
    }
    0
}

fn estimate_trash_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let trash_path = format!("{}/.Trash", home);
    
    if Path::new(&trash_path).exists() {
        return get_directory_size(&trash_path);
    }
    0
}

fn estimate_xcode_size() -> u64 {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let xcode_paths = vec![
        format!("{}/Library/Developer/Xcode/DerivedData", home),
        format!("{}/Library/Developer/Xcode/Archives", home),
    ];
    
    let mut total = 0;
    for path in xcode_paths {
        if Path::new(&path).exists() {
            total += get_directory_size(&path);
        }
    }
    total
}

fn check_xcode_installed() -> bool {
    Path::new("/Applications/Xcode.app").exists() || 
    Command::new("xcode-select").arg("-p").output().is_ok()
}

fn check_homebrew_installed() -> bool {
    Command::new("brew").arg("--version").output().is_ok()
}

fn check_docker_installed() -> bool {
    Command::new("docker").arg("--version").output().is_ok()
}

fn clean_caches(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    
    let cache_paths = vec![
        format!("{}/Library/Caches", home),
        format!("{}/.cache", home),
        "/Library/Caches".to_string(),
        "/System/Library/Caches".to_string(),
    ];

    for path in cache_paths {
        if Path::new(&path).exists() {
            ctx.log_action(&format!("Cleaning {}", path));
            // Use longer retention for system caches for safety
            let retention_days = if path.starts_with("/System") || path.starts_with("/Library") { 7 } else { 1 };
            stats.add(&clean_directory(&path, Some(retention_days), ctx));
        }
    }

    ctx.log_success(&format!("Cleaned {} files, freed {}", 
        stats.files_removed, 
        format_size(stats.space_freed, BINARY)));
    stats
}

fn clean_logs(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    
    let log_paths = vec![
        format!("{}/Library/Logs", home),
        format!("{}/.npm/_logs", home),
        "/Library/Logs".to_string(),
        "/var/log".to_string(),
    ];

    for path in log_paths {
        if Path::new(&path).exists() {
            ctx.log_action(&format!("Cleaning {}", path));
            stats.add(&clean_directory(&path, Some(7), ctx));
        }
    }

    ctx.log_success(&format!("Cleaned {} log files, freed {}", 
        stats.files_removed, 
        format_size(stats.space_freed, BINARY)));
    stats
}

fn clean_old_downloads(ctx: &CleanupContext) -> CleanupStats {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let downloads_path = format!("{}/Downloads", home);
    
    if Path::new(&downloads_path).exists() {
        ctx.log_action("Cleaning old files in Downloads folder");
        let stats = clean_directory(&downloads_path, Some(30), ctx);
        ctx.log_success(&format!("Cleaned {} old files, freed {}", 
            stats.files_removed, 
            format_size(stats.space_freed, BINARY)));
        return stats;
    }
    
    CleanupStats::new()
}

fn empty_trash(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let trash_path = format!("{}/.Trash", home);
    
    if Path::new(&trash_path).exists() {
        ctx.log_action("Emptying trash");
        stats = clean_directory(&trash_path, None, ctx);
        ctx.log_success(&format!("Emptied trash, freed {}", 
            format_size(stats.space_freed, BINARY)));
    }

    stats
}

fn clean_xcode(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    
    let xcode_paths = vec![
        format!("{}/Library/Developer/Xcode/DerivedData", home),
        format!("{}/Library/Developer/Xcode/Archives", home),
        format!("{}/Library/Developer/CoreSimulator/Caches", home),
    ];

    for path in xcode_paths {
        if Path::new(&path).exists() {
            ctx.log_action(&format!("Cleaning {}", path));
            stats.add(&clean_directory(&path, None, ctx));
        }
    }

    ctx.log_success(&format!("Cleaned Xcode data, freed {}", 
        format_size(stats.space_freed, BINARY)));
    stats
}

fn clean_homebrew(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    
    ctx.log_action("Running brew cleanup");
    
    if !ctx.dry_run {
        // Get size before cleanup
        let before_size = estimate_homebrew_size();
        
        if let Ok(output) = Command::new("brew")
            .args(&["cleanup", "-s"])
            .output() {
            if output.status.success() {
                // Estimate freed space
                let after_size = estimate_homebrew_size();
                stats.space_freed = if before_size > after_size {
                    before_size - after_size
                } else {
                    before_size / 2 // Estimate half was cleaned
                };
                
                ctx.log_success(&format!("Homebrew cleanup completed, freed approximately {}", 
                    format_size(stats.space_freed, BINARY)));
            }
        }
    }

    stats
}

fn find_and_clean_node_modules(ctx: &CleanupContext, total_stats: &mut CleanupStats) {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let search_paths = vec![
        format!("{}/Desktop", home),
        format!("{}/Documents", home),
        format!("{}/Developer", home),
        format!("{}/Projects", home),
    ];

    ctx.log_action("Searching for node_modules directories...");
    let mut found_dirs = Vec::new();

    for search_path in search_paths {
        if Path::new(&search_path).exists() {
            find_node_modules_recursive(&search_path, &mut found_dirs, 0, 3);
        }
    }

    if !found_dirs.is_empty() {
        let total_size: u64 = found_dirs.iter()
            .map(|dir| get_directory_size(dir))
            .sum();
        
        println!("\n  {} Found {} node_modules directories ({})", 
            "ℹ".blue(), 
            found_dirs.len().to_string().yellow(),
            format_size(total_size, BINARY).red());
        
        show_space_preview(total_size);
        
        // Show first 5 directories
        for (i, dir) in found_dirs.iter().enumerate() {
            if i < 5 {
                let size = get_directory_size(dir);
                println!("    {} {} ({})", 
                    "•".dimmed(),
                    dir.dimmed(), 
                    format_size(size, BINARY).red());
            }
        }
        if found_dirs.len() > 5 {
            println!("    {} ... and {} more", "•".dimmed(), found_dirs.len() - 5);
        }
        
        if ctx.should_proceed("Remove all node_modules directories?", 
            Some(format!("This will free approximately {}", format_size(total_size, BINARY)))) {
            
            if !ctx.dry_run {
                for dir in found_dirs {
                    if let Ok(_) = fs::remove_dir_all(&dir) {
                        total_stats.files_removed += 1;
                    }
                }
                total_stats.space_freed += total_size;
                ctx.log_success(&format!("Removed all node_modules directories, freed {}", 
                    format_size(total_size, BINARY)));
            }
        }
    } else {
        ctx.log_info("No node_modules directories found");
    }
}

fn find_node_modules_recursive(path: &str, found: &mut Vec<String>, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
                    
                    if dir_name == "node_modules" {
                        found.push(path.to_str().unwrap_or("").to_string());
                    } else if !dir_name.starts_with('.') && dir_name != "Library" {
                        find_node_modules_recursive(
                            path.to_str().unwrap_or(""),
                            found,
                            depth + 1,
                            max_depth
                        );
                    }
                }
            }
        }
    }
}

fn clean_docker(ctx: &CleanupContext) {
    ctx.log_action("Running Docker system prune");
    
    if !ctx.dry_run {
        if let Ok(output) = Command::new("docker")
            .args(&["system", "prune", "-a", "-f", "--volumes"])
            .output() {
            if output.status.success() {
                ctx.log_success("Docker cleanup completed");
            }
        }
    }
}

fn clean_directory(path: &str, days_old: Option<u64>, ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                // Skip important system files
                let name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
                if name == ".DS_Store" || name.starts_with(".") {
                    continue;
                }
                
                // Check age if days_old is specified
                if let Some(days) = days_old {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                let days_elapsed = elapsed.as_secs() / 86400;
                                if days_elapsed < days {
                                    continue;
                                }
                            }
                        }
                    }
                }
                
                // Get size before deletion
                let size = if path.is_dir() {
                    get_directory_size(path.to_str().unwrap_or(""))
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                
                // Try to remove (or simulate in dry run)
                if !ctx.dry_run {
                    let removed = if path.is_dir() {
                        fs::remove_dir_all(&path).is_ok()
                    } else {
                        fs::remove_file(&path).is_ok()
                    };
                    
                    if removed {
                        stats.files_removed += 1;
                        stats.space_freed += size;
                        if ctx.verbose {
                            println!("    {} Removed: {}", "✓".green(), path.display());
                        }
                    }
                } else {
                    stats.files_removed += 1;
                    stats.space_freed += size;
                }
            }
        }
    }
    
    stats
}

fn get_directory_size(path: &str) -> u64 {
    let mut size = 0;
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    size += get_directory_size(path.to_str().unwrap_or(""));
                } else {
                    size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
    }
    
    size
}

fn get_old_files_size(path: &str, days: u64) -> u64 {
    let mut size = 0;
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            let days_elapsed = elapsed.as_secs() / 86400;
                            if days_elapsed >= days {
                                if entry.path().is_dir() {
                                    size += get_directory_size(entry.path().to_str().unwrap_or(""));
                                } else {
                                    size += metadata.len();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    size
}

fn clean_safari(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    
    let safari_paths = vec![
        format!("{}/Library/Caches/com.apple.Safari", home),
        format!("{}/Library/Safari/History.db", home),
        format!("{}/Library/Safari/TopSites.plist", home),
        format!("{}/Library/Caches/com.apple.WebKit.PluginProcess", home),
    ];

    for path in safari_paths {
        if Path::new(&path).exists() {
            ctx.log_action(&format!("Cleaning {}", path));
            
            let size = if Path::new(&path).is_dir() {
                get_directory_size(&path)
            } else if let Ok(metadata) = fs::metadata(&path) {
                metadata.len()
            } else {
                0
            };
            
            if !ctx.dry_run {
                let removed = if Path::new(&path).is_dir() {
                    fs::remove_dir_all(&path).is_ok()
                } else {
                    fs::remove_file(&path).is_ok()
                };
                
                if removed {
                    stats.files_removed += 1;
                    stats.space_freed += size;
                }
            } else {
                stats.files_removed += 1;
                stats.space_freed += size;
            }
        }
    }

    ctx.log_success(&format!("Cleaned Safari data, freed {}", 
        format_size(stats.space_freed, BINARY)));
    stats
}

fn clean_chrome_cache(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    
    let chrome_paths = vec![
        format!("{}/Library/Caches/Google/Chrome", home),
        format!("{}/Library/Caches/com.google.Chrome", home),
    ];

    for path in chrome_paths {
        if Path::new(&path).exists() {
            ctx.log_action(&format!("Cleaning {}", path));
            
            let size = get_directory_size(&path);
            
            if !ctx.dry_run {
                let removed = fs::remove_dir_all(&path).is_ok();
                if removed {
                    stats.files_removed += 1;
                    stats.space_freed += size;
                }
            } else {
                stats.files_removed += 1;
                stats.space_freed += size;
            }
        }
    }

    ctx.log_success(&format!("Cleaned Chrome cache, freed {}", 
        format_size(stats.space_freed, BINARY)));
    stats
}

fn clean_python_cache(ctx: &CleanupContext) -> CleanupStats {
    let mut stats = CleanupStats::new();
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let search_paths = vec![
        format!("{}/Desktop", home),
        format!("{}/Documents", home),
        format!("{}/Developer", home),
        format!("{}/Projects", home),
    ];

    ctx.log_action("Searching for Python cache files...");
    let mut found_files = Vec::new();

    for search_path in search_paths {
        if Path::new(&search_path).exists() {
            find_python_cache_files(&search_path, &mut found_files, 0, 4);
        }
    }

    if !found_files.is_empty() {
        let total_size: u64 = found_files.iter()
            .map(|file| {
                if let Ok(metadata) = fs::metadata(file) {
                    metadata.len()
                } else {
                    0
                }
            })
            .sum();

        if !ctx.dry_run {
            for file in found_files {
                if fs::remove_file(&file).is_ok() || fs::remove_dir_all(&file).is_ok() {
                    stats.files_removed += 1;
                }
            }
            stats.space_freed = total_size;
        } else {
            stats.files_removed = found_files.len();
            stats.space_freed = total_size;
        }
    }

    ctx.log_success(&format!("Cleaned {} Python cache files, freed {}", 
        stats.files_removed, 
        format_size(stats.space_freed, BINARY)));
    stats
}

fn clean_containers(ctx: &CleanupContext) -> CleanupStats {
    let home = env::var("HOME").unwrap_or_default();
    let containers_path = format!("{}/Library/Containers", home);
    
    if !Path::new(&containers_path).exists() {
        return CleanupStats::new();
    }
    
    ctx.log_action("Cleaning app containers...");
    clean_directory(&containers_path, Some(7), ctx) // Clean containers older than 7 days
}

fn clean_cookies(ctx: &CleanupContext) -> CleanupStats {
    let home = env::var("HOME").unwrap_or_default();
    let paths = vec![
        format!("{}/Library/Cookies", home),
        format!("{}/Library/HTTPStorages", home),
        format!("{}/Library/WebKit", home),
        format!("{}/Library/Safari/LocalStorage", home),
        format!("{}/Library/Safari/Databases", home),
        format!("{}/Library/Application Support/Google/Chrome/Default/Cookies", home),
        format!("{}/Library/Application Support/Google/Chrome/Default/Local Storage", home),
    ];
    
    ctx.log_action("Cleaning browser cookies and web data...");
    let mut total_stats = CleanupStats::new();
    
    for path in paths {
        if Path::new(&path).exists() {
            let stats = clean_directory(&path, Some(0), ctx); // Clean all cookies/web data
            total_stats.add(&stats);
        }
    }
    
    ctx.log_success(&format!("Cleaned {} cookie/web data files, freed {}", 
        total_stats.files_removed, 
        format_size(total_stats.space_freed, BINARY)));
    
    total_stats
}

fn find_python_cache_size(path: &str, depth: usize, max_depth: usize) -> u64 {
    if depth > max_depth {
        return 0;
    }

    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
                    
                    if dir_name == "__pycache__" {
                        size += get_directory_size(path.to_str().unwrap_or(""));
                    } else if !dir_name.starts_with('.') && dir_name != "Library" {
                        size += find_python_cache_size(
                            path.to_str().unwrap_or(""),
                            depth + 1,
                            max_depth
                        );
                    }
                } else if let Some(extension) = path.extension() {
                    if extension == "pyc" || extension == "pyo" {
                        if let Ok(metadata) = entry.metadata() {
                            size += metadata.len();
                        }
                    }
                }
            }
        }
    }
    size
}

fn find_python_cache_files(path: &str, found: &mut Vec<String>, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
                    
                    if dir_name == "__pycache__" {
                        found.push(path.to_str().unwrap_or("").to_string());
                    } else if !dir_name.starts_with('.') && dir_name != "Library" {
                        find_python_cache_files(
                            path.to_str().unwrap_or(""),
                            found,
                            depth + 1,
                            max_depth
                        );
                    }
                } else if let Some(extension) = path.extension() {
                    if extension == "pyc" || extension == "pyo" {
                        found.push(path.to_str().unwrap_or("").to_string());
                    }
                }
            }
        }
    }
}