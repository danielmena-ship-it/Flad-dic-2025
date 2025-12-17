<script>
  import { onMount } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { getRequerimientosParaInformePago, crearInformePago } from '$lib/utils/db-helpers.js';
  import { formatearNumero, calcularMontoTotalSeleccionados } from '$lib/utils/calculos.js';
  import { formatearFecha } from '$lib/utils/formatoFecha.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';
  import SelectorJardinModerno from './SelectorJardinModerno.svelte';

  let jardines = [];
  let jardinSeleccionado = '';
  let requerimientos = [];
  let seleccionados = new Set();
  let cargando = false;
  let mensaje = '';
  let logs = []; // ✅ Array de logs visibles
  
  function addLog(msg, tipo = 'info') {
    const timestamp = new Date().toLocaleTimeString();
    logs = [...logs, { timestamp, msg, tipo }];
    console.log(`[${timestamp}] ${msg}`);
  }
  
  let sortColumn = null;
  let sortDirection = 'asc';
  let ordenesTrabajoMap = {};

  onMount(async () => {
    // jardines ahora se cargan en SelectorJardinModerno
    const ots = await db.ordenesTrabajo.getAll();
    ordenesTrabajoMap = ots.reduce((acc, ot) => {
      acc[ot.id] = ot.codigo;
      return acc;
    }, {});
  });

  async function cargarRequerimientos() {
    if (!jardinSeleccionado) {
      requerimientos = [];
      return;
    }
    
    cargando = true;
    const reqs = await getRequerimientosParaInformePago({ 
      jardinCodigo: jardinSeleccionado 
    });
    requerimientos = await enriquecerRequerimientos(reqs);
    seleccionados.clear();
    cargando = false;
  }

  function toggleSeleccion(id) {
    if (seleccionados.has(id)) {
      seleccionados.delete(id);
    } else {
      seleccionados.add(id);
    }
    seleccionados = seleccionados;
  }

  function seleccionarTodos() {
    if (requerimientos.length === seleccionados.size) {
      seleccionados.clear();
    } else {
      requerimientos.forEach(r => seleccionados.add(r.id));
    }
    seleccionados = seleccionados;
  }

  async function generarInforme() {
    if (seleccionados.size === 0) {
      mensaje = '⚠️ Selecciona al menos un requerimiento';
      addLog('Intento crear informe sin seleccionar requerimientos', 'warning');
      setTimeout(() => mensaje = '', 3000);
      return;
    }

    try {
      cargando = true;
      mensaje = '';
      logs = []; // Limpiar logs anteriores
      
      addLog(`Iniciando creación de informe para jardín: ${jardinSeleccionado}`, 'info');
      addLog(`Requerimientos seleccionados: ${seleccionados.size}`, 'info');
      addLog(`IDs: ${Array.from(seleccionados).join(', ')}`, 'info');
      
      await crearInformePago(jardinSeleccionado, Array.from(seleccionados));
      
      addLog('✅ Informe creado exitosamente', 'success');
      mensaje = '✅ Informe de Pago creado exitosamente';
      await cargarRequerimientos();
      setTimeout(() => { mensaje = ''; logs = []; }, 5000);
    } catch (error) {
      addLog(`❌ ERROR: ${error?.message || error}`, 'error');
      addLog(`Stack: ${error?.stack || 'No disponible'}`, 'error');
      const errorMsg = error?.message || error?.toString() || 'Error desconocido';
      mensaje = '❌ Error al crear informe: ' + errorMsg;
    } finally {
      cargando = false;
    }
  }

  function sortBy(column) {
    if (sortColumn === column) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
    sortColumn = sortColumn;
  }

  $: sortedRequerimientos = !sortColumn ? requerimientos : [...requerimientos].sort((a, b) => {
    let valA, valB;
    let result = 0;
    
    switch(sortColumn) {
      case 'fecha':
        valA = new Date(a.fechaRecepcion).getTime();
        valB = new Date(b.fechaRecepcion).getTime();
        result = valA - valB;
        break;
      case 'zona':
        valA = (a.recinto || '').toLowerCase();
        valB = (b.recinto || '').toLowerCase();
        result = valA.localeCompare(valB);
        break;
      case 'partida':
        valA = (a.partidaNombre || '').toLowerCase();
        valB = (b.partidaNombre || '').toLowerCase();
        result = valA.localeCompare(valB);
        break;
      case 'cantidad':
        valA = parseFloat(a.cantidad) || 0;
        valB = parseFloat(b.cantidad) || 0;
        result = valA - valB;
        break;
    }
    
    return sortDirection === 'asc' ? result : -result;
  });

  $: montoTotal = calcularMontoTotalSeleccionados(requerimientos, seleccionados);
</script>

<div class="container">
  <div class="header">
    <h2>Ingresar Pago</h2>
    
    <div class="filtro">
      <label>Jardín:</label>
      <SelectorJardinModerno bind:value={jardinSeleccionado} onChange={cargarRequerimientos} />
    </div>
  </div>

  {#if mensaje}
    <div class="mensaje {mensaje.includes('✅') ? 'exito' : mensaje.includes('⚠️') ? 'warning' : 'error'}">{mensaje}</div>
  {/if}

  {#if cargando}
    <p class="cargando">Cargando...</p>
  {:else if !jardinSeleccionado}
    <p class="placeholder">Selecciona un jardín para ver recepciones pendientes de informe</p>
  {:else if jardinSeleccionado}
    <div class="acciones">
      <button class="generar" on:click={generarInforme} disabled={seleccionados.size === 0 || cargando}>
        {cargando ? 'Generando...' : `Generar Informe (${seleccionados.size} seleccionados)`}
      </button>
      <div class="total">Total Seleccionado: ${formatearNumero(montoTotal)}</div>
    </div>

    {#if cargando}
      <p>Cargando requerimientos...</p>
    {:else if requerimientos.length === 0}
      <p class="vacio">No hay requerimientos disponibles sin informe para este jardín</p>
    {:else}
      <div class="tabla-wrapper">
        <table>
          <thead>
            <tr>
              <th><input type="checkbox" on:change={seleccionarTodos} checked={requerimientos.length === seleccionados.size && requerimientos.length > 0} /></th>
              <th class="sortable" on:click={() => sortBy('fecha')}>
                Fecha Recepción
                {#if sortColumn === 'fecha'}
                  <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
                {/if}
              </th>
              <th class="sortable" on:click={() => sortBy('zona')}>
                Zona
                {#if sortColumn === 'zona'}
                  <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
                {/if}
              </th>
              <th class="sortable" on:click={() => sortBy('partida')}>
                Item - Partida
                {#if sortColumn === 'partida'}
                  <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
                {/if}
              </th>
              <th>Descripción</th>
              <th>Observación</th>
              <th class="sortable" on:click={() => sortBy('cantidad')}>
                Cantidad
                {#if sortColumn === 'cantidad'}
                  <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
                {/if}
              </th>
            </tr>
          </thead>
          <tbody>
            {#each sortedRequerimientos as req}
              <tr class:seleccionado={seleccionados.has(req.id)}>
                <td>
                  <input 
                    type="checkbox" 
                    checked={seleccionados.has(req.id)} 
                    on:change={() => toggleSeleccion(req.id)}
                  />
                </td>
                <td>{formatearFecha(req.fechaRecepcion)}</td>
                <td>{req.recinto}</td>
                <td>{req.partidaItem} - {req.partidaNombre}</td>
                <td class="descripcion" title={req.descripcion || '-'}>{req.descripcion || '-'}</td>
                <td class="observacion" title={req.observaciones || '-'}>{req.observaciones || '-'}</td>
                <td>{formatearNumero(req.cantidad)} {req.partidaUnidad}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

<style>
  .container { 
    padding: 0 2rem 2rem 2rem;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }
  
  h2 { 
    color: #a8c5e0; 
    margin: 0;
  }

  .filtro {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .filtro label {
    color: #8b9eb3;
    font-weight: 500;
    white-space: nowrap;
  }

  select {
    padding: 0.75rem;
    background: #0f1419;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    color: #a8c5e0;
    font-family: 'Inter', sans-serif;
    min-width: 300px;
  }

  .cargando {
    text-align: center;
    color: #8b9eb3;
    padding: 2rem;
  }

  .placeholder {
    text-align: center;
    color: #6b7d8f;
    padding: 4rem 2rem;
    background: #1a2332;
    border: 2px dashed #2d3e50;
    border-radius: 8px;
    font-size: 1.05rem;
  }

  .mensaje {
    padding: 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
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

  .mensaje.warning {
    background: rgba(255, 193, 7, 0.2);
    color: #ffd54f;
    border: 1px solid #ffc107;
  }

  .acciones {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
    align-items: center;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
    background: #2d3e50;
    color: #a8c5e0;
  }

  button:hover:not(:disabled) {
    background: #3a4f66;
    transform: translateY(-1px);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.generar {
    background: #5a8fc4;
    color: white;
  }

  button.generar:hover:not(:disabled) {
    background: #4a7fb4;
  }

  .total {
    margin-left: auto;
    color: #a8c5e0;
    font-weight: 600;
    font-size: 1.1rem;
  }

  .vacio {
    text-align: center;
    color: #8b9eb3;
    padding: 2rem;
  }

  .tabla-wrapper {
    overflow-x: auto;
    background: #1a2332;
    border-radius: 8px;
    padding: 1rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead th {
    background: #0f1419;
    color: #8b9eb3;
    padding: 1rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #2d3e50;
    border-left: none;
    border-right: none;
    position: relative;
  }

  thead th.sortable {
    cursor: pointer;
    user-select: none;
  }

  thead th.sortable:hover {
    background: #1a2332;
    color: #a8c5e0;
  }

  .sort-icon {
    margin-left: 0.5rem;
    font-size: 0.8rem;
    color: #5a8fc4;
  }

  tbody td {
    padding: 0.75rem 1rem;
    color: #a8c5e0;
    border-bottom: 1px solid #2d3e50;
    border-left: none;
    border-right: none;
  }

  tbody td.descripcion {
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  tbody td.observacion {
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  tbody tr:hover {
    background: #0f1419;
  }

  tbody tr.seleccionado {
    background: rgba(90, 143, 196, 0.2);
  }

  input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  /* Panel de Logs */
  .panel-logs {
    margin: 1rem 0;
    padding: 1rem;
    background: #0f1419;
    border: 1px solid #2d3e50;
    border-radius: 8px;
    max-height: 300px;
    overflow-y: auto;
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
  }

  .logs-header {
    font-weight: 600;
    color: #7aafde;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #2d3e50;
  }

  .log-item {
    padding: 0.5rem;
    margin: 0.25rem 0;
    border-radius: 4px;
    display: flex;
    gap: 0.75rem;
  }

  .log-item.info {
    background: rgba(90, 143, 196, 0.1);
    color: #7aafde;
  }

  .log-item.success {
    background: rgba(76, 175, 80, 0.1);
    color: #81c784;
  }

  .log-item.warning {
    background: rgba(255, 152, 0, 0.1);
    color: #ffb74d;
  }

  .log-item.error {
    background: rgba(244, 67, 54, 0.1);
    color: #e57373;
  }

  .log-time {
    color: #546e7a;
    min-width: 70px;
  }

  .log-msg {
    flex: 1;
    word-break: break-word;
  }
</style>
