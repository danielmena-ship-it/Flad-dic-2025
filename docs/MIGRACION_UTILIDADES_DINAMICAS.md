# MIGRACIÓN: UTILIDADES DINÁMICAS POR CATÁLOGO

**Proyecto**: FLAD Gestión  
**Fecha**: 2024-12-11  
**Estado**: EJECUTADO
**Objetivo**: Cambiar utilidades de % fijo (25%) a % definido en configuración global

---

## RESUMEN EJECUTIVO

### Cambio conceptual
- **ANTES**: Utilidades = 25% fijo en `calculos.js` → cálculo a nivel informe
- **DESPUÉS**: Utilidades = % en `configuracion_contrato.porcentaje_utilidades` → cálculo por línea

### Fórmula nueva por línea
```
a_pago = precio_total - multa
utilidades = a_pago × porcentaje_utilidades_config
iva = (a_pago + utilidades) × 0.19
total_linea = a_pago + utilidades + iva
```

### Informe de pago (agregación)
```
neto = Σ a_pago
utilidades = Σ utilidades_linea
iva = Σ iva_linea
total_pagar = Σ total_linea
```

---

## CAMBIOS REALIZADOS

### 1. BASE DE DATOS

#### 1.1 Tabla `configuracion_contrato`
```sql
ALTER TABLE configuracion_contrato 
ADD COLUMN porcentaje_utilidades REAL NOT NULL DEFAULT 0.25;
```

**Ubicación**: Después de `contratista` (línea ~8 schema.sql)

#### 1.2 Tabla `requerimientos`
```sql
ALTER TABLE requerimientos ADD COLUMN a_pago REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN utilidades REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN iva REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN total_linea REAL DEFAULT 0;
```

**Ubicación**: Después de `multa` (línea ~88 schema.sql)

#### 1.3 Tabla `informes_pago`
```sql
-- Recrear tabla (SQLite no soporta RENAME COLUMN)
-- total_final → total_pagar
```

**Nota**: Se recrea tabla completa con nuevo nombre de columna

---

### 2. MIGRACIÓN SQL

**Archivo**: `src-tauri/sql/migrations/001_utilidades_dinamicas.sql`

#### Proceso:
1. Agregar columna en `configuracion_contrato`
2. Agregar 4 columnas en `requerimientos`
3. Recalcular valores existentes (usando 25% retroactivo)
4. Recrear `informes_pago` con `total_pagar`
5. Recalcular informes por agregación

**Cálculo retroactivo**:
```sql
UPDATE requerimientos SET 
  a_pago = precio_total - COALESCE(multa, 0),
  utilidades = ROUND((precio_total - COALESCE(multa, 0)) * 0.25),
  iva = ROUND((a_pago + utilidades) * 0.19),
  total_linea = a_pago + utilidades + iva
WHERE precio_total > 0;
```

---

### 3. STRUCTS RUST

#### 3.1 `Configuracion` (db.rs línea ~206)
```rust
pub struct Configuracion {
    pub id: i64,
    pub titulo: String,
    pub contratista: String,
    pub prefijo_correlativo: String,
    pub porcentaje_utilidades: f64,  // ← AGREGADO
    pub ito_nombre: Option<String>,
    pub ito_firma_base64: Option<String>,
}
```

#### 3.2 `Requerimiento` (db.rs línea ~93)
```rust
pub struct Requerimiento {
    // ... campos existentes ...
    pub multa: f64,
    pub a_pago: f64,           // ← AGREGADO
    pub utilidades: f64,       // ← AGREGADO
    pub iva: f64,              // ← AGREGADO
    pub total_linea: f64,      // ← AGREGADO
    pub descripcion: Option<String>,
    // ...
}
```

#### 3.3 `InformePago` (db.rs línea ~177)
```rust
pub struct InformePago {
    pub id: i64,
    pub codigo: String,
    pub jardin_codigo: String,
    pub fecha_creacion: String,
    pub neto: f64,
    pub utilidades: f64,
    pub iva: f64,
    pub total_pagar: f64,  // ← CAMBIO: de total_final
    pub observaciones: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

---

### 4. IMPORTACIÓN EXCEL

#### 4.1 Nuevo Excel: `Catalogo TIPO v2.xlsx`
**Ubicación**: `/Users/junji/Library/CloudStorage/OneDrive-JUNJI(2)/00 JUNJI/02 PROYECTOS/117 CONT MANT 2026/03 FLAD/01 Base Datos/`

**Estructura pestaña `configuracion`**:
```
titulo | prefijo_correlativo | contratista | utilidades
Contrato... | M | Constructora... | 25
```

**IMPORTANTE**: Usuario ingresa `25`, backend convierte a `0.25` (dividing by 100)

#### 4.2 Código Rust (commands.rs línea ~1310)
```rust
let utilidades = if row.len() > 3 {
    row[3].to_string().parse::<f64>().unwrap_or(25.0) / 100.0
} else {
    0.25
};

sqlx::query(
    "UPDATE configuracion_contrato 
     SET titulo = ?, prefijo_correlativo = ?, contratista = ?, 
         porcentaje_utilidades = ?, updated_at = datetime('now')
     WHERE id = 1"
)
.bind(&titulo)
.bind(&prefijo)
.bind(&contratista)
.bind(utilidades)  // ← AGREGADO
.execute(&mut *tx)
.await
```

---

### 5. CÁLCULOS JAVASCRIPT

#### 5.1 Eliminar constante obsoleta (calculos.js línea ~21)
```javascript
// ELIMINADO:
// export const PORCENTAJE_UTILIDADES = 0.25;

// MANTENER:
export const PORCENTAJE_IVA = 0.19;
```

#### 5.2 Nueva función (calculos.js línea ~230)
```javascript
/**
 * Calcula campos financieros de una línea de requerimiento
 * @param {number} aPago - Monto a pago (después de multas)
 * @param {number} porcentajeUtilidades - % de utilidades (ej: 0.25)
 * @returns {Object} { utilidades, iva, totalLinea }
 */
export function calcularLineaRequerimiento(aPago, porcentajeUtilidades) {
  const utilidades = redondearEntero(aPago * porcentajeUtilidades);
  const baseIva = aPago + utilidades;
  const iva = redondearEntero(baseIva * PORCENTAJE_IVA);
  const totalLinea = aPago + utilidades + iva;
  
  return { utilidades, iva, totalLinea };
}
```

#### 5.3 Modificar `calcularCamposInforme` (calculos.js línea ~247)
```javascript
/**
 * Calcula campos financieros de un informe de pago (AGREGACIÓN)
 * @param {Array} requerimientos - Array con: aPago, utilidades, iva, totalLinea
 * @returns {Object} { neto, utilidades, iva, totalPagar }
 */
export function calcularCamposInforme(requerimientos) {
  const neto = requerimientos.reduce((sum, req) => 
    sum + (Number(req.aPago) || 0), 0);
  const utilidades = requerimientos.reduce((sum, req) => 
    sum + (Number(req.utilidades) || 0), 0);
  const iva = requerimientos.reduce((sum, req) => 
    sum + (Number(req.iva) || 0), 0);
  const totalPagar = requerimientos.reduce((sum, req) => 
    sum + (Number(req.totalLinea) || 0), 0);
  
  return { neto, utilidades, iva, totalPagar };
}
```

---

### 6. COMANDO RUST: actualizar_fecha_recepcion

**Archivo**: `src-tauri/src/commands.rs` (línea ~236)

**CRÍTICO**: Este comando ahora calcula automáticamente los 4 campos nuevos.

```rust
pub async fn actualizar_fecha_recepcion(
    db: State<'_, DbState>,
    id: i64,
    fecha_recepcion: String,
) -> Result<(), String> {
    // 1. Obtener requerimiento y config
    let req: Requerimiento = sqlx::query_as(
        "SELECT * FROM requerimientos WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&*db.pool)
    .await?;
    
    let config: Configuracion = sqlx::query_as(
        "SELECT id, titulo, contratista, prefijo_correlativo, 
                porcentaje_utilidades, ito_nombre, 
                firma_png as ito_firma_base64 
         FROM configuracion_contrato WHERE id = 1"
    )
    .fetch_one(&*db.pool)
    .await?;
    
    // 2. Calcular campos
    let a_pago = req.precio_total - req.multa;
    let utilidades = (a_pago * config.porcentaje_utilidades).round();
    let iva = ((a_pago + utilidades) * 0.19).round();
    let total_linea = a_pago + utilidades + iva;
    
    // 3. Actualizar TODO en una transacción
    sqlx::query(
        "UPDATE requerimientos 
         SET fecha_recepcion = ?, a_pago = ?, utilidades = ?, 
             iva = ?, total_linea = ?, updated_at = datetime('now') 
         WHERE id = ?"
    )
    .bind(&fecha_recepcion)
    .bind(a_pago)
    .bind(utilidades)
    .bind(iva)
    .bind(total_linea)
    .bind(id)
    .execute(&*db.pool)
    .await?;
    
    Ok(())
}
```

---

### 7. FRONTEND (db-helpers.js)

#### Import actualizado (línea ~5)
```javascript
import { 
  calcularPlazoTotal, 
  calcularFechaLimite, 
  calcularDiasAtraso, 
  calcularMulta, 
  calcularAPago, 
  calcularLineaRequerimiento  // ← AGREGADO
} from './calculos';
```

**NOTA**: No se modificó UI. Cálculos son invisibles al usuario.

---

## FLUJO DE DATOS

### Creación de requerimiento
1. Usuario crea requerimiento → `precio_total` calculado
2. Usuario asigna a OT → sin cambios
3. Usuario registra recepción → **comando Rust calcula a_pago, utilidades, iva, total_linea**
4. Sistema guarda en BD automáticamente

### Creación de informe
1. Frontend obtiene requerimientos con campos calculados
2. `calcularCamposInforme()` suma campos por columna
3. Informe guarda: neto, utilidades, iva, total_pagar

### UI sin cambios
- Usuario NO ve columnas nuevas
- Cálculos ejecutan en backend/helpers
- Compatibilidad total con flujo existente

---

## ARCHIVOS MODIFICADOS

### Schema/Migración
- ✅ `src-tauri/sql/schema.sql`
- ✅ `src-tauri/sql/migrations/001_utilidades_dinamicas.sql` (nuevo)

### Backend Rust
- ✅ `src-tauri/src/db.rs` (structs)
- ✅ `src-tauri/src/commands.rs` (import Excel + comando fecha)

### Frontend JS
- ✅ `src/lib/utils/calculos.js`
- ✅ `src/lib/utils/db-helpers.js`

### Datos
- ✅ Excel nuevo: `Catalogo TIPO v2.xlsx`

### UI
- ❌ Sin cambios (intencional)

---

## TESTING PENDIENTE

### 1. Importar Excel v2
```bash
# Probar importación con columna utilidades
# Verificar: SELECT porcentaje_utilidades FROM configuracion_contrato
```

### 2. Crear requerimiento + recepción
```bash
# 1. Crear req → precio_total calculado
# 2. Asignar a OT
# 3. Guardar fecha recepción
# 4. Verificar BD: a_pago, utilidades, iva, total_linea
```

### 3. Crear informe
```bash
# 1. Seleccionar reqs con recepción
# 2. Crear informe
# 3. Verificar: neto, utilidades, iva, total_pagar (agregados)
```

### 4. Build producción
```bash
npm run tauri build
# Test en Windows + macOS
```

---

## ROLLBACK

```bash
# Restaurar backup
cp sistema-piloto-cont-mant_backup_YYYYMMDD_HHMMSS.db sistema-piloto-cont-mant.db

# Revertir código
git checkout HEAD -- src-tauri/sql/schema.sql
git checkout HEAD -- src-tauri/src/db.rs
git checkout HEAD -- src-tauri/src/commands.rs
git checkout HEAD -- src/lib/utils/calculos.js
```

---

## NOTAS CRÍTICAS

1. **% utilidades**: Usuario ingresa `25`, backend guarda `0.25` (división /100)
2. **Cálculo automático**: `actualizar_fecha_recepcion` calcula 4 campos
3. **UI invisible**: Cambios NO afectan experiencia de usuario
4. **Compatibilidad**: Datos antiguos recalculados con 25% retroactivo
5. **Agregación**: Informe suma campos, no recalcula desde cero

---

**Estado**: Código modificado, testing pendiente  
**Próximo paso**: Build + debug
