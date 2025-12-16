<script>
  import { onMount } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { getRequerimientosParaRecepcion, guardarFechasRecepcion } from '$lib/utils/db-helpers.js';
  import { formatearNumero } from '$lib/utils/calculos.js';
  import { formatearFecha } from '$lib/utils/formatoFecha.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';
  import { log, error } from '$lib/utils/logger.js';
  import SelectorFechaRecepcion from './SelectorFechaRecepcion.svelte';
  import SelectorJardinModerno from './SelectorJardinModerno.svelte';

  let jardines = [];
  let jardinSeleccionado = '';
  let requerimientos = [];
  let cargando = false;
  let mensaje = '';
  let seleccionados = new Set();
  let fechaRecepcionCompartida = ''; // Fecha única para todos los seleccionados
  let ordenesTrabajoMap = {}; // { [ot_id]: codigo_ot }
  let fechaMaxima = new Date().toISOString().split('T')[0];

  // Calcular fecha mínima dinámica: la fecha de inicio más reciente de los seleccionados
  $: fechaMinimaSeleccion = (() => {
    if (seleccionados.size === 0) return new Date().toISOString().split('T')[0];
    
    const fechasInicio = Array.from(seleccionados)
      .map(id => requerimientos.find(r => r.id === id)?.fechaInicio)
      .filter(f => f && f.length === 10) // Validar formato
      .sort((a, b) => new Date(b) - new Date(a));
    
    return fechasInicio[0] || new Date().toISOString().split('T')[0];
  })();

  // Resetear fecha cuando cambia la selección o cuando la fecha es inválida
  $: {
    if (fechaRecepcionCompartida && fechaMinimaSeleccion) {
      const fechaActual = new Date(fechaRecepcionCompartida);
      const fechaMin = new Date(fechaMinimaSeleccion);
      const fechaMax = new Date(fechaMaxima);
      
      // Si la fecha está fuera del rango válido, resetearla
      if (fechaActual < fechaMin || fechaActual > fechaMax) {
        fechaRecepcionCompartida = '';
      }
    }
  }

  onMount(async () => {
    // jardines ahora se cargan en SelectorJardinModerno
    // Cargar todas las OT para mapear códigos
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
    try {
      const reqs = await getRequerimientosParaRecepcion(jardinSeleccionado);
      requerimientos = await enriquecerRequerimientos(reqs);
      seleccionados.clear();
      fechaRecepcionCompartida = '';
    } catch (error) {
      console.error('Error cargando requerimientos:', error);
      mensaje = '❌ Error al cargar requerimientos';
    } finally {
      cargando = false;
    }
  }

  function toggleSeleccion(id) {
    if (seleccionados.has(id)) {
      seleccionados.delete(id);
    } else {
      seleccionados.add(id);
    }
    seleccionados = seleccionados;
  }

  function toggleTodos() {
    if (seleccionados.size === requerimientos.length) {
      seleccionados.clear();
      seleccionados = seleccionados;
    } else {
      seleccionados = new Set(requerimientos.map(r => r.id));
    }
  }

  async function guardar() {
    // Validar que se haya seleccionado al menos un requerimiento
    if (seleccionados.size === 0) {
      mensaje = '⚠️ Selecciona al menos un requerimiento';
      setTimeout(() => mensaje = '', 3000);
      return;
    }

    // Validar que se haya ingresado una fecha
    if (!fechaRecepcionCompartida) {
      mensaje = '⚠️ Ingresa la fecha de recepción';
      setTimeout(() => mensaje = '', 3000);
      return;
    }

    const seleccionadosConFecha = Array.from(seleccionados)
      .map(id => {
        const req = requerimientos.find(r => r.id === id);
        return {
          id,
          fechaRecepcion: fechaRecepcionCompartida,
          fechaInicio: req.fechaInicio
        };
      });

    // Validación
    const fechaHoy = new Date().toISOString().split('T')[0];
    const errores = [];
    
    for (const item of seleccionadosConFecha) {
      if (!item.fechaInicio || item.fechaInicio.length !== 10) {
        errores.push(`Requerimiento ${item.id}: fecha inicio inválida`);
        continue;
      }
      
      const fechaInicio = item.fechaInicio;
      const fechaRecepcion = item.fechaRecepcion;
      
      if (fechaRecepcion < fechaInicio) {
        errores.push(`Requerimiento ${item.id}: fecha recepción anterior a fecha inicio`);
      }
      if (fechaRecepcion > fechaHoy) {
        errores.push(`Requerimiento ${item.id}: fecha recepción no puede ser futura`);
      }
    }
    
    if (errores.length > 0) {
      mensaje = '❌ Fechas inválidas detectadas:\n' + errores.join('\n');
      setTimeout(() => mensaje = '', 5000);
      return;
    }

    try {
      cargando = true;
      await guardarFechasRecepcion(seleccionadosConFecha);
      mensaje = `✅ ${seleccionadosConFecha.length} recepción(es) guardada(s) → Ve a "Lista Recepción" para verlas`;
      await cargarRequerimientos();
      setTimeout(() => mensaje = '', 5000);
    } catch (error) {
      mensaje = '❌ Error al guardar: ' + (error.message || error || 'Error desconocido');
    } finally {
      cargando = false;
    }
  }

  function extraerNumeroZona(zona) {
    const match = zona.match(/^(\d+)/);
    return match ? match[1] : zona;
  }

  function truncarPartida(nombre) {
    if (!nombre) return '-';
    const palabras = nombre.split(' ');
    return palabras.slice(0, 5).join(' ') + (palabras.length > 5 ? '...' : '');
  }

  function truncarDescripcion(texto) {
    if (!texto) return '-';
    const palabras = texto.split(' ');
    return palabras.slice(0, 5).join(' ') + (palabras.length > 5 ? '...' : '');
  }

</script>

<div class="container">
  <div class="header">
    <h2>Recepción de Trabajos</h2>
    
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
    <p class="placeholder">Selecciona un jardín para ver requerimientos pendientes de recepción</p>
  {:else if requerimientos.length === 0}
    <p class="vacio">No hay requerimientos pendientes de recepción para este jardín</p>
  {:else}
    <div class="acciones">
      <div class="selector-fecha-compartido">
        <label for="fecha-compartida">F. recepción:</label>
        <SelectorFechaRecepcion 
          bind:value={fechaRecepcionCompartida}
          fechaMinima={fechaMinimaSeleccion}
          fechaMaxima={fechaMaxima}
          disabled={seleccionados.size === 0}
        />
        {#if seleccionados.size > 0}
          <span class="rango-info">
            (Rango: {formatearFecha(fechaMinimaSeleccion)} - Hoy)
          </span>
        {/if}
      </div>
      <button class="btn-guardar" on:click={guardar} disabled={seleccionados.size === 0}>
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
          <polyline points="17 21 17 13 7 13 7 21"></polyline>
          <polyline points="7 3 7 8 15 8"></polyline>
        </svg>
        Guardar ({seleccionados.size})
      </button>
    </div>

    <div class="tabla-contenedor">
      <table>
        <thead>
          <tr>
            <th>
              <input 
                type="checkbox" 
                checked={seleccionados.size === requerimientos.length}
                on:change={toggleTodos}
              />
            </th>
            <th>Código OT</th>
            <th>Zona</th>
            <th>Item - Partida</th>
            <th>Descripción</th>
            <th>Cant.</th>
            <th>F. Inicio</th>
            <th>F. Limite</th>
            <th>Plazo</th>
          </tr>
        </thead>
        <tbody>
          {#each requerimientos as req}
            <tr class:seleccionado={seleccionados.has(req.id)}>
              <td>
                <input 
                  type="checkbox" 
                  checked={seleccionados.has(req.id)}
                  on:change={() => toggleSeleccion(req.id)}
                />
              </td>
              <td class="ot-codigo">{ordenesTrabajoMap[req.otId] || 'N/A'}</td>
              <td>{extraerNumeroZona(req.recinto)}</td>
              <td title="{req.partidaItem} - {req.partidaNombre}">{req.partidaItem} - {truncarPartida(req.partidaNombre)}</td>
              <td class="descripcion" title={req.descripcion || '-'}>{truncarDescripcion(req.descripcion)}</td>
              <td>{formatearNumero(req.cantidad)} {req.partidaUnidad}</td>
              <td>{formatearFecha(req.fechaInicio)}</td>
              <td>{req.fechaLimite ? formatearFecha(req.fechaLimite) : '-'}</td>
              <td>{req.plazoDias} días</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
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
    flex-wrap: wrap;
    gap: 1rem;
  }

  h2 {
    color: #a8c5e0;
    font-size: 1.5rem;
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
  }

  .filtro select {
    padding: 0.75rem;
    background: #0f1419;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    color: #a8c5e0;
    font-family: 'Inter', sans-serif;
    min-width: 300px;
    cursor: pointer;
  }

  .filtro select:focus {
    outline: none;
    border-color: #5a8fc4;
  }

  .mensaje {
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
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

  .mensaje.warning {
    background: rgba(255, 193, 7, 0.2);
    color: #ffd54f;
    border: 1px solid #ffc107;
  }

  .cargando {
    text-align: center;
    color: #8b9eb3;
    padding: 2rem;
  }

  .placeholder, .vacio {
    text-align: center;
    color: #6b7d8f;
    font-size: 1.05rem;
    padding: 4rem 2rem;
    background: #1a2332;
    border-radius: 8px;
    border: 2px dashed #2d3e50;
  }

  .acciones {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    gap: 1.5rem;
  }

  .selector-fecha-compartido {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: rgba(45, 62, 80, 0.3);
    border-radius: 6px;
    border: 1px solid #2d3e50;
  }

  .selector-fecha-compartido label {
    color: #8b9eb3;
    font-weight: 500;
    white-space: nowrap;
  }

  .rango-info {
    color: #6b7d8f;
    font-size: 0.85rem;
    font-style: italic;
    white-space: nowrap;
  }

  .btn-guardar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: #5a8fc4;
    color: white;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
  }

  .btn-guardar:hover:not(:disabled) {
    background: #4a7fb4;
    transform: translateY(-1px);
  }

  .btn-guardar:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-guardar svg {
    width: 16px;
    height: 16px;
  }

  .tabla-contenedor {
    border-radius: 8px;
    border: 1px solid #2d3e50;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    background: #0f1419;
  }

  thead th {
    background: #1a2332;
    color: #8b9eb3;
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #2d3e50;
    white-space: nowrap;
  }

  tbody td {
    padding: 0.75rem;
    color: #a8c5e0;
    border-bottom: 1px solid #2d3e50;
    vertical-align: middle;
    overflow: visible;
  }

  tbody tr {
    transition: background 0.2s;
  }

  tbody tr:hover {
    background: rgba(90, 143, 196, 0.05);
  }

  tbody tr.seleccionado {
    background: rgba(90, 143, 196, 0.1);
  }

  input[type="checkbox"] {
    cursor: pointer;
    width: 16px;
    height: 16px;
  }

  .ot-codigo {
    color: #81c784;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .descripcion {
    max-width: 250px;
    word-wrap: break-word;
    font-size: 0.9rem;
  }
</style>
