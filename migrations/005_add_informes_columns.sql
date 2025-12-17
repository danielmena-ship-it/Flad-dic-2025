-- Migración 005: Columnas informes_pago
-- Fecha: 2025-12-17
-- Objetivo: Agregar columnas faltantes en informes_pago

ALTER TABLE informes_pago ADD COLUMN neto REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN utilidades REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN iva REAL DEFAULT 0;
ALTER TABLE informes_pago ADD COLUMN total_pagar REAL DEFAULT 0;
