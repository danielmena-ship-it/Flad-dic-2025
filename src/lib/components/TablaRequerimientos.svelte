<script>
  import { onMount, onDestroy } from 'svelte';
  import { getRequerimientos, deleteRequerimiento } from '$lib/utils/db-helpers.js';
  import { formatearFecha } from '$lib/utils/formatoFecha.js';
  import { jardines, cargarJardines } from '$lib/stores/catalogos.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';
  import ModalEditarRequerimiento from './ModalEditarRequerimiento.svelte';

  let requerimientos = [];
  let cargando = true;
  let modalAbierto = false;
  let requerimientoSeleccionado = null;
  let ordenColumna = null;
  let ordenDireccion = 'asc';
  let itemAEliminar = null;
  
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

  $: requerimientosFiltrados = filtrarRequerimientos(requerimientos, filtroJardin, filtroFecha);
  $: requerimientosOrdenados = ordenarRequerimientos(requerimientosFiltrados, ordenColumna, ordenDireccion);

  onMount(async () => {
    await cargarJardines();
    await cargarRequerimientos();
  });

  async function cargarRequerimientos() {
    cargando = true;
    try {
      console.log('🔍 Iniciando carga de requerimientos...');
      
      const timeoutPromise = new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Timeout: La carga tardó más de 15 segundos')), 15000)
      );
      
      const loadPromise = async () => {
        console.log('📡 Llamando getRequerimientos()...');
        const reqs = await getRequerimientos();
        console.log('✅ getRequerimientos() respondió:', reqs);
        console.log('📊 Cantidad de requerimientos:', Array.isArray(reqs) ? reqs.length : 'NO ES ARRAY');
        
        console.log('🔧 Enriqueciendo requerimientos...');
        const enriched = await enriquecerRequerimientos(reqs);
        console.log('✅ Enriquecimiento completo:', enriched);
        
        return enriched;
      };
      
      requerimientos = await Promise.race([loadPromise(), timeoutPromise]);
      console.log('✅ Carga completa. Total requerimientos:', requerimientos.length);
      
    } catch (error) {
      console.error('❌ ERROR COMPLETO:', error);
      console.error('❌ Error name:', error?.name);
      console.error('❌ Error message:', error?.message);
      console.error('❌ Error stack:', error?.stack);
      
      const errorMsg = `Error al cargar requerimientos:
      
Tipo: ${error?.name || 'Desconocido'}
Mensaje: ${error?.message || String(error)}

Stack trace:
${error?.stack || 'No disponible'}

Revisa DevTools (F12 o clic derecho > Inspeccionar) para más detalles.`;
      
      alert(errorMsg);
      requerimientos = [];
    } finally {
      cargando = false;
    }
  }

  function filtrarRequerimientos(reqs, jardin, fecha) {
    let resultado = [...reqs];
    
    // Filtrar por jardín si está seleccionado
    if (jardin) {
      resultado = resultado.filter(req => req.jardinCodigo === jardin);
    }
    
    // Filtrar por fecha si está seleccionada
    if (fecha) {
      const fechaFiltro = new Date(fecha);
      resultado = resultado.filter(req => {
        if (!req.fechaInicio) return false;
        const fechaReq = new Date(req.fechaInicio);
        return fechaReq >= fechaFiltro;
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

  function abrirModal(req) {
    requerimientoSeleccionado = { ...req };
    modalAbierto = true;
  }

  function confirmarEliminar(id) {
    itemAEliminar = id;
    document.addEventListener('keydown', handleKeyPress);
  }

  function handleKeyPress(e) {
    if (e.key === 'Enter' && itemAEliminar) {
      ejecutarEliminar();
    }
  }

  async function ejecutarEliminar() {
    if (itemAEliminar) {
      await deleteRequerimiento(itemAEliminar);
      await cargarRequerimientos();
      cancelarEliminar();
    }
  }

  function cancelarEliminar() {
    itemAEliminar = null;
    document.removeEventListener('keydown', handleKeyPress);
  }

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeyPress);
  });

  async function cerrarModal() {
    modalAbierto = false;
    requerimientoSeleccionado = null;
    await cargarRequerimientos();
  }

  function truncarPartida(nombre) {
    if (!nombre) return '-';
    const palabras = nombre.split(' ');
    return palabras.slice(0, 3).join(' ') + (palabras.length > 3 ? '...' : '');
  }

  function truncarDescripcion(texto) {
    if (!texto) return '-';
    const palabras = texto.split(' ');
    return palabras.slice(0, 3).join(' ') + (palabras.length > 3 ? '...' : '');
  }

  function extractCorrelativo(codigo) {
    // Extrae "M001" de "OT-BB-M001" o "IP-BB-M001"
    if (!codigo) return '-';
    const partes = codigo.split('-');
    return partes[partes.length - 1] || '-';
  }

  function ordenarPor(columna) {
    if (ordenColumna === columna) {
      ordenDireccion = ordenDireccion === 'asc' ? 'desc' : 'asc';
    } else {
      ordenColumna = columna;
      ordenDireccion = 'asc';
    }
  }

  function ordenarRequerimientos(reqs, columna, direccion) {
    if (!columna) return reqs;
    
    const sorted = [...reqs].sort((a, b) => {
      let valA = a[columna];
      let valB = b[columna];

      // Manejar valores numéricos
      if (columna === 'cantidad' || columna === 'plazo' || columna === 'precio_total') {
        valA = Number(valA) || 0;
        valB = Number(valB) || 0;
      }

      // Manejar fechas
      if (columna === 'fechaInicio') {
        valA = new Date(valA).getTime();
        valB = new Date(valB).getTime();
      }

      // Comparación
      if (valA < valB) return direccion === 'asc' ? -1 : 1;
      if (valA > valB) return direccion === 'asc' ? 1 : -1;
      return 0;
    });

    return sorted;
  }
</script>

<div class="contenedor">
  <h2 class="titulo">Lista de Requerimientos</h2>

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
              {#each $jardines as jardin (jardin.codigo)}
                <button type="button" class="grid-btn" class:selected={filtroJardin === jardin.codigo} on:click={() => seleccionarJardin(jardin.codigo)}>
                  [{jardin.codigo}] - {jardin.nombre}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="form-group">
      <label>Filtrar por Fecha de Ingreso (desde)</label>
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
  <p>Cargando requerimientos...</p>
{:else if requerimientosFiltrados.length === 0}
  <p>No hay requerimientos que cumplan con los filtros seleccionados</p>
{:else}
  <div class="tabla-container">
    <table>
      <thead>
        <tr>
          <th class="sortable" on:click={() => ordenarPor('jardin_codigo')}>
            Jardín {ordenColumna === 'jardin_codigo' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('recinto')}>
            Zona {ordenColumna === 'recinto' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('partida_nombre')}>
            Item - Partida {ordenColumna === 'partida_nombre' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('descripcion')}>
            Descripción {ordenColumna === 'descripcion' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('observaciones')}>
            Observaciones {ordenColumna === 'observaciones' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('cantidad')}>
            Cant. {ordenColumna === 'cantidad' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('fechaInicio')}>
            F.Inicio {ordenColumna === 'fechaInicio' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('plazo')}>
            Plazo {ordenColumna === 'plazo' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('plazoAdicional')}>
            P. Adic. {ordenColumna === 'plazoAdicional' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('fechaLimite')}>
            F.Límite {ordenColumna === 'fechaLimite' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('otCodigo')}>
            Código OT {ordenColumna === 'otCodigo' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" on:click={() => ordenarPor('informePagoCodigo')}>
            Código IP {ordenColumna === 'informePagoCodigo' ? (ordenDireccion === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th>Acciones</th>
        </tr>
      </thead>
      <tbody>
        {#each requerimientosOrdenados as req (req.id)}
          <tr>
            <td>{req.jardinCodigo}</td>
            <td>{req.recinto ? req.recinto.split(' - ')[0] : '-'}</td>
            <td title="{req.partidaItem} - {req.partidaNombre}">{req.partidaItem} - {truncarPartida(req.partidaNombre)}</td>
            <td title={req.descripcion}>{truncarDescripcion(req.descripcion)}</td>
            <td title={req.observaciones || ''}>{req.observaciones ? truncarDescripcion(req.observaciones) : '-'}</td>
            <td>{req.cantidad} {req.partidaUnidad || ''}</td>
            <td>{formatearFecha(req.fechaInicio)}</td>
            <td>{req.plazoDias}</td>
            <td>{req.plazoAdicional || '-'}</td>
            <td>{req.fechaLimite ? formatearFecha(req.fechaLimite) : '-'}</td>
            <td>{extractCorrelativo(req.otCodigo)}</td>
            <td>{extractCorrelativo(req.informePagoCodigo)}</td>
            <td class="acciones">
              <button on:click={() => abrirModal(req)} class="btn-icon" title="Editar">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
                </svg>
              </button>
              {#if itemAEliminar === req.id}
                <button on:click={ejecutarEliminar} class="btn-confirmar-eliminar">Eliminar</button>
                <button on:click={cancelarEliminar} class="btn-cancelar">Cancelar</button>
              {:else}
                <button on:click={() => confirmarEliminar(req.id)} class="btn-icon btn-delete" title="Eliminar">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  </svg>
                </button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

{#if modalAbierto}
  <ModalEditarRequerimiento 
    requerimiento={requerimientoSeleccionado} 
    on:actualizar={cerrarModal}
    on:cerrar={cerrarModal}
  />
{/if}
</div>

<style>
  .contenedor { 
    max-width: 1200px; 
    margin: 0 auto; 
    padding: 0 2rem 2rem 2rem;
  }
  .titulo { 
    text-align: left; 
    color: #7aafde; 
    font-size: 1.5rem; 
    font-weight: 600; 
    margin-bottom: 2rem; 
    letter-spacing: 0.02em;
  }
  
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
  
  /* Estilos de Tabla */
  .tabla-container { overflow-x: auto; background: #1a2332; border-radius: 8px; padding: 1rem; }
  table { width: 100%; border-collapse: collapse; table-layout: auto; min-width: 800px; }
  th, td { padding: 0.4rem 0.6rem; text-align: left; border-bottom: 1px solid #2d3e50; color: #e0e6ed; font-size: 0.85rem; }
  td.acciones { white-space: nowrap; }
  td:nth-child(1) { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } /* Jardín */
  td:nth-child(3) { max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } /* Item-Partida */
  td:nth-child(4) { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } /* Descripción */
  td:nth-child(5) { max-width: 100px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } /* Observaciones */
  th { background: #0f1419; font-weight: 600; color: #7aafde; cursor: pointer; user-select: none; white-space: nowrap; }
  th:hover { background: #1a2332; }
  th:last-child { cursor: default; width: 150px; }
  th:last-child:hover { background: #0f1419; }
  tr:hover { background: #202b38; }
  button { padding: 0.4rem 0.75rem; margin: 0 0.25rem; cursor: pointer; border: 1px solid #2d3e50; background: #1a2332; color: #7aafde; border-radius: 4px; font-size: 0.85rem; transition: all 0.2s; }
  button:hover { background: #5a8fc4; color: #fff; border-color: #5a8fc4; }
  .btn-icon { padding: 0.4rem; margin: 0 0.15rem; display: inline-flex; align-items: center; justify-content: center; border: none; }
  .btn-icon svg { display: block; }
  .btn-icon:first-of-type { background: #5a8fc4; color: white; }
  .btn-icon:first-of-type:hover { background: #4a7fb4; transform: translateY(-1px); }
  .btn-delete { background: #d32f2f; color: white; border: none; }
  .btn-delete:hover { background: #b71c1c; border-color: #b71c1c; transform: translateY(-1px); }
  
  .btn-confirmar-eliminar {
    background: #8b0000;
    color: white;
    padding: 0.35rem 0.6rem;
    border: none;
    border-radius: 4px;
    font-size: 0.8rem;
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
    padding: 0.35rem 0.6rem;
    border: none;
    border-radius: 4px;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-cancelar:hover {
    background: #4a5f73;
  }
  
  p { color: #7aafde; text-align: center; padding: 2rem; }
</style>
