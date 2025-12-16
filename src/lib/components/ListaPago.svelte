<script>
  import { onMount, onDestroy } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { getInformesPago, getInformePagoDetalle, eliminarInformePago } from '$lib/utils/db-helpers.js';
  import { formatearNumero } from '$lib/utils/calculos.js';
  import { jardines, cargarJardines } from '$lib/stores/catalogos.js';
  import { enriquecerRequerimientos, enriquecerInformesPago } from '$lib/utils/enriquecimiento.js';
  import ModalEditarInforme from './ModalEditarInforme.svelte';
  import ModalVistaImpresionInforme from './ModalVistaImpresionInforme.svelte';

  let informes = [];
  let cargando = true;
  let expandido = null;
  let detalles = {};
  let modalAbierto = false;
  let informeSeleccionado = null;
  let modalImpresionAbierto = false;
  let informeImpresion = null;
  let requerimientosImpresion = [];
  let sortColumn = null;
  let sortDirection = 'asc';
  let detalleSortColumn = {};
  let detalleSortDirection = {};
  let itemAEliminar = null;
  let mensajeError = ''; // ✅ FIX WINDOWS: Mensaje in-app (no alert)
  
  // Filtros
  let filtroJardin = '';
  let filtroFecha = '';
  
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

  $: informesFiltrados = filtrarInformes(informes, filtroJardin, filtroFecha);

  onMount(async () => {
    await cargarJardines();
    await cargarInformes();
  });

  function getNombreJardin(codigo) {
    const jardin = $jardines.find(j => j.codigo === codigo);
    return jardin ? jardin.nombre : codigo;
  }

  async function cargarInformes() {
    cargando = true;
    const data = await getInformesPago();
    informes = await enriquecerInformesPago(data);
    cargando = false;
  }
  
  function filtrarInformes(infs, jardin, fecha) {
    let resultado = [...infs];
    
    if (jardin) {
      resultado = resultado.filter(inf => inf.jardinCodigo === jardin);
    }
    
    if (fecha) {
      const fechaFiltro = new Date(fecha);
      resultado = resultado.filter(inf => {
        if (!inf.fechaCreacion) return false;
        const fechaInf = new Date(inf.fechaCreacion);
        return fechaInf >= fechaFiltro;
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

  async function toggleDetalle(inf_id) {
    if (expandido === inf_id) {
      expandido = null;
    } else {
      expandido = inf_id;
      if (!detalles[inf_id]) {
        const detalle = await getInformePagoDetalle(inf_id);
        detalle.requerimientos = await enriquecerRequerimientos(detalle.requerimientos);
        detalles[inf_id] = detalle;
        detalles = detalles;
      }
    }
  }

  function abrirModalEdicion(inf) {
    informeSeleccionado = inf;
    modalAbierto = true;
  }

  async function abrirModalImpresion(inf) {
    informeImpresion = inf;
    // Cargar detalles si no existen
    if (!detalles[inf.id]) {
      const detalle = await getInformePagoDetalle(inf.id);
      detalle.requerimientos = await enriquecerRequerimientos(detalle.requerimientos);
      detalles[inf.id] = detalle;
      detalles = detalles;
    }
    requerimientosImpresion = detalles[inf.id].requerimientos;
    modalImpresionAbierto = true;
  }

  function cerrarModalImpresion() {
    modalImpresionAbierto = false;
    informeImpresion = null;
    requerimientosImpresion = [];
  }

  async function cerrarModal() {
    modalAbierto = false;
    informeSeleccionado = null;
    await cargarInformes();
    detalles = {};
  }

  async function eliminar(inf_id, codigo) {
    try {
      await eliminarInformePago(inf_id);
      await cargarInformes();
      if (expandido === inf_id) {
        expandido = null;
      }
      delete detalles[inf_id];
      detalles = detalles;
      cancelarEliminar();
    } catch (error) {
      mensajeError = '❌ Error al eliminar informe: ' + error.message;
      setTimeout(() => mensajeError = '', 4000);
    }
  }

  function confirmarEliminar(inf_id) {
    itemAEliminar = inf_id;
    document.addEventListener('keydown', handleKeyPress);
  }

  function handleKeyPress(e) {
    if (e.key === 'Enter' && itemAEliminar) {
      const informe = informes.find(i => i.id === itemAEliminar);
      if (informe) {
        eliminar(informe.id, informe.codigo);
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
    sortColumn = sortColumn;
  }

  function sortDetalleBy(inf_id, column) {
    if (detalleSortColumn[inf_id] === column) {
      detalleSortDirection[inf_id] = detalleSortDirection[inf_id] === 'asc' ? 'desc' : 'asc';
    } else {
      detalleSortColumn[inf_id] = column;
      detalleSortDirection[inf_id] = 'asc';
    }
    detalleSortColumn = { ...detalleSortColumn };
    detalleSortDirection = { ...detalleSortDirection };
  }

  $: detallesSorted = (() => {
    const _ = detalleSortColumn;
    const __ = detalleSortDirection;
    
    return Object.keys(detalles).reduce((acc, inf_id) => {
      acc[inf_id] = getSortedRequerimientos(Number(inf_id), detalles[inf_id].requerimientos);
      return acc;
    }, {});
  })();

  function truncarTexto(texto, maxPalabras = 5) {
    if (!texto) return '-';
    const palabras = texto.split(' ');
    return palabras.slice(0, maxPalabras).join(' ') + (palabras.length > maxPalabras ? '...' : '');
  }

  function getSortedRequerimientos(inf_id, requerimientos) {
    const column = detalleSortColumn[inf_id];
    const direction = detalleSortDirection[inf_id] || 'asc';
    
    if (!column) return requerimientos;
    
    return [...requerimientos].sort((a, b) => {
      let valA, valB;
      let result = 0;
      
      switch(column) {
        case 'fecha':
          valA = new Date(a.fechaRecepcion).getTime();
          valB = new Date(b.fechaRecepcion).getTime();
          result = valA - valB;
          break;
        case 'fecha_limite':
          valA = a.fechaLimite ? new Date(a.fechaLimite).getTime() : 0;
          valB = b.fechaLimite ? new Date(b.fechaLimite).getTime() : 0;
          result = valA - valB;
          break;
        case 'plazo':
          valA = parseInt(a.plazoTotal) || 0;
          valB = parseInt(b.plazoTotal) || 0;
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
        case 'multa':
          valA = parseFloat(a.multa) || 0;
          valB = parseFloat(b.multa) || 0;
          result = valA - valB;
          break;
        case 'a_pago':
          valA = parseFloat(a.aPago) || 0;
          valB = parseFloat(b.aPago) || 0;
          result = valA - valB;
          break;
      }
      
      return direction === 'asc' ? result : -result;
    });
  }

  function formatearFecha(fechaString) {
    if (!fechaString) return '-';
    const [año, mes, dia] = fechaString.split('-');
    return `${dia}/${mes}/${año}`;
  }

  function extraerNumeroZona(zona) {
    const match = zona.match(/^(\d+)/);
    return match ? match[1] : zona;
  }

  $: sortedInformes = !sortColumn ? informesFiltrados : [...informesFiltrados].sort((a, b) => {
    let valA, valB;
    let result = 0;
    
    switch(sortColumn) {
      case 'codigo':
        valA = (a.codigo || '').toLowerCase();
        valB = (b.codigo || '').toLowerCase();
        result = valA.localeCompare(valB);
        break;
      case 'jardin':
        valA = (a.jardinCodigo || '').toLowerCase();
        valB = (b.jardinCodigo || '').toLowerCase();
        result = valA.localeCompare(valB);
        break;
      case 'fecha':
        valA = new Date(a.fechaCreacion).getTime();
        valB = new Date(b.fechaCreacion).getTime();
        result = valA - valB;
        break;
      case 'total_pagar':
        valA = parseFloat(a.totalPagar) || 0;
        valB = parseFloat(b.totalPagar) || 0;
        result = valA - valB;
        break;
    }
    
    return sortDirection === 'asc' ? result : -result;
  });
</script>

<div class="container">
  <h2>Lista de Informes de Pago</h2>

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
      <label>Filtrar por F. Recepción (desde)</label>
      <div class="fecha-filtro-container">
        <div class="fecha-grid">
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('año')}>
              {añoFiltro || 'Año'}
              <span class="arrow">{dropdownAbierto === 'año' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'año'}
              <div class="dropdown-panel">
                <div class="grid-1col">
                  {#each años as a}
                    <button type="button" class="grid-btn" class:selected={añoFiltro === String(a)} on:click={() => seleccionarAño(a)}>{a}</button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('mes')}>
              {getMesNombre(mesFiltro) || 'Mes'}
              <span class="arrow">{dropdownAbierto === 'mes' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'mes'}
              <div class="dropdown-panel">
                <div class="grid-3col">
                  {#each meses as m}
                    <button type="button" class="grid-btn" class:selected={mesFiltro === m.num} on:click={() => seleccionarMes(m.num)}>{m.nombre}</button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger" on:click={() => toggleDropdown('dia')}>
              {diaFiltro || 'Día'}
              <span class="arrow">{dropdownAbierto === 'dia' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'dia'}
              <div class="dropdown-panel">
                <div class="grid-10col">
                  {#each dias as d}
                    <button type="button" class="grid-btn" class:selected={diaFiltro === String(d)} on:click={() => seleccionarDia(d)}>{d}</button>
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
  {:else if informesFiltrados.length === 0}
    <p class="vacio">No hay informes que cumplan con los filtros seleccionados</p>
  {:else}
    <div class="tabla-wrapper">
      <table>
        <thead>
          <tr>
            <th class="sortable" on:click={() => sortBy('codigo')}>
              Código Informe
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
              F. Creación
              {#if sortColumn === 'fecha'}
                <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
              {/if}
            </th>
            <th class="sortable" on:click={() => sortBy('total_pagar')}>
              Total a Pagar
              {#if sortColumn === 'total_pagar'}
                <span class="sort-icon">{sortDirection === 'asc' ? '▲' : '▼'}</span>
              {/if}
            </th>
            <th>Acciones</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedInformes as inf}
            <tr>
              <td>
                <button class="link" on:click={() => toggleDetalle(inf.id)}>
                  {inf.codigo}
                  <span class="icono">{expandido === inf.id ? '▼' : '▶'}</span>
                </button>
              </td>
              <td>{inf.jardinNombre}</td>
              <td>{new Date(inf.fechaCreacion).toLocaleDateString()}</td>
              <td class="monto">${formatearNumero(inf.totalPagar || 0)}</td>
              <td class="acciones-cell">
                <button class="btn-icono btn-imprimir" on:click={() => abrirModalImpresion(inf)} title="Imprimir">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="6 9 6 2 18 2 18 9"></polyline>
                    <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"></path>
                    <rect x="6" y="14" width="12" height="8"></rect>
                  </svg>
                </button>
                <button class="btn-icono btn-editar" on:click={() => abrirModalEdicion(inf)} title="Editar">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
                  </svg>
                </button>
                {#if itemAEliminar === inf.id}
                  <button on:click={() => eliminar(inf.id, inf.codigo)} class="btn-confirmar-eliminar">Eliminar</button>
                  <button on:click={cancelarEliminar} class="btn-cancelar">Cancelar</button>
                {:else}
                  <button class="btn-icono btn-eliminar" on:click={() => confirmarEliminar(inf.id)} title="Eliminar">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                  </button>
                {/if}
              </td>
            </tr>
            
            {#if expandido === inf.id && detalles[inf.id]}
              <tr class="detalle-row">
                <td colspan="5">
                  <div class="detalle">
                    <h3>Requerimientos Incluidos:</h3>
                    
                    {#if detalles[inf.id].requerimientos.length === 0}
                      <p class="vacio-detalle">No hay requerimientos asignados</p>
                    {:else}
                      <table class="tabla-detalle">
                        <thead>
                          <tr>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'recinto')}>
                              Zona
                              {#if detalleSortColumn[inf.id] === 'recinto'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'partida')}>
                              Item - Partida
                              {#if detalleSortColumn[inf.id] === 'partida'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'cantidad')}>
                              Cantidad
                              {#if detalleSortColumn[inf.id] === 'cantidad'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'fecha_limite')}>
                              F. Límite
                              {#if detalleSortColumn[inf.id] === 'fecha_limite'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'plazo')}>
                              T. Plazo
                              {#if detalleSortColumn[inf.id] === 'plazo'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'fecha')}>
                              F. Recepción
                              {#if detalleSortColumn[inf.id] === 'fecha'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'dias_atraso')}>
                              Días Atraso
                              {#if detalleSortColumn[inf.id] === 'dias_atraso'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'precio')}>
                              Precio
                              {#if detalleSortColumn[inf.id] === 'precio'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'multa')}>
                              Multa
                              {#if detalleSortColumn[inf.id] === 'multa'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                            <th class="sortable" on:click={() => sortDetalleBy(inf.id, 'a_pago')}>
                              A Pago
                              {#if detalleSortColumn[inf.id] === 'a_pago'}
                                <span class="sort-icon">{detalleSortDirection[inf.id] === 'asc' ? '▲' : '▼'}</span>
                              {/if}
                            </th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each detallesSorted[inf.id] || detalles[inf.id].requerimientos as req}
                            <tr>
                              <td>{extraerNumeroZona(req.recinto)}</td>
                              <td title="{req.partidaItem} - {req.partidaNombre}">{req.partidaItem} - {truncarTexto(req.partidaNombre, 5)}</td>
                              <td>{formatearNumero(req.cantidad)} {req.partidaUnidad}</td>
                              <td>{formatearFecha(req.fechaLimite)}</td>
                              <td>{req.plazoTotal || '-'}</td>
                              <td>{formatearFecha(req.fechaRecepcion)}</td>
                              <td class:atraso={req.diasAtraso > 0} class:adelanto={req.diasAtraso < 0}>
                                {req.diasAtraso || 0}
                              </td>
                              <td>${formatearNumero(req.precioTotal)}</td>
                              <td>${formatearNumero(req.multa || 0)}</td>
                              <td>${formatearNumero(req.aPago || 0)}</td>
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
  <ModalEditarInforme 
    informe={informeSeleccionado} 
    on:cerrar={cerrarModal}
  />
{/if}

{#if modalImpresionAbierto}
  <ModalVistaImpresionInforme 
    informe={informeImpresion} 
    requerimientos={requerimientosImpresion}
    on:cerrar={cerrarModalImpresion}
  />
{/if}

<style>
  .container { 
    padding: 0 2rem 2rem 2rem;
  }
  h2 { color: #7aafde; margin-bottom: 1.5rem; }

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
  
  .form-group select:hover { border-color: #5a8fc4; }
  .form-group select:focus { outline: none; border-color: #5a8fc4; }
  
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

  .dropdown-wrapper { position: relative; }
  
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
  
  .dropdown-trigger:hover { border-color: #5a8fc4; }
  .arrow { color: #7aafde; font-size: 0.7rem; }
  
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
  
  .grid-1col { display: grid; grid-template-columns: 1fr; gap: 0.25rem; }
  .grid-3col { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.25rem; }
  .grid-10col { display: grid; grid-template-columns: repeat(10, 1fr); gap: 0.25rem; }
  
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
  
  .grid-btn:hover { background: #202b38; border-color: #5a8fc4; }
  
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
  
  .btn-limpiar:hover { background: #d84545; border-color: #d84545; color: #fff; }
  .btn-limpiar svg { display: block; }

  .vacio, .vacio-detalle { text-align: center; color: #7aafde; padding: 2rem; }

  .tabla-wrapper {
    overflow-x: auto;
    background: #1a2332;
    border-radius: 8px;
    padding: 1rem;
  }

  table { width: 100%; border-collapse: collapse; table-layout: auto; }

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

  thead th.sortable { cursor: pointer; user-select: none; }
  thead th.sortable:hover { background: #1a2332; color: #7aafde; }

  .sort-icon { margin-left: 0.5rem; font-size: 0.8rem; color: #5a8fc4; }

  tbody td { padding: 0.75rem 1rem; color: #7aafde; border-bottom: 1px solid #2d3e50; }
  tbody tr:hover { background: #0f1419; }

  .monto { color: #4caf50; font-weight: 600; }

  .acciones-cell { display: flex; gap: 0.5rem; align-items: center; }

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

  .btn-icono svg { width: 18px; height: 18px; }
  .btn-imprimir { background: #6a4c93; color: white; }
  .btn-imprimir:hover { background: #5a3c83; transform: translateY(-1px); }

  .btn-editar { background: #5a8fc4; color: white; }
  .btn-editar:hover { background: #4a7fb4; transform: translateY(-1px); }

  .btn-eliminar { background: #d32f2f; color: white; }
  .btn-eliminar:hover { background: #b71c1c; transform: translateY(-1px); }

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

  .link {
    background: transparent;
    color: #5a8fc4;
    text-decoration: underline;
    padding: 0;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .link:hover { color: #7aafde; }
  .icono { font-size: 0.8rem; }

  .detalle-row { background: #0f1419; }
  .detalle { padding: 1rem; }
  .detalle h3 { color: #7aafde; margin-bottom: 1rem; font-size: 1.1rem; }

  .tabla-detalle { margin-top: 0.5rem; background: #1a2332; border-radius: 6px; }
  .tabla-detalle thead th { background: #2d3e50; font-size: 0.9rem; }
  .tabla-detalle tbody td { font-size: 0.9rem; }
</style>
