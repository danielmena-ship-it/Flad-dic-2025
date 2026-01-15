-- Migración 009: Agregar columna estado a ordenes_trabajo
-- Fecha: 2026-01-06

-- Agregar columna estado con valor por defecto
ALTER TABLE ordenes_trabajo ADD COLUMN estado TEXT NOT NULL DEFAULT 'Inicial' CHECK(estado IN ('Inicial', 'Rectificada', 'Observaciones'));

SELECT 'Migración 009 completada. Columna estado agregada a ordenes_trabajo.' AS mensaje;