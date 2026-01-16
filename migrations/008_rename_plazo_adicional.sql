-- Migración 008: Renombrar plazo_adicional a plazo_observacion
-- Solo columna, triggers se actualizan en migración 007

-- Agregar nueva columna
ALTER TABLE requerimientos ADD COLUMN plazo_observacion INTEGER DEFAULT 0;

-- Copiar datos existentes
UPDATE requerimientos SET plazo_observacion = COALESCE(plazo_adicional, 0);
