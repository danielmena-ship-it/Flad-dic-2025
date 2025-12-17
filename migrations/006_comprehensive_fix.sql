-- Migración 006: Fix Comprehensivo - Todas las columnas faltantes
-- Fecha: 2025-12-17
-- Objetivo: Asegurar que TODAS las tablas tengan TODAS las columnas del schema actual

-- REQUERIMIENTOS: Columnas de cálculo y estado
ALTER TABLE requerimientos ADD COLUMN plazo_total INTEGER DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN fecha_limite TEXT;
ALTER TABLE requerimientos ADD COLUMN fecha_registro TEXT DEFAULT (datetime('now'));
ALTER TABLE requerimientos ADD COLUMN fecha_recepcion TEXT;
ALTER TABLE requerimientos ADD COLUMN multa REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN a_pago REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN utilidades REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN iva REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN total_linea REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN descripcion TEXT;
ALTER TABLE requerimientos ADD COLUMN observaciones TEXT;
ALTER TABLE requerimientos ADD COLUMN estado TEXT DEFAULT 'pendiente';
ALTER TABLE requerimientos ADD COLUMN ot_id INTEGER REFERENCES ordenes_trabajo(id);
ALTER TABLE requerimientos ADD COLUMN informe_pago_id INTEGER REFERENCES informes_pago(id);
ALTER TABLE requerimientos ADD COLUMN created_at TEXT DEFAULT (datetime('now'));
ALTER TABLE requerimientos ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));

-- INFORMES_PAGO: Columnas de totales
ALTER TABLE informes_pago ADD COLUMN neto REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN utilidades REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN iva REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN total_pagar REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN observaciones TEXT;
ALTER TABLE informes_pago ADD COLUMN created_at TEXT DEFAULT (datetime('now'));
ALTER TABLE informes_pago ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));

-- ORDENES_TRABAJO: Timestamps
ALTER TABLE ordenes_trabajo ADD COLUMN created_at TEXT DEFAULT (datetime('now'));
ALTER TABLE ordenes_trabajo ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));

-- JARDINES: Timestamps
ALTER TABLE jardines ADD COLUMN created_at TEXT DEFAULT (datetime('now'));
ALTER TABLE jardines ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));

-- PARTIDAS: Timestamps
ALTER TABLE partidas ADD COLUMN created_at TEXT DEFAULT (datetime('now'));
ALTER TABLE partidas ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));

-- CONFIGURACION_CONTRATO: Campos completos
ALTER TABLE configuracion_contrato ADD COLUMN titulo TEXT DEFAULT 'Contrato Mantención';
ALTER TABLE configuracion_contrato ADD COLUMN prefijo_correlativo TEXT DEFAULT 'M';
ALTER TABLE configuracion_contrato ADD COLUMN contratista TEXT DEFAULT '';
ALTER TABLE configuracion_contrato ADD COLUMN created_at TEXT DEFAULT (datetime('now'));
ALTER TABLE configuracion_contrato ADD COLUMN updated_at TEXT DEFAULT (datetime('now'));
