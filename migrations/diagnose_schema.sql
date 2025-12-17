-- Script de diagnóstico: Verificar estructura completa de DB
-- Ejecutar con: sqlite3 flad.db < diagnose_schema.sql

.mode column
.headers on

SELECT '=== TABLAS EXISTENTES ===' as info;
SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;

SELECT '' as info;
SELECT '=== COLUMNAS: requerimientos ===' as info;
PRAGMA table_info(requerimientos);

SELECT '' as info;
SELECT '=== COLUMNAS: informes_pago ===' as info;
PRAGMA table_info(informes_pago);

SELECT '' as info;
SELECT '=== COLUMNAS: ordenes_trabajo ===' as info;
PRAGMA table_info(ordenes_trabajo);

SELECT '' as info;
SELECT '=== COLUMNAS: jardines ===' as info;
PRAGMA table_info(jardines);

SELECT '' as info;
SELECT '=== COLUMNAS: partidas ===' as info;
PRAGMA table_info(partidas);

SELECT '' as info;
SELECT '=== COLUMNAS: configuracion_contrato ===' as info;
PRAGMA table_info(configuracion_contrato);

SELECT '' as info;
SELECT '=== ÍNDICES EXISTENTES ===' as info;
SELECT name, tbl_name, sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL ORDER BY tbl_name, name;

SELECT '' as info;
SELECT '=== TRIGGERS EXISTENTES ===' as info;
SELECT name, tbl_name FROM sqlite_master WHERE type='trigger' ORDER BY tbl_name, name;
