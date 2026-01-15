-- Migración 007: Corregir lógica dias_atraso → multa

-- 1. Eliminar triggers antiguos
DROP TRIGGER IF EXISTS calcular_multa_insert;
DROP TRIGGER IF EXISTS calcular_multa_update;

-- 2. Crear triggers corregidos (dias_atraso PRIMERO, multa SEGUNDO)
CREATE TRIGGER calcular_multa_insert
AFTER INSERT ON requerimientos
WHEN NEW.fecha_recepcion IS NOT NULL
BEGIN
    -- Primero: Calcular dias_atraso
    UPDATE requerimientos 
    SET dias_atraso = CASE 
            WHEN NEW.fecha_limite IS NOT NULL AND NEW.fecha_recepcion > NEW.fecha_limite
            THEN CAST(julianday(NEW.fecha_recepcion) - julianday(NEW.fecha_limite) AS INTEGER)
            ELSE 0
        END
    WHERE id = NEW.id;
    
    -- Segundo: Calcular multa usando dias_atraso
    UPDATE requerimientos 
    SET multa = CASE 
            WHEN dias_atraso > 0 AND (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_observacion, 0)) > 0
            THEN MAX(
                ROUND(dias_atraso * 7500),
                ROUND(dias_atraso * (NEW.precio_total / (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_observacion, 0))))
            )
            ELSE 0
        END
    WHERE id = NEW.id;
END;

CREATE TRIGGER calcular_multa_update
AFTER UPDATE OF fecha_recepcion, fecha_limite, fecha_inicio, plazo_dias, plazo_observacion, precio_total ON requerimientos
BEGIN
    -- Primero: Calcular dias_atraso
    UPDATE requerimientos 
    SET dias_atraso = CASE 
            WHEN NEW.fecha_recepcion IS NOT NULL AND NEW.fecha_limite IS NOT NULL AND NEW.fecha_recepcion > NEW.fecha_limite
            THEN CAST(julianday(NEW.fecha_recepcion) - julianday(NEW.fecha_limite) AS INTEGER)
            ELSE 0
        END
    WHERE id = NEW.id;
    
    -- Segundo: Calcular multa usando dias_atraso
    UPDATE requerimientos 
    SET multa = CASE 
            WHEN dias_atraso > 0 AND (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_observacion, 0)) > 0
            THEN MAX(
                ROUND(dias_atraso * 7500),
                ROUND(dias_atraso * (NEW.precio_total / (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_observacion, 0))))
            )
            ELSE 0
        END
    WHERE id = NEW.id;
END;

-- 3. Actualizar registros existentes
UPDATE requerimientos 
SET dias_atraso = CASE 
        WHEN fecha_recepcion IS NOT NULL AND fecha_limite IS NOT NULL AND fecha_recepcion > fecha_limite
        THEN CAST(julianday(fecha_recepcion) - julianday(fecha_limite) AS INTEGER)
        ELSE 0
    END
WHERE fecha_recepcion IS NOT NULL;

UPDATE requerimientos 
SET multa = CASE 
        WHEN dias_atraso > 0 AND (COALESCE(plazo_dias, 0) + COALESCE(plazo_observacion, 0)) > 0
        THEN MAX(
            ROUND(dias_atraso * 7500),
            ROUND(dias_atraso * (precio_total / (COALESCE(plazo_dias, 0) + COALESCE(plazo_observacion, 0))))
        )
        ELSE 0
    END
WHERE fecha_recepcion IS NOT NULL;
