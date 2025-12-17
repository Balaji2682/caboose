use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum FrontendFramework {
    Vite,           // Vite (React, Vue, Svelte, etc.)
    NextJs,         // Next.js
    CreateReactApp, // Create React App
    VueCli,         // Vue CLI
    Angular,        // Angular CLI
    NuxtJs,         // Nuxt.js (Vue)
    SvelteKit,      // SvelteKit
    Remix,          // Remix
    Astro,          // Astro
}

impl FrontendFramework {
    /// Returns the default dev server command for each framework.
    /// These commands use the package manager (npm/yarn/pnpm/bun) which gets
    /// replaced by generate_procfile_entry() based on detected lock files.
    pub fn dev_command(&self) -> String {
        match self {
            FrontendFramework::Vite => "npm run dev".to_string(),
            FrontendFramework::NextJs => "npm run dev".to_string(),
            FrontendFramework::CreateReactApp => "npm start".to_string(),
            FrontendFramework::VueCli => "npm run serve".to_string(),
            // Angular: use npm start (runs "ng serve" from package.json via node_modules/.bin)
            // This ensures the locally installed Angular CLI is used instead of requiring
            // a global @angular/cli installation or ng in PATH.
            FrontendFramework::Angular => "npm start".to_string(),
            FrontendFramework::NuxtJs => "npm run dev".to_string(),
            FrontendFramework::SvelteKit => "npm run dev".to_string(),
            FrontendFramework::Remix => "npm run dev".to_string(),
            FrontendFramework::Astro => "npm run dev".to_string(),
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            FrontendFramework::Vite => 5173,
            FrontendFramework::NextJs => 3000,
            FrontendFramework::CreateReactApp => 3000,
            FrontendFramework::VueCli => 8080,
            FrontendFramework::Angular => 4200,
            FrontendFramework::NuxtJs => 3000,
            FrontendFramework::SvelteKit => 5173,
            FrontendFramework::Remix => 3000,
            FrontendFramework::Astro => 3000,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FrontendFramework::Vite => "Vite",
            FrontendFramework::NextJs => "Next.js",
            FrontendFramework::CreateReactApp => "Create React App",
            FrontendFramework::VueCli => "Vue CLI",
            FrontendFramework::Angular => "Angular",
            FrontendFramework::NuxtJs => "Nuxt.js",
            FrontendFramework::SvelteKit => "SvelteKit",
            FrontendFramework::Remix => "Remix",
            FrontendFramework::Astro => "Astro",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrontendApp {
    pub detected: bool,
    pub framework: Option<FrontendFramework>,
    pub path: String,
    pub package_manager: PackageManager,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    pub fn run_command(&self) -> &str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
        }
    }

    pub fn detect(frontend_path: &str) -> Self {
        // Check for lock files to determine package manager
        if Path::new(&format!("{}/bun.lockb", frontend_path)).exists() {
            return PackageManager::Bun;
        }
        if Path::new(&format!("{}/pnpm-lock.yaml", frontend_path)).exists() {
            return PackageManager::Pnpm;
        }
        if Path::new(&format!("{}/yarn.lock", frontend_path)).exists() {
            return PackageManager::Yarn;
        }
        PackageManager::Npm
    }
}

impl FrontendApp {
    pub fn detect() -> Self {
        Self::detect_with_config(None)
    }

    pub fn detect_with_config(config_path: Option<&str>) -> Self {
        // If explicit path provided, try that first
        if let Some(path) = config_path {
            if let Some(app) = Self::detect_in_path(path) {
                return app;
            }
        }

        // Common frontend directory names
        let frontend_dirs = [
            "frontend",
            "client",
            "web",
            "app",
            "ui",
            "www",
            "../frontend", // Sibling directory
            "../client",
            "../web",
        ];

        for dir in &frontend_dirs {
            if let Some(app) = Self::detect_in_path(dir) {
                return app;
            }
        }

        FrontendApp {
            detected: false,
            framework: None,
            path: String::new(),
            package_manager: PackageManager::Npm,
        }
    }

    fn detect_in_path(path: &str) -> Option<FrontendApp> {
        let package_json = format!("{}/package.json", path);

        if !Path::new(&package_json).exists() {
            return None;
        }

        // Read package.json to detect framework
        let framework = Self::detect_framework(path);

        if framework.is_some() {
            let package_manager = PackageManager::detect(path);

            return Some(FrontendApp {
                detected: true,
                framework,
                path: path.to_string(),
                package_manager,
            });
        }

        None
    }

    fn detect_framework(path: &str) -> Option<FrontendFramework> {
        // Check for framework-specific config files and package.json dependencies

        // Next.js
        if Path::new(&format!("{}/next.config.js", path)).exists()
            || Path::new(&format!("{}/next.config.mjs", path)).exists()
            || Path::new(&format!("{}/next.config.ts", path)).exists()
        {
            return Some(FrontendFramework::NextJs);
        }

        // Nuxt.js
        if Path::new(&format!("{}/nuxt.config.js", path)).exists()
            || Path::new(&format!("{}/nuxt.config.ts", path)).exists()
        {
            return Some(FrontendFramework::NuxtJs);
        }

        // SvelteKit
        if Path::new(&format!("{}/svelte.config.js", path)).exists() {
            return Some(FrontendFramework::SvelteKit);
        }

        // Remix
        if Path::new(&format!("{}/remix.config.js", path)).exists() {
            return Some(FrontendFramework::Remix);
        }

        // Astro
        if Path::new(&format!("{}/astro.config.mjs", path)).exists()
            || Path::new(&format!("{}/astro.config.js", path)).exists()
        {
            return Some(FrontendFramework::Astro);
        }

        // Vite
        if Path::new(&format!("{}/vite.config.js", path)).exists()
            || Path::new(&format!("{}/vite.config.ts", path)).exists()
        {
            return Some(FrontendFramework::Vite);
        }

        // Angular
        if Path::new(&format!("{}/angular.json", path)).exists() {
            return Some(FrontendFramework::Angular);
        }

        // Vue CLI
        if Path::new(&format!("{}/vue.config.js", path)).exists() {
            return Some(FrontendFramework::VueCli);
        }

        // Create React App (check for react-scripts in package.json)
        if let Ok(content) = std::fs::read_to_string(format!("{}/package.json", path)) {
            if content.contains("react-scripts") {
                return Some(FrontendFramework::CreateReactApp);
            }
        }

        None
    }

    pub fn generate_procfile_entry(&self, dev_command_override: Option<&str>) -> Option<String> {
        if !self.detected {
            return None;
        }

        // Use custom dev_command from config if provided
        let command = if let Some(custom_cmd) = dev_command_override {
            custom_cmd.to_string()
        } else {
            let framework = self.framework.as_ref()?;
            let pm = self.package_manager.run_command();

            // Get the base command
            let mut command = framework.dev_command();

            // Replace npm with actual package manager
            if command.starts_with("npm ") {
                command = command.replace("npm", pm);
            }

            command
        };

        // Change to frontend directory and run command
        Some(format!("cd {} && {}", self.path, command))
    }
}

// Frontend log event types
#[derive(Debug, Clone)]
pub enum FrontendLogEvent {
    ServerStart {
        port: u16,
    },
    CompileStart,
    CompileSuccess {
        duration: f64,
    },
    CompileError {
        message: String,
    },
    HotModuleReplacement {
        file: String,
    },
    ApiRequest {
        method: String,
        path: String,
        status: Option<u16>,
    },
    BuildWarning {
        message: String,
    },
    Error {
        message: String,
    },
}

pub struct FrontendLogParser;

impl FrontendLogParser {
    pub fn parse_line(line: &str) -> Option<FrontendLogEvent> {
        // Vite patterns
        if line.contains("Local:") && line.contains("http://") {
            if let Some(port) = Self::extract_port(line) {
                return Some(FrontendLogEvent::ServerStart { port });
            }
        }

        // Next.js patterns
        if line.contains("ready - started server on") || line.contains("Ready in") {
            if let Some(port) = Self::extract_port(line) {
                return Some(FrontendLogEvent::ServerStart { port });
            }
        }

        // Compile start
        if line.contains("Compiling") || line.contains("building...") {
            return Some(FrontendLogEvent::CompileStart);
        }

        // Compile success
        if line.contains("Compiled successfully")
            || line.contains("✓ Compiled")
            || line.contains("built in")
        {
            if let Some(duration) = Self::extract_build_duration(line) {
                return Some(FrontendLogEvent::CompileSuccess { duration });
            }
            return Some(FrontendLogEvent::CompileSuccess { duration: 0.0 });
        }

        // HMR
        if line.contains("hmr update")
            || line.contains("hot updated")
            || line.contains("[vite] hmr")
        {
            if let Some(file) = Self::extract_file_path(line) {
                return Some(FrontendLogEvent::HotModuleReplacement { file });
            }
        }

        // Errors
        if line.contains("ERROR") || line.contains("Failed to compile") || line.contains("✘") {
            return Some(FrontendLogEvent::Error {
                message: line.to_string(),
            });
        }

        // Warnings
        if line.contains("WARNING") || line.contains("⚠") {
            return Some(FrontendLogEvent::BuildWarning {
                message: line.to_string(),
            });
        }

        None
    }

    fn extract_port(line: &str) -> Option<u16> {
        // Extract port from URLs like "http://localhost:5173"
        if let Some(pos) = line.find("localhost:") {
            let after = &line[pos + 10..];
            if let Some(end) = after.find(|c: char| !c.is_numeric()) {
                return after[..end].parse().ok();
            }
        }

        // Extract from "port 3000" pattern
        if let Some(pos) = line.find("port ") {
            let after = &line[pos + 5..];
            if let Some(end) = after.find(|c: char| !c.is_numeric()) {
                return after[..end].parse().ok();
            }
        }

        None
    }

    fn extract_build_duration(line: &str) -> Option<f64> {
        // Pattern: "in 123ms" or "123ms"
        if let Some(pos) = line.find("ms") {
            let before = &line[..pos];
            if let Some(num_start) = before.rfind(|c: char| !c.is_numeric() && c != '.') {
                let num_str = &before[num_start + 1..];
                return num_str.parse().ok();
            }
        }

        // Pattern: "in 1.23s"
        if let Some(pos) = line.find("s ") {
            let before = &line[..pos];
            if let Some(num_start) = before.rfind(|c: char| !c.is_numeric() && c != '.') {
                let num_str = &before[num_start + 1..];
                if let Ok(seconds) = num_str.parse::<f64>() {
                    return Some(seconds * 1000.0);
                }
            }
        }

        None
    }

    fn extract_file_path(line: &str) -> Option<String> {
        // Extract file paths from HMR messages
        // Common patterns: "src/App.tsx updated" or "[vite] hmr update /src/App.tsx"

        for part in line.split_whitespace() {
            if part.ends_with(".ts")
                || part.ends_with(".tsx")
                || part.ends_with(".js")
                || part.ends_with(".jsx")
                || part.ends_with(".vue")
                || part.ends_with(".svelte")
            {
                return Some(part.to_string());
            }
        }

        None
    }
}
