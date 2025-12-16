<script>
  import { createEventDispatcher, onMount } from 'svelte';
  import { getRequerimientosSinOT, editarOrdenTrabajo, getRequerimientosPorOT } from '$lib/utils/db-helpers.js';
  import { formatearNumero, calcularMontoTotalSeleccionados } from '$lib/utils/calculos.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';

  export let ot;

  const dispatch = createEventDispatcher();

  let requerimientosDisponibles = [];
  let requerimientosActuales = [];
  let seleccionados = new Set();
  let cargando = true;
  let mensaje = '';

  onMount(async () => {
    // Cargar requerimientos actuales de la OT
    const reqsActuales = await getRequerimientosPorOT(ot.id);
    requerimientosActuales = await enriquecerRequerimientos(reqsActuales);
    
    // Marcar como seleccionados
    requerimientosActuales.forEach(r => seleccionados.add(r.id));
    seleccionados = seleccionados; // Forzar reactividad
    
    // Cargar requerimientos disponibles (sin OT) del mismo jardín
    const reqsDisponibles = await getRequerimientosSinOT({ 
      jardinCodigo: ot.jardinCodigo 
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
      await editarOrdenTrabajo(ot.id, Array.from(seleccionados));
      dispatch('cerrar');
    } catch (error) {
      mensaje = '❌ Error al actualizar OT: ' + (error.message || error.toString() || JSON.stringify(error));
    } finally {
      cargando = false;
    }
  }

  function cerrar() {
    dispatch('cerrar');
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
      <h2>Editar {ot.codigo}</h2>
      <button class="cerrar" on:click={cerrar}>✕</button>
    </div>

    {#if mensaje}
      <div class="mensaje {mensaje.includes('✅') ? 'exito' : 'error'}">{mensaje}</div>
    {/if}

    <div class="info">
      <p><strong>Jardín:</strong> {ot.jardinCodigo}</p>
      <p><strong>Requerimientos Seleccionados:</strong> {seleccionados.size}</p>
      <p><strong>Monto Total:</strong> ${formatearNumero(montoTotal)}</p>
    </div>

    {#if cargando && todosLosRequerimientos.length === 0}
      <p>Cargando...</p>
    {:else}
      <div class="seccion">
        <h3>Requerimientos Actuales de la OT</h3>
        {#if requerimientosActuales.length === 0}
          <p class="vacio">No hay requerimientos actuales</p>
        {:else}
          <div class="lista">
            {#each requerimientosActuales as req}
              <label class="item">
                <input 
                  type="checkbox" 
                  checked={seleccionados.has(req.id)} 
                  on:change={() => toggleSeleccion(req.id)}
                />
                <div class="detalles">
                  <span class="principal">{req.recinto} - {req.partidaItem} - {req.partidaNombre}</span>
                  <span class="secundario">
                    {formatearNumero(req.cantidad)} {req.partidaUnidad} • ${formatearNumero(req.precioTotal)}
                  </span>
                </div>
              </label>
            {/each}
          </div>
        {/if}
      </div>

      <div class="seccion">
        <h3>Requerimientos Disponibles (sin OT asignada)</h3>
        {#if requerimientosDisponibles.length === 0}
          <p class="vacio">No hay requerimientos disponibles para este jardín</p>
        {:else}
          <div class="lista">
            {#each requerimientosDisponibles as req}
              <label class="item">
                <input 
                  type="checkbox" 
                  checked={seleccionados.has(req.id)} 
                  on:change={() => toggleSeleccion(req.id)}
                />
                <div class="detalles">
                  <span class="principal">{req.recinto} - {req.partidaItem} - {req.partidaNombre}</span>
                  <span class="secundario">
                    {formatearNumero(req.cantidad)} {req.partidaUnidad} • ${formatearNumero(req.precioTotal)}
                  </span>
                </div>
              </label>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <div class="footer">
      <button class="cancelar" on:click={cerrar}>Cancelar</button>
      <button class="guardar" on:click={guardar} disabled={cargando || seleccionados.size === 0}>
        {cargando ? 'Guardando...' : 'Guardar Cambios'}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal {
    background: #1a2332;
    border-radius: 12px;
    max-width: 800px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 2px solid #2d3e50;
  }

  .header h2 {
    color: #a8c5e0;
    margin: 0;
  }

  .cerrar {
    background: transparent;
    border: none;
    color: #8b9eb3;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.5rem;
  }

  .cerrar:hover {
    color: #e57373;
  }

  .mensaje {
    margin: 1rem 1.5rem;
    padding: 1rem;
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
    padding: 1rem 1.5rem;
    background: #0f1419;
    display: flex;
    gap: 2rem;
    color: #a8c5e0;
  }

  .info p {
    margin: 0;
  }

  .info strong {
    color: #8b9eb3;
  }

  .seccion {
    padding: 1.5rem;
    border-bottom: 1px solid #2d3e50;
  }

  .seccion h3 {
    color: #a8c5e0;
    margin: 0 0 1rem 0;
    font-size: 1rem;
  }

  .vacio {
    color: #8b9eb3;
    font-style: italic;
    padding: 1rem;
    text-align: center;
    background: #0f1419;
    border-radius: 6px;
  }

  .lista {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: #0f1419;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .item:hover {
    background: #2d3e50;
  }

  .item input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .detalles {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    flex: 1;
  }

  .principal {
    color: #a8c5e0;
    font-weight: 500;
  }

  .secundario {
    color: #8b9eb3;
    font-size: 0.9rem;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
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

  .cancelar {
    background: #2d3e50;
    color: #a8c5e0;
  }

  .cancelar:hover {
    background: #3a4f66;
  }

  .guardar {
    background: #5a8fc4;
    color: white;
  }

  .guardar:hover:not(:disabled) {
    background: #4a7fb4;
  }

  .guardar:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
