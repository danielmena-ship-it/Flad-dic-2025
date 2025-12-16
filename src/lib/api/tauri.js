/**
 * Cliente Tauri directo - Sin capas de compatibilidad
 * Comunicación directa con comandos Rust
 */

import { invoke } from '@tauri-apps/api/core';

// Transformar camelCase → snake_case
function toSnake(obj) {
  if (!obj || typeof obj !== 'object') return obj;
  if (Array.isArray(obj)) return obj.map(toSnake);
  
  const result = {};
  for (const [key, value] of Object.entries(obj)) {
    const snakeKey = key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
    result[snakeKey] = toSnake(value);
  }
  return result;
}

// Transformar snake_case → camelCase
function toCamel(obj) {
  if (!obj || typeof obj !== 'object') return obj;
  if (Array.isArray(obj)) return obj.map(toCamel);
  
  const result = {};
  for (const [key, value] of Object.entries(obj)) {
    const camelKey = key.replace(/_([a-z])/g, (_, l) => l.toUpperCase());
    result[camelKey] = toCamel(value);
  }
  return result;
}

// API Cliente - TODOS los parámetros en snake_case
export const db = {
  // Jardines
  jardines: {
    getAll: async () => toCamel(await invoke('get_jardines')),
    getByCode: async (codigo) => toCamel(await invoke('get_jardin_by_codigo', { codigo })),
    add: (jardin) => invoke('add_jardin', toSnake(jardin))
  },

  // Partidas
  partidas: {
    getAll: async () => toCamel(await invoke('get_partidas')),
    add: (partida) => invoke('add_partida', toSnake(partida))
  },

  // Requerimientos
  requerimientos: {
    getAll: async () => toCamel(await invoke('get_requerimientos')),
    add: (req) => invoke('add_requerimiento', toSnake(req)),
    update: (id, data) => invoke('update_requerimiento', { id, ...toSnake(data) }),
    delete: (id) => invoke('delete_requerimiento', { id })
  },

  // Recintos
  recintos: {
    getAll: async () => toCamel(await invoke('get_recintos')),
    getByJardin: async (jardinCodigo) => toCamel(await invoke('get_recintos_by_jardin', { jardin_codigo: jardinCodigo })),
    add: (recinto) => invoke('add_recinto', toSnake(recinto))
  },

  // Órdenes de Trabajo
  ordenesTrabajo: {
    getAll: async () => toCamel(await invoke('get_ordenes_trabajo')),
    getDetalle: async (otId) => toCamel(await invoke('get_orden_trabajo_detalle', { ot_id: otId })),
    crear: (data) => invoke('crear_orden_trabajo', toSnake(data)),
    update: (otId, data) => invoke('update_orden_trabajo', { ot_id: otId, ...toSnake(data) }),
    eliminar: (id) => invoke('eliminar_orden_trabajo', { ot_id: id })
  },

  // Informes de Pago
  informesPago: {
    getAll: async () => toCamel(await invoke('get_informes_pago')),
    getDetalle: async (informeId) => toCamel(await invoke('get_informe_pago_detalle', { informe_id: informeId })),
    getRequerimientosParaInforme: async (jardinCodigo) => toCamel(await invoke('get_requerimientos_para_informe', { jardin_codigo: jardinCodigo })),
    crear: (data) => invoke('crear_informe_pago', toSnake(data)),
    update: (informeId, data) => invoke('update_informe_pago', {
      informe_id: informeId,
      ...toSnake(data)
    }),
    eliminar: (id) => invoke('eliminar_informe_pago', { informe_id: id })
  },

  // Configuración
  configuracion: {
    get: async () => toCamel(await invoke('get_configuracion')),
    update: (data) => invoke('update_configuracion', toSnake(data))
  },

  // Importar
  importar: {
    catalogoJson: (data) => invoke('importar_catalogo_json', { 
      jsonStr: typeof data === 'string' ? data : JSON.stringify(data) 
    }),
    catalogoCsv: (csvStr, tipo) => invoke('importar_catalogo_csv', { csvStr, tipo }),
    catalogoXlsx: (filePath, sheetName, tipo) => invoke('importar_catalogo_xlsx', { 
      filePath, sheetName, tipo 
    }),
    catalogoXlsxBytes: (fileBytes) => invoke('importar_catalogo_xlsx_bytes', { 
      file_bytes: fileBytes  // ✅ FIXED: snake_case para match con comando Tauri
    }),
    baseDatosCompleta: (jsonStr) => invoke('importar_base_datos_completa', {
      json_str: typeof jsonStr === 'string' ? jsonStr : JSON.stringify(jsonStr)  // ✅ FIXED: snake_case
    }),
    firma: (imagenBase64) => invoke('importar_firma', { imagenBase64 }),
    getFirma: async () => toCamel(await invoke('get_firma')),
    clearAll: () => invoke('clear_all')
  }
};
