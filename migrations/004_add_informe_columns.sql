-- Migración 004: Columnas para Informes de Pago
-- Fecha: 2025-12-17
-- Objetivo: Agregar columnas a_pago, utilidades, iva, total_linea

-- Agregar columnas si no existen (SQLite 3.35.0+)
ALTER TABLE requerimientos ADD COLUMN a_pago REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN utilidades REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN iva REAL DEFAULT 0;
ALTER TABLE requerimientos ADD COLUMN total_linea REAL DEFAULT 0;
