<script>
  import { createEventDispatcher, onMount } from 'svelte';
  import { getRequerimientosParaInformePago, editarInformePago, getRequerimientosPorInforme } from '$lib/utils/db-helpers.js';
  import { formatearNumero, calcularMontoTotalSeleccionados } from '$lib/utils/calculos.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';

  export let informe;

  const dispatch = createEventDispatcher();

  let requerimientosDisponibles = [];
  let requerimientosActuales = [];
  let seleccionados = new Set();
  let cargando = true;
  let mensaje = '';

  onMount(async () => {
    // Cargar requerimientos actuales del informe
    const reqsActuales = await getRequerimientosPorInforme(informe.id);
    requerimientosActuales = await enriquecerRequerimientos(reqsActuales);
    
    // Marcar como seleccionados
    requerimientosActuales.forEach(r => seleccionados.add(r.id));
    seleccionados = seleccionados;
    
    // Cargar requerimientos disponibles (sin informe) del mismo jardín
    const reqsDisponibles = await getRequerimientosParaInformePago({ 
      jardinCodigo: informe.jardinCodigo  // ✅ FIXED: camelCase para match con db-helpers
    });
    requerimientosDisponibles = await enriquecerRequerimientos(reqsDisponibles);
    
    cargando = false;
  });

  function toggleSeleccion(id) {
    if (seleccionados.has(id)) {
      seleccionados.delete(id);
    } else {
      seleccionados.add(id);
    }
    seleccionados = seleccionados;
  }

  async function guardar() {
    if (seleccionados.size === 0) {
      mensaje = '⚠️ Debes seleccionar al menos un requerimiento';
      return;
    }

    try {
      cargando = true;
      await editarInformePago(informe.id, Array.from(seleccionados));
      dispatch('cerrar');
    } catch (error) {
      mensaje = '❌ Error al actualizar informe: ' + error.message;
    } finally {
      cargando = false;
    }
  }

  function cerrar() {
    dispatch('cerrar');
  }

  function formatearCantidad(cantidad) {
    const num = Number(cantidad);
    if (isNaN(num)) return '0.00';
    return num.toFixed(2);
  }

  $: todosLosRequerimientos = [...requerimientosActuales, ...requerimientosDisponibles];
  $: montoTotal = calcularMontoTotalSeleccionados(todosLosRequerimientos, seleccionados);
</script>

<div 
  class="overlay" 
  role="presentation"
  on:click={(e) => e.target === e.currentTarget && cerrar()}
>
  <div 
    class="modal" 
    role="dialog"
    aria-modal="true"
  >
    <div class="header">
      <h2>Editar {informe.codigo}</h2>
      <button class="cerrar" on:click={cerrar}>✕</button>
    </div>

    {#if mensaje}
      <div class="mensaje {mensaje.includes('✅') ? 'exito' : 'error'}">{mensaje}</div>
    {/if}

    <div class="info">
      <p><strong>Jardín:</strong> {informe.jardinCodigo}</p>
      <p><strong>Requerimientos Seleccionados:</strong> {seleccionados.size}</p>
      <p><strong>Monto Total:</strong> ${formatearNumero(montoTotal)}</p>
    </div>

    {#if cargando && todosLosRequerimientos.length === 0}
      <p>Cargando...</p>
    {:else}
      <div class="seccion">
        <h3>Requerimientos Actuales del Informe</h3>
        {#if requerimientosActuales.length === 0}
          <p class="vacio">No hay requerimientos actuales</p>
        {:else}
          <div class="lista">
            {#each requerimientosActuales as req}
              <div class="item {seleccionados.has(req.id) ? 'seleccionado' : ''}">
                <input 
                  type="checkbox" 
                  checked={seleccionados.has(req.id)} 
                  on:change={() => toggleSeleccion(req.id)}
                />
                <div class="info-req">
                  <p><strong>{req.recinto}</strong> - {req.partidaItem} - {req.partidaNombre}</p>
                  <p class="detalle">{formatearCantidad(req.cantidad)} {req.partidaUnidad} • ${formatearNumero(req.precioTotal)}</p>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="seccion">
        <h3>Requerimientos Disponibles (sin Informe asignado)</h3>
        {#if requerimientosDisponibles.length === 0}
          <p class="vacio">No hay requerimientos disponibles para este jardín</p>
        {:else}
          <div class="lista">
            {#each requerimientosDisponibles as req}
              <div class="item {seleccionados.has(req.id) ? 'seleccionado' : ''}">
                <input 
                  type="checkbox" 
                  checked={seleccionados.has(req.id)} 
                  on:change={() => toggleSeleccion(req.id)}
                />
                <div class="info-req">
                  <p><strong>{req.recinto}</strong> - {req.partidaItem} - {req.partidaNombre}</p>
                  <p class="detalle">{formatearCantidad(req.cantidad)} {req.partidaUnidad} • ${formatearNumero(req.precioTotal)}</p>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="footer">
        <button class="btn-cancelar" on:click={cerrar} disabled={cargando}>Cancelar</button>
        <button class="btn-guardar" on:click={guardar} disabled={cargando || seleccionados.size === 0}>
          {cargando ? 'Guardando...' : 'Guardar Cambios'}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal {
    background: #1a2332;
    border-radius: 12px;
    width: 90%;
    max-width: 800px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 2px solid #2d3e50;
  }

  h2 { color: #a8c5e0; margin: 0; }

  .cerrar {
    background: none;
    border: none;
    color: #8b9eb3;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.25rem;
    line-height: 1;
  }

  .cerrar:hover { color: #a8c5e0; }

  .mensaje {
    padding: 1rem 1.5rem;
    margin: 1rem 1.5rem;
    border-radius: 6px;
    font-weight: 500;
  }

  .mensaje.exito {
    background: rgba(76, 175, 80, 0.2);
    color: #81c784;
    border: 1px solid #4caf50;
  }

  .mensaje.error {
    background: rgba(244, 67, 54, 0.2);
    color: #e57373;
    border: 1px solid #f44336;
  }

  .info {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    padding: 1rem 1.5rem;
    background: #0f1419;
    margin: 1rem 1.5rem;
    border-radius: 6px;
  }

  .info p { margin: 0; color: #a8c5e0; }
  .info strong { color: #5a8fc4; }

  .seccion {
    padding: 1rem 1.5rem;
  }

  h3 { color: #8b9eb3; font-size: 1rem; margin-bottom: 0.75rem; }

  .vacio {
    text-align: center;
    color: #6b7d8f;
    padding: 2rem;
    background: #0f1419;
    border-radius: 6px;
    font-style: italic;
  }

  .lista {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background: #0f1419;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .item:hover { border-color: #5a8fc4; background: #1a2332; }
  .item.seleccionado { background: rgba(90, 143, 196, 0.15); border-color: #5a8fc4; }

  input[type="checkbox"] { width: 18px; height: 18px; cursor: pointer; margin-top: 0.25rem; }

  .info-req { flex: 1; }
  .info-req p { margin: 0.25rem 0; color: #a8c5e0; }
  .info-req p:first-child { font-weight: 500; }
  .detalle { color: #8b9eb3; font-size: 0.9rem; }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 2px solid #2d3e50;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
  }

  button:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-cancelar {
    background: #2d3e50;
    color: #a8c5e0;
  }

  .btn-cancelar:hover:not(:disabled) { background: #3a4f66; }

  .btn-guardar {
    background: #5a8fc4;
    color: white;
  }

  .btn-guardar:hover:not(:disabled) {
    background: #4a7fb4;
    transform: translateY(-1px);
  }
</style>
