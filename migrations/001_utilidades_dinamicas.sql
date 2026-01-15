-- Migración 001: Utilidades dinámicas
-- Cambio: De % utilidades fijo (25%) a % definido en configuración

-- 1. Agregar columna en configuracion_contrato
ALTER TABLE configuracion_contrato ADD COLUMN porcentaje_utilidades REAL NOT NULL DEFAULT 0.25;

-- 2. Agregar columnas en requerimientos
ALTER TABLE requerimientos ADD COLUMN a_pago REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN utilidades REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN iva REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN total_linea REAL DEFAULT 0;

-- 3. Recalcular valores existentes (usando 25% fijo)
UPDATE requerimientos
SET 
  a_pago = precio_total - COALESCE(multa, 0),
  utilidades = ROUND((precio_total - COALESCE(multa, 0)) * 0.25),
  iva = ROUND((precio_total - COALESCE(multa, 0) + ROUND((precio_total - COALESCE(multa, 0)) * 0.25)) * 0.19),
  total_linea = (precio_total - COALESCE(multa, 0)) + 
                ROUND((precio_total - COALESCE(multa, 0)) * 0.25) + 
                ROUND((precio_total - COALESCE(multa, 0) + ROUND((precio_total - COALESCE(multa, 0)) * 0.25)) * 0.19)
WHERE precio_total > 0;

-- 4. Recrear tabla informes_pago con nuevo nombre de columna
CREATE TABLE informes_pago_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  codigo TEXT NOT NULL UNIQUE,
  jardin_codigo TEXT NOT NULL,
  fecha_creacion TEXT NOT NULL,
  neto REAL NOT NULL DEFAULT 0,
  utilidades REAL NOT NULL DEFAULT 0,
  iva REAL NOT NULL DEFAULT 0,
  total_pagar REAL NOT NULL DEFAULT 0,
  observaciones TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (jardin_codigo) REFERENCES jardines(codigo)
);

INSERT INTO informes_pago_new (id, codigo, jardin_codigo, fecha_creacion, neto, utilidades, iva, total_pagar, observaciones, created_at, updated_at)
SELECT id, codigo, jardin_codigo, fecha_creacion, neto, utilidades, iva, total_final, observaciones, created_at, updated_at
FROM informes_pago;

DROP TABLE informes_pago;
ALTER TABLE informes_pago_new RENAME TO informes_pago;

-- 5. Recalcular informes
UPDATE informes_pago
SET 
  neto = (SELECT COALESCE(SUM(a_pago), 0) FROM requerimientos WHERE informe_pago_id = informes_pago.id),
  utilidades = (SELECT COALESCE(SUM(utilidades), 0) FROM requerimientos WHERE informe_pago_id = informes_pago.id),
  iva = (SELECT COALESCE(SUM(iva), 0) FROM requerimientos WHERE informe_pago_id = informes_pago.id),
  total_pagar = (SELECT COALESCE(SUM(total_linea), 0) FROM requerimientos WHERE informe_pago_id = informes_pago.id);
