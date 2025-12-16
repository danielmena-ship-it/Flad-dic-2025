<script>
  import { db } from '$lib/api/tauri';
  import { configuracion } from '$lib/stores/configuracion';
  import { createEventDispatcher } from 'svelte';

  export let visible = false;
  export let nombreActual = '';
  export let firmaActual = null;

  const dispatch = createEventDispatcher();

  let nombre = nombreActual;
  let firmaInput;
  let firmaPreview = firmaActual;
  let guardando = false;
  let mensaje = '';

  $: if (visible) {
    nombre = nombreActual;
    firmaPreview = firmaActual;
  }

  async function redimensionarImagen(file, maxWidth = 1000, maxHeight = 1000) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        const img = new Image();
        img.onload = () => {
          let width = img.width;
          let height = img.height;
          
          if (width > maxWidth || height > maxHeight) {
            const ratio = Math.min(maxWidth / width, maxHeight / height);
            width = width * ratio;
            height = height * ratio;
          }
          
          const canvas = document.createElement('canvas');
          canvas.width = width;
          canvas.height = height;
          const ctx = canvas.getContext('2d');
          ctx.drawImage(img, 0, 0, width, height);
          
          resolve(canvas.toDataURL('image/png'));
        };
        img.onerror = reject;
        img.src = e.target.result;
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  async function handleFirmaSelected(event) {
    const file = event.target.files[0];
    if (!file) return;

    if (!file.type.startsWith('image/')) {
      mensaje = '❌ Solo se permiten imágenes';
      return;
    }

    try {
      firmaPreview = await redimensionarImagen(file);
      mensaje = '';
    } catch (error) {
      mensaje = '❌ Error al procesar imagen';
    }
  }

  function seleccionarFirma() {
    firmaInput.click();
  }

  function eliminarFirma() {
    firmaPreview = null;
    if (firmaInput) firmaInput.value = '';
  }

  async function guardar() {
    if (!nombre.trim()) {
      mensaje = '⚠️ Ingrese nombre del ITO';
      return;
    }

    try {
      guardando = true;
      mensaje = '';

      const configActual = await db.configuracion.get();
      
      await db.configuracion.update({
        titulo: configActual.titulo || 'FLAD',
        contratista: configActual.contratista || '',
        itoNombre: nombre.trim(),
        prefijoCorrelativo: configActual.prefijoCorrelativo || ''
      });
      
      // Guardar firma usando API de importación
      if (firmaPreview) {
        await db.importar.firma(firmaPreview.split(',')[1]); // Remover prefijo data:image
      }

      // Recargar configuración en el store
      await configuracion.cargar();

      dispatch('guardado', { nombre: nombre.trim(), firma: firmaPreview });
      cerrar();
    } catch (error) {
      mensaje = '❌ Error al guardar: ' + error.message;
    } finally {
      guardando = false;
    }
  }

  function cerrar() {
    visible = false;
    mensaje = '';
  }
</script>

{#if visible}
  <div 
    class="modal-overlay" 
    role="presentation"
    on:click={(e) => e.target === e.currentTarget && cerrar()}
  >
    <div 
      class="modal-content" 
      role="dialog"
      aria-modal="true"
    >
      <div class="modal-header">
        <h2>Inspector Técnico de Obra (ITO)</h2>
        <button class="btn-cerrar" on:click={cerrar}>×</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="nombre-ito">Nombre ITO</label>
          <input 
            id="nombre-ito"
            type="text" 
            bind:value={nombre}
            placeholder="Ej: Juan Pérez González"
            disabled={guardando}
          />
        </div>

        <div class="form-group">
          <div class="form-label">Firma ITO</div>
          
          {#if firmaPreview}
            <div class="firma-preview">
              <img src={firmaPreview} alt="Firma ITO" />
              <button class="btn-eliminar" on:click={eliminarFirma} disabled={guardando}>
                Eliminar firma
              </button>
            </div>
          {:else}
            <button class="btn-cargar" on:click={seleccionarFirma} disabled={guardando}>
              📎 Seleccionar imagen de firma
            </button>
          {/if}

          <input 
            type="file" 
            accept="image/*"
            bind:this={firmaInput}
            on:change={handleFirmaSelected}
            style="display: none"
          />
        </div>

        {#if mensaje}
          <div class="mensaje {mensaje.includes('✅') ? 'exito' : 'error'}">
            {mensaje}
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn-secundario" on:click={cerrar} disabled={guardando}>
          Cancelar
        </button>
        <button class="btn-primario" on:click={guardar} disabled={guardando}>
          {guardando ? 'Guardando...' : 'Guardar'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    backdrop-filter: blur(3px);
  }

  .modal-content {
    background: #1a2332;
    border-radius: 12px;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    border: 1px solid #2d3e50;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid #2d3e50;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: #e0e6ed;
  }

  .btn-cerrar {
    background: none;
    border: none;
    font-size: 32px;
    color: #a8c5e0;
    cursor: pointer;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .btn-cerrar:hover {
    background: #2d3e50;
    color: #ffffff;
  }

  .modal-body {
    padding: 24px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: #a8c5e0;
    font-size: 14px;
  }

  .form-group input[type="text"] {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #2d3e50;
    background: #0f1419;
    color: #e0e6ed;
    border-radius: 8px;
    font-size: 15px;
    transition: all 0.2s;
  }

  .form-group input[type="text"]:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
  }

  .firma-preview {
    border: 2px solid #2d3e50;
    border-radius: 8px;
    padding: 16px;
    text-align: center;
    background: #ffffff !important;
  }

  .firma-preview img {
    max-width: 100%;
    max-height: 200px;
    margin-bottom: 12px;
    border-radius: 4px;
  }

  .btn-cargar, .btn-eliminar {
    padding: 10px 20px;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 14px;
  }

  .btn-cargar {
    background: #2d3e50;
    border: 2px dashed #4a5f7f;
    color: #a8c5e0;
    width: 100%;
  }

  .btn-cargar:hover {
    background: #3d4e60;
    border-color: #5a7fa0;
  }

  .btn-eliminar {
    background: #7f1d1d;
    border: 1px solid #991b1b;
    color: #fecaca;
  }

  .btn-eliminar:hover {
    background: #991b1b;
  }

  .mensaje {
    padding: 12px;
    border-radius: 8px;
    margin-top: 12px;
    font-size: 14px;
  }

  .mensaje.exito {
    background: #065f46;
    color: #d1fae5;
    border: 1px solid #10b981;
  }

  .mensaje.error {
    background: #7f1d1d;
    color: #fecaca;
    border: 1px solid #dc2626;
  }

  .modal-footer {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    padding: 16px 24px;
    border-top: 1px solid #2d3e50;
  }

  .btn-secundario, .btn-primario {
    padding: 10px 24px;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 14px;
  }

  .btn-secundario {
    background: #2d3e50;
    border: 1px solid #4a5f7f;
    color: #a8c5e0;
  }

  .btn-secundario:hover {
    background: #3d4e60;
  }

  .btn-primario {
    background: #3b82f6;
    border: 1px solid #3b82f6;
    color: white;
  }

  .btn-primario:hover {
    background: #2563eb;
  }

  .btn-secundario:disabled,
  .btn-primario:disabled,
  .btn-cargar:disabled,
  .btn-eliminar:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
