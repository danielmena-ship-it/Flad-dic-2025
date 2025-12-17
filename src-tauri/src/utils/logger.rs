use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct Logger {
    log_file: PathBuf,
}

impl Logger {
    pub fn new() -> Result<Self, std::io::Error> {
        let app_dir = if let Some(local_dir) = dirs::data_local_dir() {
            local_dir.join("sistema-piloto-cont-mant")
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("data")
        };
        
        std::fs::create_dir_all(&app_dir)?;
        let log_file = app_dir.join("debug.log");
        
        Ok(Logger { log_file })
    }
    
    pub fn log(&self, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{}] {}\n", timestamp, message);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
        
        // También imprimir a consola si está disponible
        println!("{}", log_line.trim());
    }
    
    pub fn error(&self, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{}] ❌ ERROR: {}\n", timestamp, message);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
        
        eprintln!("{}", log_line.trim());
    }
}
