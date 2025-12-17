use crate::db::{DbState, Jardin, Partida, Requerimiento, RequerimientoEnriquecido, Configuracion, Recinto, OrdenTrabajo, InformePagoEnriquecido};
use sqlx::Row;
use tauri::State;

// ========== JARDINES ==========

#[tauri::command(rename_all = "snake_case")]
pub async fn get_jardines(db: State<'_, DbState>) -> Result<Vec<Jardin>, String> {
    sqlx::query_as::<_, Jardin>("SELECT * FROM jardines ORDER BY nombre")
        .fetch_all(&*db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_jardin_by_codigo(
    db: State<'_, DbState>,
    codigo: String,
) -> Result<Option<Jardin>, String> {
    sqlx::query_as::<_, Jardin>("SELECT * FROM jardines WHERE codigo = ?")
        .bind(codigo)
        .fetch_optional(&*db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_jardin(
    db: State<'_, DbState>,
    codigo: String,
    nombre: String,
) -> Result<i64, String> {
    let result = sqlx::query(
        "INSERT INTO jardines (codigo, nombre) VALUES (?, ?)"
    )
    .bind(&codigo)
    .bind(&nombre)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

// ========== PARTIDAS ==========

#[tauri::command(rename_all = "snake_case")]
pub async fn get_partidas(db: State<'_, DbState>) -> Result<Vec<Partida>, String> {
    sqlx::query_as::<_, Partida>("SELECT * FROM partidas ORDER BY item")
        .fetch_all(&*db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_partida(
    db: State<'_, DbState>,
    item: String,
    partida: String,
    unidad: Option<String>,
    precio_unitario: f64,
) -> Result<i64, String> {
    let result = sqlx::query(
        "INSERT INTO partidas (item, partida, unidad, precio_unitario) VALUES (?, ?, ?, ?)"
    )
    .bind(&item)
    .bind(&partida)
    .bind(&unidad)
    .bind(precio_unitario)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

// ========== REQUERIMIENTOS ==========

#[tauri::command(rename_all = "snake_case")]
pub async fn get_requerimientos(db: State<'_, DbState>) -> Result<Vec<RequerimientoEnriquecido>, String> {
    sqlx::query_as::<_, RequerimientoEnriquecido>(
        "SELECT 
            r.id,
            r.jardin_codigo,
            r.recinto,
            r.partida_item,
            p.partida as partida_nombre,
            p.unidad as partida_unidad,
            r.precio_unitario,
            r.cantidad,
            r.precio_total,
            r.fecha_inicio,
            r.plazo_dias,
            r.plazo_adicional,
            (r.plazo_dias + r.plazo_adicional) as plazo_total,
            CASE 
                WHEN (r.plazo_dias + r.plazo_adicional) > 0 
                THEN date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')
                ELSE NULL
            END as fecha_limite,
            r.fecha_registro,
            r.fecha_recepcion,
            CASE 
                WHEN r.fecha_recepcion IS NOT NULL 
                     AND (r.plazo_dias + r.plazo_adicional) > 0
                     AND date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days') < r.fecha_recepcion
                THEN CAST(julianday(r.fecha_recepcion) - julianday(date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')) AS INTEGER)
                ELSE 0
            END as dias_atraso,
            r.multa,
            r.a_pago,
            r.utilidades,
            r.iva,
            r.total_linea,
            r.descripcion,
            r.observaciones,
            r.estado,
            r.ot_id,
            ot.codigo as ot_codigo,
            r.informe_pago_id,
            ip.codigo as informe_pago_codigo,
            r.created_at,
            r.updated_at
        FROM requerimientos r
        LEFT JOIN partidas p ON r.partida_item = p.item
        LEFT JOIN ordenes_trabajo ot ON r.ot_id = ot.id
        LEFT JOIN informes_pago ip ON r.informe_pago_id = ip.id
        ORDER BY r.fecha_inicio DESC"
    )
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_requerimiento(
    db: State<'_, DbState>,
    jardin_codigo: String,
    recinto: Option<String>,
    partida_item: String,
    cantidad: f64,
    precio_unitario: f64,
    fecha_inicio: String,
    fecha_registro: String,
    plazo_dias: i32,
    descripcion: Option<String>,
) -> Result<i64, String> {
    let precio_total = cantidad * precio_unitario;
    
    let result = sqlx::query(
        "INSERT INTO requerimientos 
         (jardin_codigo, recinto, partida_item, cantidad, precio_unitario, precio_total, 
          fecha_inicio, fecha_registro, plazo_dias, descripcion, estado) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pendiente')"
    )
    .bind(&jardin_codigo)
    .bind(&recinto)
    .bind(&partida_item)
    .bind(cantidad)
    .bind(precio_unitario)
    .bind(precio_total)
    .bind(&fecha_inicio)
    .bind(&fecha_registro)
    .bind(plazo_dias)
    .bind(&descripcion)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_requerimiento(
    db: State<'_, DbState>,
    id: i64,
    descripcion: Option<String>,
    observaciones: Option<String>,
    cantidad: Option<f64>,
    precio_unitario: Option<f64>,
    fecha_inicio: Option<String>,
    plazo_dias: Option<i32>,
    plazo_adicional: Option<i32>,
    fecha_recepcion: Option<String>,
    partida_item: Option<String>,
) -> Result<(), String> {
    let mut set_parts = vec![];
    
    if descripcion.is_some() { set_parts.push("descripcion = ?"); }
    if observaciones.is_some() { set_parts.push("observaciones = ?"); }
    if partida_item.is_some() { set_parts.push("partida_item = ?"); }
    if cantidad.is_some() { set_parts.push("cantidad = ?"); }
    if precio_unitario.is_some() { set_parts.push("precio_unitario = ?"); }
    if fecha_inicio.is_some() { set_parts.push("fecha_inicio = ?"); }
    if plazo_dias.is_some() { set_parts.push("plazo_dias = ?"); }
    if plazo_adicional.is_some() { set_parts.push("plazo_adicional = ?"); }
    if fecha_recepcion.is_some() { set_parts.push("fecha_recepcion = ?"); }
    
    if cantidad.is_some() || precio_unitario.is_some() {
        set_parts.push("precio_total = cantidad * precio_unitario");
    }
    
    if set_parts.is_empty() {
        return Ok(());
    }
    
    set_parts.push("updated_at = datetime('now')");
    
    let query_str = format!(
        "UPDATE requerimientos SET {} WHERE id = ?",
        set_parts.join(", ")
    );
    
    let mut query = sqlx::query(&query_str);
    
    if let Some(v) = descripcion { query = query.bind(v); }
    if let Some(v) = observaciones { query = query.bind(v); }
    if let Some(v) = partida_item { query = query.bind(v); }
    if let Some(v) = cantidad { query = query.bind(v); }
    if let Some(v) = precio_unitario { query = query.bind(v); }
    if let Some(ref v) = fecha_inicio { query = query.bind(v); }
    if let Some(ref v) = plazo_dias { query = query.bind(v); }
    if let Some(ref v) = plazo_adicional { query = query.bind(v); }
    if let Some(v) = fecha_recepcion { query = query.bind(v); }
    
    query.bind(id)
        .execute(&*db.pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn actualizar_fecha_recepcion(
    db: State<'_, DbState>,
    id: i64,
    fecha_recepcion: String,
) -> Result<(), String> {
    // 1. Obtener requerimiento y config
    let req: Requerimiento = sqlx::query_as(
        "SELECT id, jardin_codigo, recinto, partida_item, cantidad, precio_unitario, 
                precio_total, fecha_inicio, fecha_registro, estado, ot_id, informe_pago_id,
                fecha_recepcion, plazo_dias, plazo_adicional, plazo_total, fecha_limite,
                multa, a_pago, utilidades, iva, total_linea, descripcion, observaciones,
                created_at, updated_at
         FROM requerimientos WHERE id = ?"
    )
        .bind(id)
        .fetch_one(&*db.pool)
        .await
        .map_err(|e| e.to_string())?;
    
    let config: Configuracion = sqlx::query_as(
        "SELECT id, titulo, contratista, prefijo_correlativo, porcentaje_utilidades, 
                ito_nombre, NULL as ito_firma_base64 
         FROM configuracion_contrato WHERE id = 1"
    )
    .fetch_one(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    // 2. Calcular campos
    let a_pago = req.precio_total - req.multa;
    let utilidades = (a_pago * config.porcentaje_utilidades).round();
    let iva = ((a_pago + utilidades) * 0.19).round();
    let total_linea = a_pago + utilidades + iva;
    
    // 3. Actualizar
    sqlx::query(
        "UPDATE requerimientos 
         SET fecha_recepcion = ?, a_pago = ?, utilidades = ?, iva = ?, total_linea = ?, 
             updated_at = datetime('now') 
         WHERE id = ?"
    )
    .bind(&fecha_recepcion)
    .bind(a_pago)
    .bind(utilidades)
    .bind(iva)
    .bind(total_linea)
    .bind(id)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn eliminar_fecha_recepcion(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE requerimientos SET fecha_recepcion = NULL, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(id)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_requerimiento(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM requerimientos WHERE id = ?")
        .bind(id)
        .execute(&*db.pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}


// ========== RECINTOS ==========

#[tauri::command]
pub async fn get_recintos(db: State<'_, DbState>) -> Result<Vec<Recinto>, String> {
    sqlx::query_as::<_, Recinto>("SELECT * FROM recintos ORDER BY nombre")
        .fetch_all(&*db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_recintos_by_jardin(
    db: State<'_, DbState>,
    jardin_codigo: String,
) -> Result<Vec<Recinto>, String> {
    sqlx::query_as::<_, Recinto>(
        "SELECT * FROM recintos WHERE jardin_codigo = ? ORDER BY nombre"
    )
    .bind(jardin_codigo)
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_recinto(
    db: State<'_, DbState>,
    jardin_codigo: String,
    nombre: String,
) -> Result<i64, String> {
    let result = sqlx::query(
        "INSERT INTO recintos (jardin_codigo, nombre) VALUES (?, ?)"
    )
    .bind(&jardin_codigo)
    .bind(&nombre)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}


// ========== ÓRDENES DE TRABAJO ==========

#[tauri::command]
pub async fn get_ordenes_trabajo(db: State<'_, DbState>) -> Result<Vec<OrdenTrabajo>, String> {
    sqlx::query_as::<_, OrdenTrabajo>(
        "SELECT * FROM ordenes_trabajo ORDER BY fecha_creacion DESC"
    )
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_orden_trabajo_detalle(
    db: State<'_, DbState>,
    ot_id: i64,
) -> Result<Vec<RequerimientoEnriquecido>, String> {
    sqlx::query_as::<_, RequerimientoEnriquecido>(
        "SELECT 
            r.id,
            r.jardin_codigo,
            r.recinto,
            r.partida_item,
            p.partida as partida_nombre,
            p.unidad as partida_unidad,
            r.precio_unitario,
            r.cantidad,
            r.precio_total,
            r.fecha_inicio,
            r.plazo_dias,
            r.plazo_adicional,
            (r.plazo_dias + r.plazo_adicional) as plazo_total,
            CASE 
                WHEN (r.plazo_dias + r.plazo_adicional) > 0 
                THEN date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')
                ELSE NULL
            END as fecha_limite,
            r.fecha_registro,
            r.fecha_recepcion,
            CASE 
                WHEN r.fecha_recepcion IS NOT NULL 
                     AND (r.plazo_dias + r.plazo_adicional) > 0
                     AND date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days') < r.fecha_recepcion
                THEN CAST(julianday(r.fecha_recepcion) - julianday(date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')) AS INTEGER)
                ELSE 0
            END as dias_atraso,
            r.multa,
            r.a_pago,
            r.utilidades,
            r.iva,
            r.total_linea,
            r.descripcion,
            r.observaciones,
            r.estado,
            r.ot_id,
            ot.codigo as ot_codigo,
            r.informe_pago_id,
            ip.codigo as informe_pago_codigo,
            r.created_at,
            r.updated_at
        FROM requerimientos r
        LEFT JOIN partidas p ON r.partida_item = p.item
        LEFT JOIN ordenes_trabajo ot ON r.ot_id = ot.id
        LEFT JOIN informes_pago ip ON r.informe_pago_id = ip.id
        WHERE r.ot_id = ?
        ORDER BY r.fecha_inicio DESC"
    )
    .bind(ot_id)
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn crear_orden_trabajo(
    db: State<'_, DbState>,
    jardin_codigo: String,
    fecha_creacion: String,
    observaciones: Option<String>,
    requerimiento_ids: Vec<i64>,
) -> Result<i64, String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Obtener configuración para prefijo
    let config_row = sqlx::query("SELECT prefijo_correlativo FROM configuracion_contrato WHERE id = 1")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let prefijo: String = config_row.get("prefijo_correlativo");
    
    // Obtener último número de OT para este jardín específico
    let pattern = format!("OT-{}-{}", jardin_codigo, prefijo);
    let last_ot: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(CAST(SUBSTR(codigo, LENGTH(?) + 1) AS INTEGER)) FROM ordenes_trabajo WHERE codigo LIKE ? || '%'"
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .flatten();
    
    let next_num = last_ot.unwrap_or(0) + 1;
    let codigo = format!("OT-{}-{}{:03}", jardin_codigo, prefijo, next_num);
    
    // Crear OT
    let result = sqlx::query(
        "INSERT INTO ordenes_trabajo (codigo, jardin_codigo, fecha_creacion, observaciones) 
         VALUES (?, ?, ?, ?)"
    )
    .bind(&codigo)
    .bind(&jardin_codigo)
    .bind(&fecha_creacion)
    .bind(&observaciones)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    
    let ot_id = result.last_insert_rowid();
    
    // Vincular requerimientos
    for req_id in requerimiento_ids {
        sqlx::query("UPDATE requerimientos SET ot_id = ?, estado = 'en_ot', updated_at = datetime('now') WHERE id = ?")
            .bind(ot_id)
            .bind(req_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(ot_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn eliminar_orden_trabajo(
    db: State<'_, DbState>,
    ot_id: i64,
) -> Result<(), String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Desvincular requerimientos
    sqlx::query("UPDATE requerimientos SET ot_id = NULL, estado = 'pendiente', updated_at = datetime('now') WHERE ot_id = ?")
        .bind(ot_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    // Eliminar OT
    sqlx::query("DELETE FROM ordenes_trabajo WHERE id = ?")
        .bind(ot_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}


// ========== INFORMES DE PAGO ==========

#[tauri::command]
pub async fn get_informes_pago(db: State<'_, DbState>) -> Result<Vec<InformePagoEnriquecido>, String> {
    sqlx::query_as::<_, InformePagoEnriquecido>(
        "SELECT 
            ip.id,
            ip.codigo,
            ip.jardin_codigo,
            j.nombre as jardin_nombre,
            ip.fecha_creacion,
            ip.neto,
            ip.utilidades,
            ip.iva,
            ip.total_pagar,
            COUNT(DISTINCT r.id) as cantidad_requerimientos,
            ip.observaciones,
            ip.created_at,
            ip.updated_at
        FROM informes_pago ip
        LEFT JOIN jardines j ON ip.jardin_codigo = j.codigo
        LEFT JOIN requerimientos r ON ip.id = r.informe_pago_id
        GROUP BY ip.id
        ORDER BY ip.fecha_creacion DESC"
    )
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_informe_pago_detalle(
    db: State<'_, DbState>,
    informe_id: i64,
) -> Result<Vec<RequerimientoEnriquecido>, String> {
    sqlx::query_as::<_, RequerimientoEnriquecido>(
        "SELECT 
            r.id,
            r.jardin_codigo,
            r.recinto,
            r.partida_item,
            p.partida as partida_nombre,
            p.unidad as partida_unidad,
            r.precio_unitario,
            r.cantidad,
            r.precio_total,
            r.fecha_inicio,
            r.plazo_dias,
            r.plazo_adicional,
            (r.plazo_dias + r.plazo_adicional) as plazo_total,
            CASE 
                WHEN (r.plazo_dias + r.plazo_adicional) > 0 
                THEN date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')
                ELSE NULL
            END as fecha_limite,
            r.fecha_registro,
            r.fecha_recepcion,
            CASE 
                WHEN r.fecha_recepcion IS NOT NULL 
                     AND (r.plazo_dias + r.plazo_adicional) > 0
                     AND date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days') < r.fecha_recepcion
                THEN CAST(julianday(r.fecha_recepcion) - julianday(date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')) AS INTEGER)
                ELSE 0
            END as dias_atraso,
            r.multa,
            r.a_pago,
            r.utilidades,
            r.iva,
            r.total_linea,
            r.descripcion,
            r.observaciones,
            r.estado,
            r.ot_id,
            ot.codigo as ot_codigo,
            r.informe_pago_id,
            ip.codigo as informe_pago_codigo,
            r.created_at,
            r.updated_at
        FROM requerimientos r
        LEFT JOIN partidas p ON r.partida_item = p.item
        LEFT JOIN ordenes_trabajo ot ON r.ot_id = ot.id
        LEFT JOIN informes_pago ip ON r.informe_pago_id = ip.id
        WHERE r.informe_pago_id = ?
        ORDER BY r.fecha_inicio DESC"
    )
    .bind(informe_id)
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_requerimientos_para_informe(
    db: State<'_, DbState>,
    jardin_codigo: String,
) -> Result<Vec<RequerimientoEnriquecido>, String> {
    sqlx::query_as::<_, RequerimientoEnriquecido>(
        "SELECT 
            r.id,
            r.jardin_codigo,
            r.recinto,
            r.partida_item,
            p.partida as partida_nombre,
            p.unidad as partida_unidad,
            p.precio_unitario,
            r.cantidad,
            r.precio_total,
            r.fecha_inicio,
            r.plazo_dias,
            r.plazo_adicional,
            (r.plazo_dias + r.plazo_adicional) as plazo_total,
            CASE 
                WHEN (r.plazo_dias + r.plazo_adicional) > 0 
                THEN date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')
                ELSE NULL
            END as fecha_limite,
            r.fecha_registro,
            r.fecha_recepcion,
            CASE 
                WHEN r.fecha_recepcion IS NOT NULL 
                     AND (r.plazo_dias + r.plazo_adicional) > 0
                     AND date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days') < r.fecha_recepcion
                THEN CAST(julianday(r.fecha_recepcion) - julianday(date(r.fecha_inicio, '+' || (r.plazo_dias + r.plazo_adicional) || ' days')) AS INTEGER)
                ELSE 0
            END as dias_atraso,
            r.multa,
            r.a_pago,
            r.utilidades,
            r.iva,
            r.total_linea,
            r.descripcion,
            r.observaciones,
            r.estado,
            r.ot_id,
            ot.codigo as ot_codigo,
            r.informe_pago_id,
            ip.codigo as informe_pago_codigo,
            r.created_at,
            r.updated_at
        FROM requerimientos r
        LEFT JOIN partidas p ON r.partida_item = p.item
        LEFT JOIN ordenes_trabajo ot ON r.ot_id = ot.id
        LEFT JOIN informes_pago ip ON r.informe_pago_id = ip.id
        WHERE r.jardin_codigo = ? 
        AND r.fecha_recepcion IS NOT NULL
        AND r.informe_pago_id IS NULL
        ORDER BY r.fecha_inicio DESC"
    )
    .bind(jardin_codigo)
    .fetch_all(&*db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn crear_informe_pago(
    db: State<'_, DbState>,
    jardin_codigo: String,
    fecha_creacion: String,
    observaciones: Option<String>,
    requerimientos: Vec<serde_json::Value>,
) -> Result<i64, String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Obtener configuración para prefijo
    let config_row = sqlx::query("SELECT prefijo_correlativo FROM configuracion_contrato WHERE id = 1")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let prefijo: String = config_row.get("prefijo_correlativo");
    
    // Obtener último número de informe para este jardín específico
    let pattern = format!("IP-{}-{}", jardin_codigo, prefijo);
    let last_informe: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(CAST(SUBSTR(codigo, LENGTH(?) + 1) AS INTEGER)) FROM informes_pago WHERE codigo LIKE ? || '%'"
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .flatten();
    
    let next_num = last_informe.unwrap_or(0) + 1;
    let codigo = format!("IP-{}-{}{:02}", jardin_codigo, prefijo, next_num);
    
    // Calcular totales por agregación de campos pre-calculados
    let mut neto = 0.0;
    let mut utilidades_total = 0.0;
    let mut iva_total = 0.0;
    let mut total_pagar = 0.0;
    
    for req in &requerimientos {
        if let Some(a_pago) = req.get("aPago").or_else(|| req.get("a_pago")).and_then(|m| m.as_f64()) {
            neto += a_pago;
        }
        if let Some(utilidades) = req.get("utilidades").and_then(|m| m.as_f64()) {
            utilidades_total += utilidades;
        }
        if let Some(iva) = req.get("iva").and_then(|m| m.as_f64()) {
            iva_total += iva;
        }
        if let Some(total_linea) = req.get("totalLinea").or_else(|| req.get("total_linea")).and_then(|m| m.as_f64()) {
            total_pagar += total_linea;
        }
    }
    
    // Crear informe
    let result = sqlx::query(
        "INSERT INTO informes_pago (codigo, jardin_codigo, fecha_creacion, neto, utilidades, iva, total_pagar, observaciones) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&codigo)
    .bind(&jardin_codigo)
    .bind(&fecha_creacion)
    .bind(neto)
    .bind(utilidades_total)
    .bind(iva_total)
    .bind(total_pagar)
    .bind(&observaciones)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    
    let informe_id = result.last_insert_rowid();
    
    // Vincular requerimientos
    for req in requerimientos {
        if let Some(req_id) = req.get("id").and_then(|v| v.as_i64()) {
            sqlx::query("UPDATE requerimientos SET informe_pago_id = ?, estado = 'en_informe', updated_at = datetime('now') WHERE id = ?")
                .bind(informe_id)
                .bind(req_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(informe_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn eliminar_informe_pago(
    db: State<'_, DbState>,
    informe_id: i64,
) -> Result<(), String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Desvincular requerimientos (volver a pendiente)
    sqlx::query("UPDATE requerimientos SET informe_pago_id = NULL, estado = 'pendiente', updated_at = datetime('now') WHERE informe_pago_id = ?")
        .bind(informe_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    // Eliminar informe
    sqlx::query("DELETE FROM informes_pago WHERE id = ?")
        .bind(informe_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ========== CONFIGURACIÓN ==========

#[tauri::command(rename_all = "snake_case")]
pub async fn get_configuracion(db: State<'_, DbState>) -> Result<Configuracion, String> {
    use base64::{Engine as _, engine::general_purpose};
    
    let row = sqlx::query(
        "SELECT id, titulo, contratista, prefijo_correlativo, 
                COALESCE(porcentaje_utilidades, 0.25) as porcentaje_utilidades,
                ito_nombre, firma_png 
         FROM configuracion_contrato WHERE id = 1"
    )
        .fetch_one(&*db.pool)
        .await
        .map_err(|e| e.to_string())?;
    
    // Obtener firma_png y convertir a base64 con data URI
    let firma_bytes: Option<Vec<u8>> = row.get("firma_png");
    let ito_firma_base64 = firma_bytes.map(|bytes| {
        format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(&bytes))
    });
    
    Ok(Configuracion {
        id: row.get("id"),
        titulo: row.get("titulo"),
        contratista: row.get("contratista"),
        prefijo_correlativo: row.get("prefijo_correlativo"),
        porcentaje_utilidades: row.get("porcentaje_utilidades"),
        ito_nombre: row.get("ito_nombre"),
        ito_firma_base64,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_configuracion(
    db: State<'_, DbState>,
    titulo: String,
    contratista: String,
    prefijo_correlativo: String,
    porcentaje_utilidades: f64,
    ito_nombre: Option<String>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE configuracion_contrato 
         SET titulo = ?, contratista = ?, prefijo_correlativo = ?, porcentaje_utilidades = ?, 
             ito_nombre = ?, updated_at = datetime('now') 
         WHERE id = 1"
    )
    .bind(&titulo)
    .bind(&contratista)
    .bind(&prefijo_correlativo)
    .bind(porcentaje_utilidades)
    .bind(&ito_nombre)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

// ========== UTILIDADES ==========

#[tauri::command]
pub async fn clear_all(db: State<'_, DbState>) -> Result<(), String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    sqlx::query("DELETE FROM informes_pago").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM requerimientos").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM ordenes_trabajo").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM recintos").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM partidas").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM jardines").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ========== IMPORTACIÓN ==========

#[derive(serde::Deserialize)]
struct CatalogoImport {
    jardines: Option<Vec<serde_json::Value>>,
    partidas: Option<Vec<serde_json::Value>>,
    recintos: Option<Vec<serde_json::Value>>,
}

#[derive(serde::Deserialize)]
struct BaseDatosCompleta {
    jardines: Option<Vec<serde_json::Value>>,
    partidas: Option<Vec<serde_json::Value>>,
    recintos: Option<Vec<serde_json::Value>>,
    requerimientos: Option<Vec<serde_json::Value>>,
    ordenes_trabajo: Option<Vec<serde_json::Value>>,
    informes_pago: Option<Vec<serde_json::Value>>,
    configuracion: Option<serde_json::Value>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn importar_base_datos_completa(
    db: State<'_, DbState>,
    json_str: String,
) -> Result<String, String> {
    let datos: BaseDatosCompleta = serde_json::from_str(&json_str)
        .map_err(|e| format!("Error parseando JSON: {}", e))?;
    
    // 1. BORRAR TODO
    clear_all(db.clone()).await?;
    
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    let mut counts = (0, 0, 0, 0, 0, 0);
    
    // 2. IMPORTAR JARDINES
    if let Some(jardines) = datos.jardines {
        for j in jardines {
            if let (Some(codigo), Some(nombre)) = (
                j.get("codigo").and_then(|v| v.as_str()),
                j.get("nombre").and_then(|v| v.as_str())
            ) {
                sqlx::query("INSERT OR IGNORE INTO jardines (codigo, nombre) VALUES (?, ?)")
                    .bind(codigo).bind(nombre)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                counts.0 += 1;
            }
        }
    }
    
    // 3. IMPORTAR PARTIDAS
    if let Some(partidas) = datos.partidas {
        for p in partidas {
            if let (Some(item), Some(partida)) = (
                p.get("item").and_then(|v| v.as_str()),
                p.get("partida").and_then(|v| v.as_str())
            ) {
                let unidad = p.get("unidad").and_then(|v| v.as_str());
                let precio = p.get("precioUnitario").or(p.get("precio_unitario")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                sqlx::query("INSERT OR IGNORE INTO partidas (item, partida, unidad, precio_unitario) VALUES (?, ?, ?, ?)")
                    .bind(item).bind(partida).bind(unidad).bind(precio)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                counts.1 += 1;
            }
        }
    }
    
    // 4. IMPORTAR RECINTOS
    if let Some(recintos) = datos.recintos {
        for r in recintos {
            if let (Some(jardin_codigo), Some(nombre)) = (
                r.get("jardinCodigo").or(r.get("jardin_codigo")).and_then(|v| v.as_str()),
                r.get("nombre").and_then(|v| v.as_str())
            ) {
                sqlx::query("INSERT INTO recintos (jardin_codigo, nombre) VALUES (?, ?)")
                    .bind(jardin_codigo).bind(nombre)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                counts.2 += 1;
            }
        }
    }
    
    // 5. IMPORTAR ÓRDENES DE TRABAJO (ANTES de requerimientos)
    // ✅ Crear mapa de código → ID para resolver referencias
    let mut ot_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    if let Some(ordenes) = datos.ordenes_trabajo {
        for ot in ordenes {
            if let (Some(codigo), Some(jardin_codigo), Some(fecha_creacion)) = (
                ot.get("codigo").and_then(|v| v.as_str()),
                ot.get("jardinCodigo").or(ot.get("jardin_codigo")).and_then(|v| v.as_str()),
                ot.get("fechaCreacion").or(ot.get("fecha_creacion")).and_then(|v| v.as_str())
            ) {
                let observaciones = ot.get("observaciones").and_then(|v| v.as_str());
                let result = sqlx::query("INSERT INTO ordenes_trabajo (codigo, jardin_codigo, fecha_creacion, observaciones) VALUES (?, ?, ?, ?)")
                    .bind(codigo).bind(jardin_codigo).bind(fecha_creacion).bind(observaciones)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                
                // Guardar código → ID en el mapa
                ot_map.insert(codigo.to_string(), result.last_insert_rowid());
                counts.4 += 1;
            }
        }
    }
    
    // 6. IMPORTAR INFORMES DE PAGO (ANTES de requerimientos)
    // ✅ Crear mapa de código → ID para resolver referencias
    let mut informe_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    if let Some(informes) = datos.informes_pago {
        for inf in informes {
            if let (Some(codigo), Some(jardin_codigo), Some(fecha_creacion)) = (
                inf.get("codigo").and_then(|v| v.as_str()),
                inf.get("jardinCodigo").or(inf.get("jardin_codigo")).and_then(|v| v.as_str()),
                inf.get("fechaCreacion").or(inf.get("fecha_creacion")).and_then(|v| v.as_str())
            ) {
                let neto = inf.get("neto").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let utilidades = inf.get("utilidades").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let iva = inf.get("iva").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let total_pagar = inf.get("totalPagar").or(inf.get("total_pagar")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let observaciones = inf.get("observaciones").and_then(|v| v.as_str());
                
                let result = sqlx::query("INSERT INTO informes_pago (codigo, jardin_codigo, fecha_creacion, neto, utilidades, iva, total_pagar, observaciones) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                    .bind(codigo).bind(jardin_codigo).bind(fecha_creacion)
                    .bind(neto).bind(utilidades).bind(iva).bind(total_pagar).bind(observaciones)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                
                // Guardar código → ID en el mapa
                informe_map.insert(codigo.to_string(), result.last_insert_rowid());
                counts.5 += 1;
            }
        }
    }
    
    // 7. IMPORTAR REQUERIMIENTOS (AL FINAL, después de OTs e Informes)
    // ✅ Mapear códigos → IDs usando los mapas creados arriba
    if let Some(requerimientos) = datos.requerimientos {
        for req in requerimientos {
            let jardin_codigo = req.get("jardinCodigo").or(req.get("jardin_codigo")).and_then(|v| v.as_str());
            let partida_item = req.get("partidaItem").or(req.get("partida_item")).and_then(|v| v.as_str());
            
            if let (Some(jc), Some(pi)) = (jardin_codigo, partida_item) {
                let recinto = req.get("recinto").and_then(|v| v.as_str());
                let cantidad = req.get("cantidad").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let precio_unitario = req.get("precioUnitario").or(req.get("precio_unitario")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let precio_total = req.get("precioTotal").or(req.get("precio_total")).and_then(|v| v.as_f64()).unwrap_or(cantidad * precio_unitario);
                let fecha_inicio = req.get("fechaInicio").or(req.get("fecha_inicio")).and_then(|v| v.as_str()).unwrap_or("");
                let fecha_registro = req.get("fechaRegistro").or(req.get("fecha_registro")).and_then(|v| v.as_str()).unwrap_or("");
                let estado = req.get("estado").and_then(|v| v.as_str()).unwrap_or("pendiente");
                
                // ✅ Mapear código de OT → ID (SOLO códigos, ignorar IDs numéricos)
                let ot_id = req.get("ot_codigo")
                    .and_then(|v| v.as_str())
                    .and_then(|codigo| ot_map.get(codigo))
                    .copied();
                
                // ✅ Mapear código de Informe → ID (SOLO códigos, ignorar IDs numéricos)
                let informe_pago_id = req.get("informe_codigo")
                    .and_then(|v| v.as_str())
                    .and_then(|codigo| informe_map.get(codigo))
                    .copied();
                
                let plazo_dias = req.get("plazoDias").or(req.get("plazo_dias")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let plazo_adicional = req.get("plazoAdicional").or(req.get("plazo_adicional")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let descripcion = req.get("descripcion").and_then(|v| v.as_str());
                let observaciones = req.get("observaciones").and_then(|v| v.as_str());
                let fecha_recepcion = req.get("fechaRecepcion").or(req.get("fecha_recepcion")).and_then(|v| v.as_str());
                let multa = req.get("multa").and_then(|v| v.as_f64());
                let a_pago = req.get("a_pago").and_then(|v| v.as_f64());
                let utilidades = req.get("utilidades").and_then(|v| v.as_f64());
                let iva = req.get("iva").and_then(|v| v.as_f64());
                let total_linea = req.get("total_linea").and_then(|v| v.as_f64());
                
                sqlx::query(
                    "INSERT INTO requerimientos 
                     (jardin_codigo, recinto, partida_item, cantidad, precio_unitario, precio_total,
                      fecha_inicio, fecha_registro, estado, ot_id, informe_pago_id, plazo_dias, plazo_adicional, 
                      descripcion, observaciones, fecha_recepcion, multa, a_pago, utilidades, iva, total_linea)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(jc).bind(recinto).bind(pi).bind(cantidad).bind(precio_unitario).bind(precio_total)
                .bind(fecha_inicio).bind(fecha_registro).bind(estado).bind(ot_id).bind(informe_pago_id)
                .bind(plazo_dias).bind(plazo_adicional).bind(descripcion).bind(observaciones).bind(fecha_recepcion)
                .bind(multa).bind(a_pago).bind(utilidades).bind(iva).bind(total_linea)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                counts.3 += 1;
            }
        }
    }
    
    // 8. IMPORTAR CONFIGURACIÓN
    if let Some(config) = datos.configuracion {
        let titulo = config.get("titulo").and_then(|v| v.as_str()).unwrap_or("");
        let contratista = config.get("contratista").and_then(|v| v.as_str()).unwrap_or("");
        let prefijo = config.get("prefijo_correlativo").and_then(|v| v.as_str()).unwrap_or("");
        let ito_nombre = config.get("ito_nombre").and_then(|v| v.as_str()).unwrap_or("");
        let porcentaje_utilidades = config.get("porcentaje_utilidades").and_then(|v| v.as_f64()).unwrap_or(0.25);
        
        sqlx::query(
            "UPDATE configuracion_contrato 
             SET titulo = ?, contratista = ?, prefijo_correlativo = ?, ito_nombre = ?, porcentaje_utilidades = ?, updated_at = datetime('now') 
             WHERE id = 1"
        )
        .bind(titulo)
        .bind(contratista)
        .bind(prefijo)
        .bind(ito_nombre)
        .bind(porcentaje_utilidades)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        
        // Importar firma PNG si existe
        if let Some(firma_base64) = config.get("firma_png_base64").and_then(|v| v.as_str()) {
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(firma_bytes) = general_purpose::STANDARD.decode(firma_base64) {
                sqlx::query("UPDATE configuracion_contrato SET firma_png = ?, updated_at = datetime('now') WHERE id = 1")
                    .bind(firma_bytes)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
            }
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(format!("✅ Importado: {} jardines, {} partidas, {} recintos, {} requerimientos, {} OTs, {} informes", 
        counts.0, counts.1, counts.2, counts.3, counts.4, counts.5))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn importar_catalogo_json(
    db: State<'_, DbState>,
    json_str: String,
) -> Result<String, String> {
    let catalogo: CatalogoImport = serde_json::from_str(&json_str)
        .map_err(|e| format!("Error parseando JSON: {}", e))?;
    
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    let mut count = 0;
    
    // Importar jardines
    if let Some(jardines) = catalogo.jardines {
        for j in jardines {
            if let (Some(codigo), Some(nombre)) = (
                j.get("codigo").and_then(|v| v.as_str()),
                j.get("nombre").and_then(|v| v.as_str())
            ) {
                sqlx::query("INSERT OR IGNORE INTO jardines (codigo, nombre) VALUES (?, ?)")
                    .bind(codigo)
                    .bind(nombre)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }
    
    // Importar partidas
    if let Some(partidas) = catalogo.partidas {
        for p in partidas {
            if let (Some(item), Some(partida)) = (
                p.get("item").and_then(|v| v.as_str()),
                p.get("partida").and_then(|v| v.as_str())
            ) {
                let unidad = p.get("unidad").and_then(|v| v.as_str());
                let precio = p.get("precio_unitario").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                sqlx::query("INSERT OR IGNORE INTO partidas (item, partida, unidad, precio_unitario) VALUES (?, ?, ?, ?)")
                    .bind(item)
                    .bind(partida)
                    .bind(unidad)
                    .bind(precio)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }
    
    // Importar recintos
    if let Some(recintos) = catalogo.recintos {
        for r in recintos {
            if let (Some(jardin_codigo), Some(nombre)) = (
                r.get("jardin_codigo").and_then(|v| v.as_str()),
                r.get("nombre").and_then(|v| v.as_str())
            ) {
                sqlx::query("INSERT INTO recintos (jardin_codigo, nombre) VALUES (?, ?)")
                    .bind(jardin_codigo)
                    .bind(nombre)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(format!("{} registros importados", count))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn importar_catalogo_csv(
    db: State<'_, DbState>,
    csv_str: String,
    tipo: String,
) -> Result<String, String> {
    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    let mut count = 0;
    
    match tipo.as_str() {
        "jardines" => {
            for result in rdr.records() {
                let record = result.map_err(|e| e.to_string())?;
                if record.len() >= 2 {
                    sqlx::query("INSERT OR IGNORE INTO jardines (codigo, nombre) VALUES (?, ?)")
                        .bind(&record[0])
                        .bind(&record[1])
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    count += 1;
                }
            }
        }
        "partidas" => {
            for result in rdr.records() {
                let record = result.map_err(|e| e.to_string())?;
                if record.len() >= 2 {
                    let unidad = record.get(2).map(|s| s.to_string());
                    let precio = record.get(3).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    
                    sqlx::query("INSERT OR IGNORE INTO partidas (item, partida, unidad, precio_unitario) VALUES (?, ?, ?, ?)")
                        .bind(&record[0])
                        .bind(&record[1])
                        .bind(unidad)
                        .bind(precio)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    count += 1;
                }
            }
        }
        _ => return Err("Tipo de importación no válido".to_string()),
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(format!("{} registros importados", count))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn importar_catalogo_xlsx(
    db: State<'_, DbState>,
    file_path: String,
    sheet_name: String,
    tipo: String,
) -> Result<String, String> {
    use calamine::{Reader, open_workbook, Xlsx};
    
    let mut workbook: Xlsx<_> = open_workbook(&file_path)
        .map_err(|e| format!("Error abriendo Excel: {}", e))?;
    
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Error leyendo hoja: {}", e))?;
    
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    let mut count = 0;
    
    for row in range.rows().skip(1) {
        match tipo.as_str() {
            "jardines" if row.len() >= 2 => {
                let codigo = row[0].to_string();
                let nombre = row[1].to_string();
                
                sqlx::query("INSERT OR IGNORE INTO jardines (codigo, nombre) VALUES (?, ?)")
                    .bind(&codigo)
                    .bind(&nombre)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
            "partidas" if row.len() >= 2 => {
                let item = row[0].to_string();
                let partida = row[1].to_string();
                let unidad = if row.len() > 2 { Some(row[2].to_string()) } else { None };
                let precio = if row.len() > 3 {
                    row[3].to_string().parse::<f64>().unwrap_or(0.0)
                } else { 0.0 };
                
                sqlx::query("INSERT OR IGNORE INTO partidas (item, partida, unidad, precio_unitario) VALUES (?, ?, ?, ?)")
                    .bind(&item)
                    .bind(&partida)
                    .bind(unidad)
                    .bind(precio)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
            _ => {}
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(format!("{} registros importados", count))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn importar_catalogo_xlsx_bytes(
    db: State<'_, DbState>,
    file_bytes: Vec<u8>,
) -> Result<serde_json::Value, String> {
    use calamine::{Reader, Xlsx, open_workbook_from_rs};
    use std::io::Cursor;
    
    let cursor = Cursor::new(file_bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| format!("Error abriendo Excel: {}", e))?;
    
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Borrar catálogos existentes antes de importar
    sqlx::query("DELETE FROM recintos").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM partidas").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM jardines").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    
    let mut jardines_count = 0;
    let mut partidas_count = 0;
    let mut recintos_count = 0;
    let mut contrato_actualizado = false;
    
    // HOJA: jardines
    if let Ok(range) = workbook.worksheet_range("jardines") {
        for row in range.rows().skip(1) {
            if row.len() >= 2 {
                let codigo = row[0].to_string().trim().to_string();
                let nombre = row[1].to_string().trim().to_string();
                
                if !codigo.is_empty() && !nombre.is_empty() {
                    sqlx::query("INSERT INTO jardines (codigo, nombre) VALUES (?, ?)")
                        .bind(&codigo)
                        .bind(&nombre)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    jardines_count += 1;
                }
            }
        }
    }
    
    // HOJA: partidas
    if let Ok(range) = workbook.worksheet_range("partidas") {
        for row in range.rows().skip(1) {
            if row.len() >= 2 {
                let item = row[0].to_string().trim().to_string();
                let partida = row[1].to_string().trim().to_string();
                let unidad = if row.len() > 2 { 
                    let u = row[2].to_string().trim().to_string();
                    if u.is_empty() { None } else { Some(u) }
                } else { None };
                let precio = if row.len() > 3 {
                    row[3].to_string().trim().parse::<f64>().unwrap_or(0.0)
                } else { 0.0 };
                
                if !item.is_empty() && !partida.is_empty() {
                    sqlx::query("INSERT INTO partidas (item, partida, unidad, precio_unitario) VALUES (?, ?, ?, ?)")
                        .bind(&item)
                        .bind(&partida)
                        .bind(unidad)
                        .bind(precio)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    partidas_count += 1;
                }
            }
        }
    }
    
    // HOJA: recintos
    if let Ok(range) = workbook.worksheet_range("recintos") {
        for row in range.rows().skip(1) {
            if row.len() >= 2 {
                let jardin_codigo = row[0].to_string().trim().to_string();
                let nombre = row[1].to_string().trim().to_string();
                
                if !jardin_codigo.is_empty() && !nombre.is_empty() {
                    sqlx::query("INSERT INTO recintos (jardin_codigo, nombre) VALUES (?, ?)")
                        .bind(&jardin_codigo)
                        .bind(&nombre)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    recintos_count += 1;
                }
            }
        }
    }
    
    // HOJA: configuracion
    if let Ok(range) = workbook.worksheet_range("configuracion") {
        if let Some(row) = range.rows().nth(1) {
            if row.len() >= 3 {
                let titulo = row[0].to_string().trim().to_string();
                let prefijo = row[1].to_string().trim().to_string();
                let contratista = row[2].to_string().trim().to_string();
                let utilidades = if row.len() > 3 {
                    row[3].to_string().parse::<f64>().unwrap_or(25.0) / 100.0
                } else {
                    0.25
                };
                
                if !titulo.is_empty() {
                    sqlx::query(
                        "UPDATE configuracion_contrato 
                         SET titulo = ?, prefijo_correlativo = ?, contratista = ?, porcentaje_utilidades = ?, updated_at = datetime('now')
                         WHERE id = 1"
                    )
                    .bind(&titulo)
                    .bind(&prefijo)
                    .bind(&contratista)
                    .bind(utilidades)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                    
                    contrato_actualizado = true;
                }
            }
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "jardines": jardines_count,
        "partidas": partidas_count,
        "recintos": recintos_count,
        "contrato": contrato_actualizado
    }))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_orden_trabajo(
    db: State<'_, DbState>,
    ot_id: i64,
    requerimiento_ids: Vec<i64>,
    observaciones: Option<String>,
) -> Result<(), String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Obtener el jardín de la OT
    let ot_jardin: (String,) = sqlx::query_as("SELECT jardin_codigo FROM ordenes_trabajo WHERE id = ?")
        .bind(ot_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("OT no encontrada: {}", e))?;
    
    // Validar que todos los requerimientos pertenecen al mismo jardín
    for req_id in &requerimiento_ids {
        let req_jardin: (String,) = sqlx::query_as(
            "SELECT jardin_codigo FROM requerimientos WHERE id = ?"
        )
        .bind(req_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Error al verificar requerimiento {}: {}", req_id, e))?
        .ok_or_else(|| format!("Requerimiento {} no existe", req_id))?;
        
        if req_jardin.0 != ot_jardin.0 {
            return Err(format!(
                "El requerimiento {} pertenece al jardín '{}' pero la OT es del jardín '{}'",
                req_id, req_jardin.0, ot_jardin.0
            ));
        }
    }
    
    // Actualizar observaciones si se proporcionan
    if let Some(obs) = observaciones {
        sqlx::query("UPDATE ordenes_trabajo SET observaciones = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(obs)
            .bind(ot_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    // Desvincular todos los requerimientos actuales
    sqlx::query("UPDATE requerimientos SET ot_id = NULL, estado = 'pendiente', updated_at = datetime('now') WHERE ot_id = ?")
        .bind(ot_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    // Vincular nuevos requerimientos
    for req_id in requerimiento_ids {
        sqlx::query("UPDATE requerimientos SET ot_id = ?, estado = 'en_ot', updated_at = datetime('now') WHERE id = ?")
            .bind(ot_id)
            .bind(req_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_informe_pago(
    db: State<'_, DbState>,
    informe_id: i64,
    requerimientos: Vec<serde_json::Value>,
    observaciones: Option<String>,
) -> Result<(), String> {
    let mut tx = db.pool.begin().await.map_err(|e| e.to_string())?;
    
    // Calcular totales por agregación (sin aplicar %)
    let mut neto = 0.0;
    let mut utilidades_total = 0.0;
    let mut iva_total = 0.0;
    let mut total_pagar = 0.0;
    
    for req in &requerimientos {
        if let Some(a_pago) = req.get("aPago").or_else(|| req.get("a_pago")).and_then(|m| m.as_f64()) {
            neto += a_pago;
        }
        if let Some(utilidades) = req.get("utilidades").and_then(|m| m.as_f64()) {
            utilidades_total += utilidades;
        }
        if let Some(iva) = req.get("iva").and_then(|m| m.as_f64()) {
            iva_total += iva;
        }
        if let Some(total_linea) = req.get("totalLinea").or_else(|| req.get("total_linea")).and_then(|m| m.as_f64()) {
            total_pagar += total_linea;
        }
    }
    
    // Actualizar informe
    sqlx::query(
        "UPDATE informes_pago 
         SET neto = ?, utilidades = ?, iva = ?, total_pagar = ?, observaciones = ?, updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(neto)
    .bind(utilidades_total)
    .bind(iva_total)
    .bind(total_pagar)
    .bind(&observaciones)
    .bind(informe_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    
    // Desvincular requerimientos viejos
    sqlx::query("UPDATE requerimientos SET informe_pago_id = NULL, estado = 'pendiente', updated_at = datetime('now') WHERE informe_pago_id = ?")
        .bind(informe_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    // Vincular nuevos requerimientos
    for req in requerimientos {
        if let Some(req_id) = req.get("id").and_then(|v| v.as_i64()) {
            sqlx::query("UPDATE requerimientos SET informe_pago_id = ?, estado = 'en_informe', updated_at = datetime('now') WHERE id = ?")
                .bind(informe_id)
                .bind(req_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
