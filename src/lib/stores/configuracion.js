import { writable } from 'svelte/store';
import { db } from '$lib/api/tauri';

// Store reactivo para configuración
function crearConfigStore() {
  const { subscribe, set, update } = writable({
    titulo: 'FLAD',
    contratista: '',
    itoNombre: '',
    prefijoCorrelativo: '',
    porcentajeUtilidades: 0.25,
    firmaBase64: null
  });

  return {
    subscribe,
    // Cargar configuración desde BD
    async cargar() {
      try {
        const config = await db.configuracion.get();
        const firmaBase64 = await db.importar.getFirma();
        
        set({
          titulo: config.titulo || 'FLAD',
          contratista: config.contratista || '',
          itoNombre: config.itoNombre || '',
          prefijoCorrelativo: config.prefijoCorrelativo || '',
          porcentajeUtilidades: config.porcentajeUtilidades || 0.25,
          firmaBase64: firmaBase64 || null
        });
      } catch (error) {
        console.error('Error cargando configuración:', error);
      }
    },
    // Actualizar solo ITO
    actualizarITO(nombre, firmaBase64) {
      update(cfg => ({ ...cfg, itoNombre: nombre, firmaBase64 }));
    },
    // Actualizar título
    actualizarTitulo(titulo) {
      update(cfg => ({ ...cfg, titulo }));
    }
  };
}

export const configuracion = crearConfigStore();
