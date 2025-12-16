<script>
  import { onMount, onDestroy } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { getOrdenesTrabajo, getOrdenTrabajoDetalle, eliminarOrdenTrabajo } from '$lib/utils/db-helpers.js';
  import { formatearFecha } from '$lib/utils/formatoFecha.js';
  import { jardines, cargarJardines } from '$lib/stores/catalogos.js';
  import { enriquecerRequerimientos, enriquecerOrdenesTrabajo } from '$lib/utils/enriquecimiento.js';
  import { log } from '$lib/utils/logger.js';
  import ModalEditarOT from './ModalEditarOT.svelte';
  import ModalEditarRequerimiento from './ModalEditarRequerimiento.svelte';
  import ModalVistaImpresion from './ModalVistaImpresion.svelte';

  let ordenes = [];
  let cargando = true;
  let expandido = null;
  let detalles = {};
  let modalAbierto = false;
  let modalReqAbierto = false;
  let modalImpresionAbierto = false;
  let otSeleccionada = null;
  let reqSeleccionado = null;
  let datosImpresion = null;
  let sortColumn = null;
  let sortDirection = 'asc';
  let detalleSortColumn = {};
  let detalleSortDirection = {};
  let itemAEliminar = null;
  let mensajeError = ''; // ✅ FIX WINDOWS: Mensaje in-app (no alert)
  
  // Filtros
  let filtroJardin = '';
  let filtroFecha = '';
  let sortAlerta = null; // null, 'asc', 'desc' - ordenamiento por atraso 
  
  // Selector de fecha con dropdown
  let añoFiltro = '';
  let mesFiltro = '';
  let diaFiltro = '';
  let dropdownAbierto = null;
  
  const fechaHoy = new Date();
  const añoActual = fechaHoy.getFullYear();
  const mesActual = fechaHoy.getMonth() + 1;
  const diaActual = fechaHoy.getDate();
  
  const años = [2025, 2026, 2027, 2028, 2029, 2030];
  const meses = [
    { num: '01', nombre: 'Ene' }, { num: '02', nombre: 'Feb' }, { num: '03', nombre: 'Mar' },
    { num: '04', nombre: 'Abr' }, { num: '05', nombre: 'May' }, { num: '06', nombre: 'Jun' },
    { num: '07', nombre: 'Jul' }, { num: '08', nombre: 'Ago' }, { num: '09', nombre: 'Sep' },
    { num: '10', nombre: 'Oct' }, { num: '11', nombre: 'Nov' }, { num: '12', nombre: 'Dic' }
  ];
  
  $: diasEnMes = mesFiltro && añoFiltro ? new Date(parseInt(añoFiltro), parseInt(mesFiltro), 0).getDate() : 31;
  $: dias = Array.from({ length: diasEnMes }, (_, i) => i + 1);
  
  $: if (añoFiltro && mesFiltro && diaFiltro) {
    filtroFecha = `${añoFiltro}-${mesFiltro}-${String(diaFiltro).padStart(2, '0')}`;
  } else {
    filtroFecha = '';
  }

  let ordenesFiltradas = [];
  
  $: filtrarOrdenesReactivo(ordenes, filtroJardin, filtroFecha);
  
  async function filtrarOrdenesReactivo(ords, jardin, fecha) {
    let resultado = await filtrarOrdenes(ords, jardin, fecha);
    ordenesFiltradas = resultado;
  }

  function toggleSortAlerta() {
    if (sortAlerta === null) {
      sortAlerta = 'asc';
    } else if (sortAlerta === 'asc') {
      sortAlerta = 'desc';
    } else {
      sortAlerta = null;
    }
  }

  function esFechaAtrasada(fechaLimite, fechaRecepcion) {
    if (fechaRecepcion) return false; // Ya recibido, no está atrasado
    if (!fechaLimite) return false;
    const hoy = new Date();
    hoy.setHours(0, 0, 0, 0);
    const limite = new Date(fechaLimite);
    limite.setHours(0, 0, 0, 0);
    return limite < hoy;
  }

  onMount(async () => {
    await cargarJardines();
    await cargarOrdenes();
  });

  function getNombreJardin(codigo) {
    const jardin = $jardines.find(j => j.codigo === codigo);
    return jardin ? jardin.nombre : codigo;
  }

  async function cargarOrdenes() {
    try {
      cargando = true;
      const ordenesRaw = await getOrdenesTrabajo();
      ordenes = await enriquecerOrdenesTrabajo(ordenesRaw);
      
      // Calcular alertas para cada orden
      const hoy = new Date();
      hoy.setHours(0, 0, 0, 0);
      
      for (const orden of ordenes) {
        try {
          const detalle = await getOrdenTrabajoDetalle(orden.id);
          orden.tieneAtrasados = detalle.requerimientos.some(req => {
            if (req.fechaRecepcion) return false; // Ya recibido, no cuenta
            if (!req.fechaLimite) return false;
            const fechaLimite = new Date(req.fechaLimite);
            fechaLimite.setHours(0, 0, 0, 0);
            return fechaLimite < hoy;
          });
        } catch (err) {
          console.error(`Error calculando alertas para OT ${orden.id}:`, err);
          orden.tieneAtrasados = false;
        }
      }
    } catch (error) {
      console.error('Error cargando órdenes:', error);
      mensajeError = 'Error cargando órdenes: ' + error.message;
    } finally {
      cargando = false;
    }
  }
  
  async function filtrarOrdenes(ords, jardin, fecha) {
    let resultado = [...ords];
    
    // Filtrar por jardín si está seleccionado
    if (jardin) {
      resultado = resultado.filter(ord => ord.jardinCodigo === jardin);
    }
    
    // Filtrar por fecha si está seleccionada
    if (fecha) {
      const fechaFiltro = new Date(fecha);
      resultado = resultado.filter(ord => {
        if (!ord.fechaCreacion) return false;
        const fechaOrd = new Date(ord.fechaCreacion);
        return fechaOrd >= fechaFiltro;
      });
    }
    
    return resultado;
  }
  
  function toggleDropdown(tipo) {
    dropdownAbierto = dropdownAbierto === tipo ? null : tipo;
  }
  
  function seleccionarJardin(codigo) {
    filtroJardin = codigo;
    dropdownAbierto = null;
  }

  function seleccionarAño(a) {
    añoFiltro = String(a);
    dropdownAbierto = null;
  }

  function seleccionarMes(m) {
    mesFiltro = m;
    dropdownAbierto = null;
  }

  function seleccionarDia(d) {
    diaFiltro = String(d);
    dropdownAbierto = null;
  }

  function getMesNombre(num) {
    return meses.find(m => m.num === num)?.nombre || '';
  }
  
  function getJardinNombre(codigo) {
    if (!codigo) return 'Todos los jardines';
    const jardin = $jardines.find(j => j.codigo === codigo);
    return jardin ? `[${jardin.codigo}] ${jardin.nombre}` : 'Todos los jardines';
  }
  
  function limpiarFiltroFecha() {
    añoFiltro = '';
    mesFiltro = '';
    diaFiltro = '';
    filtroFecha = '';
  }

  async function toggleDetalle(ot_id) {
    if (expandido === ot_id) {
      expandido = null;
    } else {
      expandido = ot_id;
      if (!detalles[ot_id]) {
        const detalle = await getOrdenTrabajoDetalle(ot_id);
        detalle.requerimientos = await enriquecerRequerimientos(detalle.requerimientos);
        detalles[ot_id] = detalle;
        detalles = detalles; // trigger reactivity
      }
    }
  }

  function abrirModalEdicion(ot) {
    otSeleccionada = ot;
    modalAbierto = true;
  }

  async function cerrarModal() {
    modalAbierto = false;
    otSeleccionada = null;
    await cargarOrdenes();
    detalles = {}; // limpiar cache de detalles
  }

  async function abrirModalImpresion(ot) {
    const detalle = await getOrdenTrabajoDetalle(ot.id);
    detalle.requerimientos = await enriquecerRequerimientos(detalle.requerimientos);
    datosImpresion = {
      ot: ot,
      requerimientos: detalle.requerimientos
    };
    modalImpresionAbierto = true;
  }

  function cerrarModalImpresion() {
    modalImpresionAbierto = false;
    datosImpresion = null;
  }

  function abrirModalEdicionRequerimiento(req) {
    reqSeleccionado = req;
    modalReqAbierto = true;
  }

  async function cerrarModalRequerimiento() {
    modalReqAbierto = false;
    reqSeleccionado = null;
    // Recargar detalles de la OT actual
    if (expandido) {
      const detalle = await getOrdenTrabajoDetalle(expandido);
      detalle.requerimientos = await enriquecerRequerimientos(detalle.requerimientos);
      detalles[expandido] = detalle;
      detalles = detalles;
    }
  }

  async function eliminar(ot_id, codigo) {
    try {
      await eliminarOrdenTrabajo(ot_id);
      await cargarOrdenes();
      if (expandido === ot_id) {
        expandido = null;
      }
      delete detalles[ot_id];
      detalles = detalles;
      cancelarEliminar();
    } catch (error) {
      mensajeError = '❌ Error al eliminar OT: ' + error.message;
      setTimeout(() => mensajeError = '', 4000);
    }
  }

  function confirmarEliminar(ot_id) {
    itemAEliminar = ot_id;
    document.addEventListener('keydown', handleKeyPress);
  }

  function handleKeyPress(e) {
    if (e.key === 'Enter' && itemAEliminar) {
      const orden = ordenes.find(o => o.id === itemAEliminar);
      if (orden) {
        eliminar(orden.id, orden.codigo);
      }
    }
  }

  function cancelarEliminar() {
    itemAEliminar = null;
    document.removeEventListener('keydown', handleKeyPress);
  }

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeyPress);
  });

  function sortBy(column) {
    if (sortColumn === column) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
    // Force trigger
    sortColumn = sortColumn;
  }

  function sortDetalleBy(ot_id, column) {
    if (detalleSortColumn[ot_id] === column) {
      detalleSortDirection[ot_id] = detalleSortDirection[ot_id] === 'asc' ? 'desc' : 'asc';
    } else {
      detalleSortColumn[ot_id] = column;
      detalleSortDirection[ot_id] = 'asc';
    }
    
    // Force reactivity
    detalleSortColumn = { ...detalleSortColumn };
    detalleSortDirection = { ...detalleSortDirection };
  }

  $: detallesSorted = (() => {
    // Forzar reactividad incluyendo las dependencias de sort
    const _ = detalleSortColumn;
    const __ = detalleSortDirection;
    
    return Object.keys(detalles).reduce((acc, ot_id) => {
      acc[ot_id] = getSortedRequerimientos(Number(ot_id), detalles[ot_id].requerimientos);
      return acc;
    }, {});
  })();

  function truncarTexto(texto, maxPalabras = 5) {
    if (!texto) return '-';
    const palabras = texto.split(' ');
    return palabras.slice(0, maxPalabras).join(' ') + (palabras.length > maxPalabras ? '...' : '');
  }

  function getSortedRequerimientos(ot_id, requerimientos) {
    const column = detalleSortColumn[ot_id];
    const direction = detalleSortDirection[ot_id] || 'asc';
    
    if (!column) return requerimientos;
    
    return [...requerimientos].sort((a, b) => {
      let valA, valB;
      let result = 0;
      
      switch(column) {
        case 'fecha':
          valA = new Date(a.fechaInicio).getTime();
          valB = new Date(b.fechaInicio).getTime();
          result = valA - valB;
          break;
        case 'recinto':
          valA = (a.recinto || '').toLowerCase();
          valB = (b.recinto || '').toLowerCase();
          result = valA.localeCompare(valB);
          break;
        case 'partida':
          valA = (a.partidaNombre || '').toLowerCase();
          valB = (b.partidaNombre || '').toLowerCase();
          result = valA.localeCompare(valB);
          break;
        case 'descripcion':
          valA = (a.descripcion || '').toLowerCase();
          valB = (b.descripcion || '').toLowerCase();
          result = valA.localeCompare(valB);
          break;
        case 'observacion':
          valA = (a.observacion || '').toLowerCase();
          valB = (b.observacion || '').toLowerCase();
          result = valA.localeCompare(valB);
          break;
        case 'plazo':
          valA = parseFloat(a.plazoTotal) || 0;
          valB = parseFloat(b.plazoTotal) || 0;
          result = valA - valB;
          break;
        case 'fecha_limite':
          valA = a.fechaLimite ? new Date(a.fechaLimite).getTime() : 0;
          valB = b.fechaLimite ? new Date(b.fechaLimite).getTime() : 0;
          result = valA - valB;
          break;
        case 'cantidad':
          valA = parseFloat(a.cantidad) || 0;
          valB = parseFloat(b.cantidad) || 0;
          result = valA - valB;
          break;
        case 'precio':
          valA = parseFloat(a.precioTotal) || 0;
          valB = parseFloat(b.precioTotal) || 0;
          result = valA - valB;
          break;
      }
      
      return direction === 'asc' ? result : -result;
    });
  }

  $: sortedOrdenes = (() => {
    let resultado = [...ordenesFiltradas];
    
    // Aplicar ordenamiento regular si está activo
    if (sortColumn) {
      resultado.sort((a, b) => {
        let valA, valB;
        let sortResult = 0;
        
        switch(sortColumn) {
          case 'codigo':
            valA = (a.codigo || '').toLowerCase();
            valB = (b.codigo || '').toLowerCase();
            sortResult = valA.localeCompare(valB);
            break;
          case 'jardin':
            valA = (a.jardinCodigo || '').toLowerCase();
            valB = (b.jardinCodigo || '').toLowerCase();
            sortResult = valA.localeCompare(valB);
            break;
          case 'fecha':
            valA = new Date(a.fechaCreacion).getTime();
            valB = new Date(b.fechaCreacion).getTime();
            sortResult = valA - valB;
            break;
        }
        
        return sortDirection === 'asc' ? sortResult : -sortResult;
      });
    }
    
    // Aplicar ordenamiento por alerta si está activo
    if (sortAlerta) {
      resultado.sort((a, b) => {
        const aAtrasado = a.tieneAtrasados ? 1 : 0;
        const bAtrasado = b.tieneAtrasados ? 1 : 0;
        const sortResult = aAtrasado - bAtrasado;
        return sortAlerta === 'asc' ? sortResult : -sortResult;
      });
    }
    
    return resultado;
  })();
</script>

<div class="container">
  <h2>Lista de Órdenes de Trabajo</h2>

  <!-- Filtros -->
  <div class="filtros">
    <div class="form-group">
      <label>Filtrar por Jardín</label>
      <div class="dropdown-wrapper">
        <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('jardin')}>
          {getJardinNombre(filtroJardin)}
          <span class="arrow">{dropdownAbierto === 'jardin' ? '▲' : '▼'}</span>
        </button>
        {#if dropdownAbierto === 'jardin'}
          <div class="dropdown-panel">
            <div class="grid-1col">
              <button type="button" class="grid-btn" class:selected={!filtroJardin} on:click={() => seleccionarJardin('')}>
                Todos los jardines
              </button>
              {#each $jardines as jardin}
                <button type="button" class="grid-btn" class:selected={filtroJardin === jardin.codigo} on:click={() => seleccionarJardin(jardin.codigo)}>
                  [{jardin.codigo}] - {jardin.nombre}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="form-group" role="group" aria-label="Filtrar por fecha de creación">
      <label>Filtrar por F. Creación (desde)</label>
      <div class="fecha-filtro-container">
        <div class="fecha-grid">
          <!-- Año -->
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('año')}>
              {añoFiltro || 'Año'}
              <span class="arrow">{dropdownAbierto === 'año' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'año'}
              <div class="dropdown-panel">
                <div class="grid-1col">
                  {#each años as a}
                    <button type="button" class="grid-btn" class:selected={añoFiltro === String(a)} on:click={() => seleccionarAño(a)}>
                      {a}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Mes -->
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('mes')}>
              {getMesNombre(mesFiltro) || 'Mes'}
              <span class="arrow">{dropdownAbierto === 'mes' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'mes'}
              <div class="dropdown-panel">
                <div class="grid-3col">
                  {#each meses as m}
                    <button type="button" class="grid-btn" class:selected={mesFiltro === m.num} on:click={() => seleccionarMes(m.num)}>
                      {m.nombre}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Día -->
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('dia')}>
              {diaFiltro || 'Día'}
              <span class="arrow">{dropdownAbierto === 'dia' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'dia'}
              <div class="dropdown-panel">
                <div class="grid-10col">
                  {#each dias as d}
                    <button type="button" class="grid-btn" class:selected={diaFiltro === String(d)} on:click={() => seleccionarDia(d)}>
                      {d}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
        {#if filtroFecha}
          <button type="button" class="btn-limpiar" on:click={limpiarFiltroFecha}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
            Limpiar
          </button>
        {/if}
      </div>
    </div>


  </div>

  {#if cargando}
    <p>Cargando...</p>
  {:else if ordenesFiltradas.length === 0}
    <p class="vacio">No hay órdenes que cumplan con los filtros seleccionados</p>
  {:else}
    <div class="tabla-wrapper">
      <table>
        <thead>
          <tr>
            <th class="sortable" on:click={() => sortBy('codigo')}>
              Código OT
              {#if sortColumn === 'codigo'}
                <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
              {/if}
            </th>
            <th class="sortable" on:click={() => sortBy('jardin')}>
              Jardín
              {#if sortColumn === 'jardin'}
                <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
              {/if}
            </th>
            <th class="sortable" on:click={() => sortBy('fecha')}>
              Fecha Creación
              {#if sortColumn === 'fecha'}
                <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
              {/if}
            </th>
            <th class="th-alerta sortable" on:click={toggleSortAlerta}>
              <svg class="icono-header" class:active={sortAlerta !== null} width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                <line x1="12" y1="9" x2="12" y2="13"/>
                <line x1="12" y1="17" x2="12.01" y2="17"/>
              </svg>
              {#if sortAlerta !== null}
                <span class="sort-icon">{sortAlerta === 'asc' ? '▲' : '▼'}</span>
              {/if}
            </th>
            <th>Acciones</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedOrdenes as ot}
            <tr>
              <td>
                <button class="link" on:click={() => toggleDetalle(ot.id)}>
                  {ot.codigo}
                  <span class="icono">{expandido === ot.id ? '▼' : '▶'}</span>
                </button>
              </td>
              <td>{getNombreJardin(ot.jardinCodigo)}</td>
              <td>{formatearFecha(ot.fechaCreacion)}</td>
              <td class="alerta-cell">
                {#if ot.tieneAtrasados}
                  <svg class="icono-alerta" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                    <line x1="12" y1="9" x2="12" y2="13"/>
                    <line x1="12" y1="17" x2="12.01" y2="17"/>
                  </svg>
                {/if}
              </td>
              <td class="acciones-cell">
                <button class="btn-icono btn-imprimir" on:click={() => abrirModalImpresion(ot)} title="Imprimir">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="6 9 6 2 18 2 18 9"></polyline>
                    <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"></path>
                    <rect x="6" y="14" width="12" height="8"></rect>
                  </svg>
                </button>
                <button class="btn-icono btn-editar" on:click={() => abrirModalEdicion(ot)} title="Editar">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
                  </svg>
                </button>
                {#if itemAEliminar === ot.id}
                  <button on:click={() => eliminar(ot.id, ot.codigo)} class="btn-confirmar-eliminar">Eliminar</button>
                  <button on:click={cancelarEliminar} class="btn-cancelar">Cancelar</button>
                {:else}
                  <button class="btn-icono btn-eliminar" on:click={() => confirmarEliminar(ot.id)} title="Eliminar">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                  </button>
                {/if}
              </td>
            </tr>
            
            {#if expandido === ot.id && detalles[ot.id]}
              <tr class="detalle-row">
                <td colspan="4">
                  <div class="detalle">
                    <h3>Requerimientos Incluidos:</h3>
                    
                    {#if detalles[ot.id].requerimientos.length === 0}
                      <p class="vacio-detalle">No hay requerimientos asignados</p>
                    {:else}
                      <table class="tabla-detalle">
                        <thead>
                          <tr>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'recinto')}>
                              Zona
                              {#if detalleSortColumn[ot.id] === 'recinto'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'partida')}>
                              Item - Partida
                              {#if detalleSortColumn[ot.id] === 'partida'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'descripcion')}>
                              Descripción
                              {#if detalleSortColumn[ot.id] === 'descripcion'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'observaciones')}>
                              Observaciones
                              {#if detalleSortColumn[ot.id] === 'observaciones'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'fecha')}>
                              F. Inicio
                              {#if detalleSortColumn[ot.id] === 'fecha'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'plazo')}>
                              T. Plazo
                              {#if detalleSortColumn[ot.id] === 'plazo'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(ot.id, 'fecha_limite')}>
                              F. Límite
                              {#if detalleSortColumn[ot.id] === 'fecha_limite'}
                                <span class="sort-icon">{detalleSortDirection[ot.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th>Acciones</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each detallesSorted[ot.id] || detalles[ot.id].requerimientos as req}
                            <tr>
                              <td>{req.recinto}</td>
                              <td title="{req.partidaItem} - {req.partidaNombre}">{req.partidaItem} - {truncarTexto(req.partidaNombre, 5)}</td>
                              <td title={req.descripcion}>{truncarTexto(req.descripcion, 5)}</td>
                              <td title={req.observaciones}>{req.observaciones ? truncarTexto(req.observaciones, 5) : '-'}</td>
                              <td>{formatearFecha(req.fechaInicio)}</td>
                              <td>{req.plazoTotal || 0}</td>
                              <td class:fecha-atrasada={esFechaAtrasada(req.fechaLimite, req.fechaRecepcion)}>
                                {req.fechaLimite ? formatearFecha(req.fechaLimite) : '-'}
                              </td>
                              <td>
                                <button class="btn-icono btn-editar-req" on:click={() => abrirModalEdicionRequerimiento(req)} title="Editar requerimiento">
                                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
                                  </svg>
                                </button>
                              </td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    {/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

{#if modalAbierto}
  <ModalEditarOT 
    ot={otSeleccionada} 
    on:cerrar={cerrarModal}
  />
{/if}

{#if modalImpresionAbierto && datosImpresion}
  <ModalVistaImpresion
    ot={datosImpresion.ot}
    requerimientos={datosImpresion.requerimientos}
    on:cerrar={cerrarModalImpresion}
  />
{/if}

{#if modalReqAbierto && reqSeleccionado}
  <ModalEditarRequerimiento
    requerimiento={reqSeleccionado}
    on:cerrar={cerrarModalRequerimiento}
  />
{/if}

<style>
  .container { 
    padding: 0 2rem 2rem 2rem;
  }
  h2 { color: #7aafde; margin-bottom: 1.5rem; }

  /* Estilos de Filtros */
  .filtros {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #1a2332;
    border-radius: 8px;
    align-items: end;
  }
  
  .form-group {
    display: flex;
    flex-direction: column;
  }
  
  .form-group label {
    color: #7aafde;
    font-weight: 500;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
  }
  
  .form-group select {
    padding: 0.65rem;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    background: #0f1419;
    color: #e0e6ed;
    font-family: 'Inter', sans-serif;
    font-size: 0.9rem;
    transition: border-color 0.2s;
  }
  
  .form-group select:hover {
    border-color: #5a8fc4;
  }
  
  .form-group select:focus {
    outline: none;
    border-color: #5a8fc4;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    color: #7aafde;
    padding: 0.65rem;
    background: #0f1419;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .checkbox-label:hover {
    border-color: #5a8fc4;
    background: #1a2332;
  }

  .checkbox-label input[type="checkbox"] {
    width: 20px;
    height: 20px;
    cursor: pointer;
    accent-color: #d84545;
  }

  .checkbox-label span {
    font-size: 0.9rem;
    font-weight: 500;
  }
  
  /* Selector de fecha dropdown */
  .fecha-filtro-container {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
  }
  
  .fecha-grid {
    display: grid;
    grid-template-columns: 1fr 1.5fr 1fr;
    gap: 0.5rem;
    flex: 1;
  }

  .dropdown-wrapper { 
    position: relative; 
  }
  
  .dropdown-trigger {
    width: 100%;
    padding: 0.65rem;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    background: #0f1419;
    color: #e0e6ed;
    font-family: 'Inter', sans-serif;
    cursor: pointer;
    text-align: left;
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: border-color 0.2s;
    font-size: 0.9rem;
  }
  
  .dropdown-trigger:hover { 
    border-color: #5a8fc4; 
  }
  
  .arrow { 
    color: #7aafde; 
    font-size: 0.7rem; 
  }
  
  .dropdown-panel {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.25rem;
    background: #1a2332;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    padding: 0.5rem;
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    min-width: 100%;
    max-height: 300px;
    overflow-y: auto;
  }
  
  .grid-1col { 
    display: grid; 
    grid-template-columns: 1fr; 
    gap: 0.25rem; 
  }
  
  .grid-3col { 
    display: grid; 
    grid-template-columns: repeat(3, 1fr); 
    gap: 0.25rem; 
  }
  
  .grid-10col { 
    display: grid; 
    grid-template-columns: repeat(10, 1fr); 
    gap: 0.25rem; 
  }
  
  .grid-btn {
    padding: 0.5rem;
    border: 1px solid #2d3e50;
    background: #0f1419;
    color: #7aafde;
    cursor: pointer;
    border-radius: 4px;
    font-family: 'Inter', sans-serif;
    font-size: 0.85rem;
    transition: all 0.2s;
    white-space: nowrap;
  }
  
  .grid-btn:hover { 
    background: #202b38; 
    border-color: #5a8fc4; 
  }
  
  .grid-btn.selected {
    background: linear-gradient(135deg, #5a8fc4 0%, #4a7ba7 100%);
    color: #fff;
    border-color: #5a8fc4;
  }
  
  .btn-limpiar {
    padding: 0.65rem 0.875rem;
    border: 1px solid #2d3e50;
    background: #1a2332;
    color: #7aafde;
    border-radius: 6px;
    cursor: pointer;
    font-family: 'Inter', sans-serif;
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    gap: 0.375rem;
    transition: all 0.2s;
    white-space: nowrap;
  }
  
  .btn-limpiar:hover {
    background: #d84545;
    border-color: #d84545;
    color: #fff;
  }
  
  .btn-limpiar svg {
    display: block;
  }

  .vacio, .vacio-detalle {
    text-align: center;
    color: #7aafde;
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
    table-layout: auto;
  }

  thead th {
    background: #0f1419;
    color: #7aafde;
    padding: 1rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #2d3e50;
    position: relative;
  }

  thead th:last-child {
    min-width: 200px;
  }

  thead th.sortable {
    cursor: pointer;
    user-select: none;
  }

  thead th.sortable:hover {
    background: #1a2332;
    color: #7aafde;
  }

  .sort-icon {
    margin-left: 0.5rem;
    font-size: 0.8rem;
    color: #5a8fc4;
  }

  tbody td {
    padding: 1rem;
    color: #7aafde;
    border-bottom: 1px solid #2d3e50;
    vertical-align: middle;
  }

  tbody tr:hover {
    background: #0f1419;
  }

  .acciones-cell {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    min-height: 52px;
  }

  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
  }

  .btn-icono {
    padding: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 6px;
  }

  .btn-icono svg {
    width: 18px;
    height: 18px;
  }

  .btn-imprimir {
    background: #6a4c93;
    color: white;
  }

  .btn-imprimir:hover {
    background: #5a3c83;
    transform: translateY(-1px);
  }

  .link {
    background: transparent;
    color: #5a8fc4;
    text-decoration: underline;
    padding: 0;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .link:hover {
    color: #7aafde;
  }

  .icono {
    font-size: 0.8rem;
  }

  .btn-editar {
    background: #5a8fc4;
    color: white;
  }

  .btn-editar:hover {
    background: #4a7fb4;
    transform: translateY(-1px);
  }

  .btn-eliminar {
    background: #d32f2f;
    color: white;
  }

  .btn-eliminar:hover {
    background: #b71c1c;
    transform: translateY(-1px);
  }

  .btn-confirmar-eliminar {
    background: #8b0000;
    color: white;
    padding: 0.4rem 0.75rem;
    border: none;
    border-radius: 4px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-confirmar-eliminar:hover {
    background: #a00000;
    transform: translateY(-1px);
  }
  
  .btn-cancelar {
    background: #3d4f5f;
    color: #7aafde;
    padding: 0.4rem 0.75rem;
    border: none;
    border-radius: 4px;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-cancelar:hover {
    background: #4a5f73;
  }

  .btn-editar-req {
    background: #5a8fc4;
    color: white;
    padding: 0.4rem;
    width: 32px;
    height: 32px;
  }

  .btn-editar-req svg {
    width: 16px;
    height: 16px;
  }

  .btn-editar-req:hover {
    background: #4a7fb4;
    transform: translateY(-1px);
  }

  .detalle-row {
    background: #0f1419;
  }

  .detalle {
    padding: 1rem;
  }

  .detalle h3 {
    color: #7aafde;
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }

  .tabla-detalle {
    margin-top: 0.5rem;
    background: #1a2332;
    border-radius: 6px;
  }

  .tabla-detalle thead th {
    background: #2d3e50;
    font-size: 0.9rem;
    padding: 0.75rem 0.5rem;
  }

  .tabla-detalle tbody td {
    font-size: 0.9rem;
    padding: 0.875rem 0.5rem;
    vertical-align: middle;
  }

  .tabla-detalle tfoot td {
    background: #0f1419;
    border-top: 2px solid #2d3e50;
    padding: 1rem;
  }

  /* Columna de alerta */
  .th-alerta {
    text-align: center;
    width: 80px;
    min-width: 80px;
    cursor: pointer;
    user-select: none;
    position: relative;
  }

  .th-alerta:hover {
    background: #1a2332;
  }
  
  .icono-header {
    color: #7aafde;
    transition: all 0.3s;
    display: inline-block;
    vertical-align: middle;
  }
  
  .icono-header.active {
    color: #ff4444;
    animation: pulso-lento 2s ease-in-out infinite;
  }

  .alerta-cell {
    text-align: center;
    padding: 0.75rem 0.5rem;
  }

  .icono-alerta {
    color: #ff4444;
    display: inline-block;
    animation: pulso-lento 2s ease-in-out infinite;
  }
  
  @keyframes pulso-lento {
    0%, 100% { 
      opacity: 1;
      filter: drop-shadow(0 0 4px rgba(255, 68, 68, 0.8));
    }
    50% { 
      opacity: 0.5;
      filter: drop-shadow(0 0 8px rgba(255, 68, 68, 0.4));
    }
  }

  /* Fecha límite atrasada en rojo */
  .fecha-atrasada {
    color: #ff4444 !important;
    font-weight: 600;
    background: rgba(255, 68, 68, 0.1);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
  }
</style>
