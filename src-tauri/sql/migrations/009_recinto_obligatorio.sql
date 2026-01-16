-- Migración: Hacer recinto obligatorio

-- 1. Actualizar requerimientos existentes sin recinto (poner valor por defecto temporal)
UPDATE requerimientos SET recinto = 'SIN ESPECIFICAR' WHERE recinto IS NULL OR recinto = '';

-- 2. Crear tabla temporal con nueva estructura
CREATE TABLE requerimientos_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    jardin_codigo TEXT NOT NULL,
    recinto TEXT NOT NULL,
    partida_item TEXT NOT NULL,
    cantidad REAL NOT NULL DEFAULT 0,
    precio_unitario REAL NOT NULL DEFAULT 0,
    precio_total REAL NOT NULL DEFAULT 0,
    fecha_inicio TEXT NOT NULL,
    fecha_registro TEXT NOT NULL,
    estado TEXT NOT NULL DEFAULT 'pendiente',
    ot_id INTEGER,
    informe_pago_id INTEGER,
    fecha_recepcion TEXT,
    plazo_dias INTEGER DEFAULT 0,
    plazo_observacion INTEGER DEFAULT 0,
    plazo_total INTEGER DEFAULT 0,
    fecha_limite TEXT,
    multa REAL DEFAULT 0,
    a_pago REAL DEFAULT 0,
    sobre_costo REAL DEFAULT 0,
    utilidades REAL DEFAULT 0,
    iva REAL DEFAULT 0,
    total_linea REAL DEFAULT 0,
    descripcion TEXT,
    observaciones TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (jardin_codigo) REFERENCES jardines(codigo) ON DELETE CASCADE,
    FOREIGN KEY (partida_item) REFERENCES partidas(item) ON DELETE CASCADE,
    FOREIGN KEY (ot_id) REFERENCES ordenes_trabajo(id) ON DELETE SET NULL,
    FOREIGN KEY (informe_pago_id) REFERENCES informes_pago(id) ON DELETE SET NULL
);

-- 3. Copiar datos
INSERT INTO requerimientos_new SELECT * FROM requerimientos;

-- 4. Eliminar tabla antigua
DROP TABLE requerimientos;

-- 5. Renombrar tabla nueva
ALTER TABLE requerimientos_new RENAME TO requerimientos;

-- 6. Recrear índices
CREATE INDEX idx_requerimientos_jardin ON requerimientos(jardin_codigo);
CREATE INDEX idx_requerimientos_ot ON requerimientos(ot_id);
CREATE INDEX idx_requerimientos_ip ON requerimientos(informe_pago_id);
CREATE INDEX idx_requerimientos_estado ON requerimientos(estado);
