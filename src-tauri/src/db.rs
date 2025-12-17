use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;

#[derive(Clone)]
pub struct DbState {
    pub pool: Arc<Pool<Sqlite>>,
}

impl DbState {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Intentar data_local_dir, fallback a portable
        let app_dir = if let Some(local_dir) = dirs::data_local_dir() {
            local_dir.join("sistema-piloto-cont-mant")
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("data")
        };
        
        // Crear directorio con manejo de errores
        if let Err(e) = std::fs::create_dir_all(&app_dir) {
            eprintln!("⚠️ Error creando directorio {:?}: {}", app_dir, e);
            return Err(sqlx::Error::Io(e));
        }
        
        let db_path = app_dir.join("database.db");
        
        let pool = SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .busy_timeout(std::time::Duration::from_secs(30))
        )
        .await?;
        
        // ✅ Windows: Configurar checkpoint agresivo para evitar WAL crecimiento
        #[cfg(target_os = "windows")]
        {
            let _ = sqlx::query("PRAGMA wal_autocheckpoint=100")
                .execute(&pool)
                .await;
        }
        
        // SSOL: Cargar schema único
        let schema = include_str!("../sql/schema.sql");
        
        // Dividir statements respetando bloques BEGIN...END
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_trigger = false;
        
        for line in schema.lines() {
            let trimmed = line.trim();
            current_statement.push_str(line);
            current_statement.push('\n');
            
            // Detectar inicio de trigger
            if trimmed.to_uppercase().contains("BEGIN") {
                in_trigger = true;
            }
            
            // Detectar fin de statement
            if trimmed.ends_with(';') {
                if !in_trigger || trimmed == "END;" {
                    statements.push(current_statement.clone());
                    current_statement.clear();
                    in_trigger = false;
                }
            }
        }
        
        // Ejecutar cada statement
        for statement in statements {
            let statement = statement.trim();
            if !statement.is_empty() {
                match sqlx::query(statement).execute(&pool).await {
                    Ok(_) => {},
                    Err(e) => {
                        let err_msg = e.to_string();
                        // Ignorar error de columna duplicada (migraciones)
                        if err_msg.contains("duplicate column name") {
                            eprintln!("⚠️ [DB] Columna ya existe (esperado en migración): {}", err_msg);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
        }
        
        Ok(DbState { pool: Arc::new(pool) })
    }
}

// Tipos de datos
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Jardin {
    pub id: i64,
    pub codigo: String,
    pub nombre: String,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Partida {
    pub id: i64,
    pub item: String,
    pub partida: String,
    pub unidad: Option<String>,
    pub precio_unitario: f64,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Requerimiento {
    pub id: i64,
    pub jardin_codigo: String,
    pub recinto: Option<String>,
    pub partida_item: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub precio_total: f64,
    pub fecha_inicio: String,
    pub fecha_registro: String,
    pub estado: String,
    pub ot_id: Option<i64>,
    pub informe_pago_id: Option<i64>,
    pub fecha_recepcion: Option<String>,
    pub plazo_dias: i32,
    pub plazo_adicional: i32,
    pub plazo_total: i32,
    pub fecha_limite: Option<String>,
    pub multa: f64,
    pub a_pago: Option<f64>,
    pub utilidades: Option<f64>,
    pub iva: Option<f64>,
    pub total_linea: Option<f64>,
    pub descripcion: Option<String>,
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// Struct para consultas enriquecidas con JOINs
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RequerimientoEnriquecido {
    pub id: i64,
    pub jardin_codigo: String,
    pub recinto: Option<String>,
    pub partida_item: String,
    pub partida_nombre: Option<String>,
    pub partida_unidad: Option<String>,
    pub precio_unitario: Option<f64>,
    pub cantidad: f64,
    pub precio_total: f64,
    pub fecha_inicio: String,
    pub plazo_dias: i32,
    pub plazo_adicional: i32,
    pub plazo_total: i32,
    pub fecha_limite: Option<String>,
    pub fecha_registro: String,
    pub fecha_recepcion: Option<String>,
    pub dias_atraso: i32,
    pub multa: f64,
    pub a_pago: Option<f64>,
    pub utilidades: Option<f64>,
    pub iva: Option<f64>,
    pub total_linea: Option<f64>,
    pub descripcion: Option<String>,
    pub observaciones: Option<String>,
    pub estado: String,
    pub ot_id: Option<i64>,
    pub ot_codigo: Option<String>,
    pub informe_pago_id: Option<i64>,
    pub informe_pago_codigo: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Recinto {
    pub id: i64,
    pub jardin_codigo: String,
    pub nombre: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajo {
    pub id: i64,
    pub codigo: String,
    pub jardin_codigo: String,
    pub fecha_creacion: String,
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InformePago {
    pub id: i64,
    pub codigo: String,
    pub jardin_codigo: String,
    pub fecha_creacion: String,
    pub neto: f64,
    pub utilidades: f64,
    pub iva: f64,
    pub total_pagar: f64,
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InformePagoEnriquecido {
    pub id: i64,
    pub codigo: String,
    pub jardin_codigo: String,
    pub jardin_nombre: Option<String>,
    pub fecha_creacion: String,
    pub neto: f64,
    pub utilidades: f64,
    pub iva: f64,
    pub total_pagar: f64,
    pub cantidad_requerimientos: i64,
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Configuracion {
    pub id: i64,
    pub titulo: String,
    pub contratista: String,
    pub prefijo_correlativo: String,
    pub porcentaje_utilidades: f64,
    pub ito_nombre: Option<String>,
    pub ito_firma_base64: Option<String>,
}
