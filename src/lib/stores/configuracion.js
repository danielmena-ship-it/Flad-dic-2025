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
        // Timeout 3 segundos
        const timeoutPromise = new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Timeout cargar config')), 3000)
        );
        
        const loadPromise = (async () => {
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
        })();
        
        await Promise.race([loadPromise, timeoutPromise]);
      } catch (error) {
        console.error('Error cargando configuración:', error);
        // Valores por defecto
        set({
          titulo: 'FLAD',
          contratista: '',
          itoNombre: '',
          prefijoCorrelativo: '',
          porcentajeUtilidades: 0.25,
          firmaBase64: null
        });
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
