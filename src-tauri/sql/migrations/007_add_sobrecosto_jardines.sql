-- Migración 007: Agregar sobre_costo a jardines
-- Sobre costo es un porcentaje adicional al neto antes de aplicar utilidades

ALTER TABLE jardines ADD COLUMN sobre_costo REAL DEFAULT 0.0;

-- Valores de prueba
UPDATE jardines SET sobre_costo = 10.0 WHERE codigo = 'BB';
UPDATE jardines SET sobre_costo = 15.0 WHERE codigo = 'CB';
UPDATE jardines SET sobre_costo = 20.0 WHERE codigo = 'DR';
UPDATE jardines SET sobre_costo = 25.0 WHERE codigo = 'LA';
