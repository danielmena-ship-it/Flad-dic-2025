-- SSOL: Schema corregido

-- CONFIGURACIÓN
CREATE TABLE IF NOT EXISTS configuracion_contrato (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    titulo TEXT NOT NULL DEFAULT 'Contrato Mantención',
    prefijo_correlativo TEXT NOT NULL DEFAULT 'M',
    contratista TEXT NOT NULL DEFAULT '',
    porcentaje_utilidades REAL NOT NULL DEFAULT 0.25,
    ito_nombre TEXT,
    firma_png BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- JARDINES
CREATE TABLE IF NOT EXISTS jardines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo TEXT NOT NULL UNIQUE,
    nombre TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- PARTIDAS
CREATE TABLE IF NOT EXISTS partidas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item TEXT NOT NULL UNIQUE,
    partida TEXT NOT NULL,
    unidad TEXT,
    precio_unitario REAL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- RECINTOS
CREATE TABLE IF NOT EXISTS recintos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    jardin_codigo TEXT NOT NULL,
    nombre TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (jardin_codigo) REFERENCES jardines(codigo) ON DELETE CASCADE
);

-- ÓRDENES DE TRABAJO (antes de requerimientos)
CREATE TABLE IF NOT EXISTS ordenes_trabajo (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo TEXT NOT NULL UNIQUE,
    jardin_codigo TEXT NOT NULL,
    fecha_creacion TEXT NOT NULL,
    observaciones TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (jardin_codigo) REFERENCES jardines(codigo) ON DELETE CASCADE
);

-- INFORMES DE PAGO (antes de requerimientos)
CREATE TABLE IF NOT EXISTS informes_pago (
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
    FOREIGN KEY (jardin_codigo) REFERENCES jardines(codigo) ON DELETE CASCADE
);

-- REQUERIMIENTOS (después de OT e Informes)
CREATE TABLE IF NOT EXISTS requerimientos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    jardin_codigo TEXT NOT NULL,
    recinto TEXT,
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
    plazo_adicional INTEGER DEFAULT 0,
    plazo_total INTEGER DEFAULT 0,
    fecha_limite TEXT,
    multa REAL DEFAULT 0,
    a_pago REAL DEFAULT 0,
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

-- ÍNDICES
CREATE INDEX IF NOT EXISTS idx_jardines_codigo ON jardines(codigo);
CREATE INDEX IF NOT EXISTS idx_partidas_item ON partidas(item);
CREATE INDEX IF NOT EXISTS idx_recintos_jardin ON recintos(jardin_codigo);
CREATE INDEX IF NOT EXISTS idx_req_jardin ON requerimientos(jardin_codigo);
CREATE INDEX IF NOT EXISTS idx_req_estado ON requerimientos(estado);
CREATE INDEX IF NOT EXISTS idx_req_jardin_estado ON requerimientos(jardin_codigo, estado);
CREATE INDEX IF NOT EXISTS idx_req_ot ON requerimientos(ot_id);
CREATE INDEX IF NOT EXISTS idx_req_informe ON requerimientos(informe_pago_id);
CREATE INDEX IF NOT EXISTS idx_req_partida ON requerimientos(partida_item);
CREATE INDEX IF NOT EXISTS idx_ot_jardin ON ordenes_trabajo(jardin_codigo);
CREATE INDEX IF NOT EXISTS idx_ot_codigo ON ordenes_trabajo(codigo);
CREATE INDEX IF NOT EXISTS idx_informe_jardin ON informes_pago(jardin_codigo);
CREATE INDEX IF NOT EXISTS idx_informe_codigo ON informes_pago(codigo);

-- DATOS INICIALES
INSERT OR IGNORE INTO configuracion_contrato (id, titulo, prefijo_correlativo, contratista) 
VALUES (1, 'Contrato Mantención', 'M', '');

-- TRIGGERS
CREATE TRIGGER IF NOT EXISTS actualizar_plazo_total_insert
AFTER INSERT ON requerimientos
BEGIN
    UPDATE requerimientos 
    SET plazo_total = COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0),
        fecha_limite = CASE 
            WHEN NEW.fecha_inicio IS NOT NULL AND (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) > 0
            THEN date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days')
            ELSE NULL
        END,
        precio_total = COALESCE(NEW.cantidad, 0) * COALESCE(NEW.precio_unitario, 0)
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS actualizar_plazo_total_update
AFTER UPDATE OF plazo_dias, plazo_adicional, fecha_inicio, cantidad, precio_unitario ON requerimientos
BEGIN
    UPDATE requerimientos 
    SET plazo_total = COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0),
        fecha_limite = CASE 
            WHEN NEW.fecha_inicio IS NOT NULL AND (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) > 0
            THEN date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days')
            ELSE NULL
        END,
        precio_total = COALESCE(NEW.cantidad, 0) * COALESCE(NEW.precio_unitario, 0)
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS calcular_multa_insert
AFTER INSERT ON requerimientos
WHEN NEW.fecha_recepcion IS NOT NULL
BEGIN
    UPDATE requerimientos 
    SET multa = CASE 
        WHEN (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) > 0 
             AND date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days') < NEW.fecha_recepcion
        THEN MAX(
            CAST(julianday(NEW.fecha_recepcion) - julianday(date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days')) AS INTEGER) * 7500,
            CAST(julianday(NEW.fecha_recepcion) - julianday(date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days')) AS INTEGER) * (NEW.precio_total / (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)))
        )
        ELSE 0
    END
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS calcular_multa_update
AFTER UPDATE OF fecha_recepcion, fecha_inicio, plazo_dias, plazo_adicional, precio_total ON requerimientos
BEGIN
    UPDATE requerimientos 
    SET multa = CASE 
        WHEN NEW.fecha_recepcion IS NOT NULL 
             AND (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) > 0 
             AND date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days') < NEW.fecha_recepcion
        THEN MAX(
            CAST(julianday(NEW.fecha_recepcion) - julianday(date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days')) AS INTEGER) * 7500,
            CAST(julianday(NEW.fecha_recepcion) - julianday(date(NEW.fecha_inicio, '+' || (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)) || ' days')) AS INTEGER) * (NEW.precio_total / (COALESCE(NEW.plazo_dias, 0) + COALESCE(NEW.plazo_adicional, 0)))
        )
        ELSE 0
    END
    WHERE id = NEW.id;
END;
