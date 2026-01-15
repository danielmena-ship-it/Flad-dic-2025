<script>
  import { createEventDispatcher } from 'svelte';
  import { guardarFechasRecepcion, recalcularInforme } from '$lib/utils/db-helpers.js';

  export let requerimiento;
  const dispatch = createEventDispatcher();

  let mensajeError = ''; // ✅ FIX WINDOWS: Mensaje in-app (no alert)

  // Parsear fecha actual si existe
  let añoSeleccionado = '';
  let mesSeleccionado = '';
  let diaSeleccionado = '';
  
  if (requerimiento.fechaRecepcion) {
    const [año, mes, dia] = requerimiento.fechaRecepcion.split('-');
    añoSeleccionado = año;
    mesSeleccionado = mes;
    diaSeleccionado = dia;
  }
  
  let dropdownAbierto = null;
  let dropdownButton = null;
  let dropdownTop = 0;
  let dropdownLeft = 0;
  let dropdownWidth = 0;

  // Parsear fecha_inicio y fecha actual
  const [añoInicio, mesInicio, diaInicio] = requerimiento.fechaInicio.split('-').map(Number);
  const fechaHoy = new Date();
  const añoActual = fechaHoy.getFullYear();
  const mesActual = fechaHoy.getMonth() + 1;
  const diaActual = fechaHoy.getDate();
  
  const todosMeses = [
    { num: '01', nombre: 'Ene' }, { num: '02', nombre: 'Feb' }, { num: '03', nombre: 'Mar' },
    { num: '04', nombre: 'Abr' }, { num: '05', nombre: 'May' }, { num: '06', nombre: 'Jun' },
    { num: '07', nombre: 'Jul' }, { num: '08', nombre: 'Ago' }, { num: '09', nombre: 'Sep' },
    { num: '10', nombre: 'Oct' }, { num: '11', nombre: 'Nov' }, { num: '12', nombre: 'Dic' }
  ];

  // Años disponibles: desde año de inicio hasta año actual
  $: años = Array.from(
    { length: añoActual - añoInicio + 1 }, 
    (_, i) => añoInicio + i
  );

  // Meses disponibles según año seleccionado
  $: meses = (() => {
    if (!añoSeleccionado) return todosMeses;
    const añoNum = parseInt(añoSeleccionado);
    
    if (añoNum === añoInicio && añoNum === añoActual) {
      // Mismo año inicio y actual: filtrar por ambos
      return todosMeses.filter(m => {
        const mesNum = parseInt(m.num);
        return mesNum >= mesInicio && mesNum <= mesActual;
      });
    } else if (añoNum === añoInicio) {
      // Año de inicio: desde mes de inicio
      return todosMeses.filter(m => parseInt(m.num) >= mesInicio);
    } else if (añoNum === añoActual) {
      // Año actual: hasta mes actual
      return todosMeses.filter(m => parseInt(m.num) <= mesActual);
    }
    return todosMeses;
  })();

  // Días disponibles según mes y año seleccionados
  $: dias = (() => {
    if (!mesSeleccionado || !añoSeleccionado) return [];
    
    const añoNum = parseInt(añoSeleccionado);
    const mesNum = parseInt(mesSeleccionado);
    const diasEnMes = new Date(añoNum, mesNum, 0).getDate();
    const todosDias = Array.from({ length: diasEnMes }, (_, i) => i + 1);
    
    if (añoNum === añoInicio && mesNum === mesInicio && añoNum === añoActual && mesNum === mesActual) {
      // Mismo mes/año inicio y actual: filtrar por ambos días
      return todosDias.filter(d => d >= diaInicio && d <= diaActual);
    } else if (añoNum === añoInicio && mesNum === mesInicio) {
      // Mes de inicio: desde día de inicio
      return todosDias.filter(d => d >= diaInicio);
    } else if (añoNum === añoActual && mesNum === mesActual) {
      // Mes actual: hasta día actual
      return todosDias.filter(d => d <= diaActual);
    }
    return todosDias;
  })();
  
  $: fechaRecepcion = (añoSeleccionado && mesSeleccionado && diaSeleccionado) 
    ? `${añoSeleccionado}-${mesSeleccionado}-${String(diaSeleccionado).padStart(2, '0')}`
    : '';

  function toggleDropdown(tipo, e) {
    if (dropdownAbierto !== tipo && e?.currentTarget) {
      const rect = e.currentTarget.getBoundingClientRect();
      dropdownTop = rect.bottom + 4;
      dropdownLeft = rect.left;
      dropdownWidth = rect.width;
    }
    dropdownAbierto = dropdownAbierto === tipo ? null : tipo;
  }

  function seleccionarAño(año) {
    añoSeleccionado = String(año);
    // Validar mes seleccionado sigue siendo válido
    const mesNum = parseInt(mesSeleccionado);
    if (mesSeleccionado && !meses.find(m => parseInt(m.num) === mesNum)) {
      mesSeleccionado = '';
      diaSeleccionado = '';
    }
    dropdownAbierto = null;
  }

  function seleccionarMes(mes) {
    mesSeleccionado = mes;
    // Validar día seleccionado sigue siendo válido
    const diaNum = parseInt(diaSeleccionado);
    if (diaSeleccionado && !dias.includes(diaNum)) {
      diaSeleccionado = '';
    }
    dropdownAbierto = null;
  }

  function seleccionarDia(dia) {
    diaSeleccionado = String(dia);
    dropdownAbierto = null;
  }

  function getMesNombre(num) {
    return meses.find(m => m.num === num)?.nombre || '';
  }

  async function guardar() {
    if (!fechaRecepcion) {
      mensajeError = '❌ Debe seleccionar una fecha de recepción';
      setTimeout(() => mensajeError = '', 3000);
      return;
    }

    // ✅ Usar comando Rust que calcula multa correctamente vía trigger SQL
    await guardarFechasRecepcion([{
      id: requerimiento.id,
      fechaRecepcion: fechaRecepcion
    }]);
    
    // Recalcular informe si está asignado
    if (requerimiento.informePagoId) {
      await recalcularInforme(requerimiento.informePagoId);
    }
    
    dispatch('close');
  }

  function cancelar() {
    dispatch('close');
  }

  function handleKeydown(e) {
    if (e.key === 'Escape') cancelar();
  }

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget) cancelar();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="modal-overlay" on:click={handleOverlayClick}>
  <div class="modal-content" role="dialog" aria-modal="true" tabindex="-1">
    <h2>Editar Fecha de Recepción</h2>

    {#if mensajeError}
      <div class="mensaje-error">{mensajeError}</div>
    {/if}

    <form on:submit|preventDefault={guardar}>
      <div class="info-group">
        <p><strong>Jardín:</strong> {requerimiento.jardinNombre}</p>
        <p><strong>Zona:</strong> {requerimiento.recinto}</p>
        <p><strong>Partida:</strong> {requerimiento.partidaItem} - {requerimiento.partidaNombre}</p>
      </div>

      <div class="form-group">
        <div class="form-label">Fecha de Recepción</div>
        <div class="fecha-grid">
          <!-- Año -->
          <div class="dropdown-wrapper">
            <button 
              type="button" 
              class="dropdown-trigger" 
              on:click={(e) => toggleDropdown('año', e)}
            >
              {añoSeleccionado || 'Año'}
              <span class="arrow">{dropdownAbierto === 'año' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'año'}
              <div 
                class="dropdown-panel"
                style="top: {dropdownTop}px; left: {dropdownLeft}px; width: {dropdownWidth}px;"
              >
                <div class="grid-1col">
                  {#each años as año}
                    <button 
                      type="button" 
                      class="grid-btn" 
                      class:selected={añoSeleccionado === String(año)} 
                      on:click={() => seleccionarAño(año)}
                    >
                      {año}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Mes -->
          <div class="dropdown-wrapper">
            <button 
              type="button" 
              class="dropdown-trigger" 
              on:click={(e) => toggleDropdown('mes', e)}
            >
              {getMesNombre(mesSeleccionado) || 'Mes'}
              <span class="arrow">{dropdownAbierto === 'mes' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'mes'}
              <div 
                class="dropdown-panel"
                style="top: {dropdownTop}px; left: {dropdownLeft}px; width: {dropdownWidth}px;"
              >
                <div class="grid-3col">
                  {#each meses as mes}
                    <button 
                      type="button" 
                      class="grid-btn" 
                      class:selected={mesSeleccionado === mes.num} 
                      on:click={() => seleccionarMes(mes.num)}
                    >
                      {mes.nombre}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Día -->
          <div class="dropdown-wrapper">
            <button 
              type="button" 
              class="dropdown-trigger" 
              on:click={(e) => toggleDropdown('dia', e)}
            >
              {diaSeleccionado || 'Día'}
              <span class="arrow">{dropdownAbierto === 'dia' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownAbierto === 'dia'}
              <div 
                class="dropdown-panel dia-panel"
                style="top: {dropdownTop}px; left: {dropdownLeft}px;"
              >
                <div class="grid-10col">
                  {#each dias as dia}
                    <button 
                      type="button" 
                      class="grid-btn" 
                      class:selected={diaSeleccionado === String(dia)} 
                      on:click={() => seleccionarDia(dia)}
                    >
                      {dia}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button type="button" on:click={cancelar}>Cancelar</button>
        <button type="submit">Guardar</button>
      </div>
    </form>
  </div>
</div>

<style>
  .modal-overlay { 
    position: fixed; 
    top: 0; 
    left: 0; 
    right: 0; 
    bottom: 0; 
    background: rgba(0, 0, 0, 0.75); 
    display: flex; 
    align-items: center; 
    justify-content: center; 
    z-index: 1000; 
  }
  
  .modal-content { 
    background: #1a2332; 
    padding: 2rem; 
    border-radius: 12px; 
    max-width: 500px; 
    width: 90%; 
    border: 1px solid #2d3e50; 
  }
  
  h2 { 
    color: #a8c5e0; 
    margin-bottom: 1.5rem; 
    font-size: 1.25rem; 
  }
  
  .info-group {
    background: #0f1419;
    padding: 1rem;
    border-radius: 6px;
    margin-bottom: 1.5rem;
    border: 1px solid #2d3e50;
  }
  
  .info-group p {
    color: #e0e6ed;
    margin: 0.5rem 0;
    font-size: 0.9rem;
  }
  
  .info-group strong {
    color: #a8c5e0;
  }
  
  .form-group { 
    margin-bottom: 1rem; 
  }
  
  /* Fecha Grid */
  .fecha-grid {
    display: grid;
    grid-template-columns: 1fr 1.5fr 1fr;
    gap: 0.5rem;
  }

  /* Dropdown */
  .dropdown-wrapper { 
    position: relative; 
  }
  
  .dropdown-trigger {
    width: 100%;
    padding: 0.75rem;
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
    font-size: 0.95rem;
  }
  
  .dropdown-trigger:hover { 
    border-color: #5a8fc4; 
  }
  
  .arrow { 
    color: #a8c5e0; 
    font-size: 0.7rem; 
  }
  
  .dropdown-panel {
    position: fixed;
    background: #1a2332;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    padding: 0.75rem;
    z-index: 1001;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
  
  .dia-panel {
    min-width: 480px;
    width: max-content;
  }
  
  .grid-1col { 
    display: grid; 
    grid-template-columns: 1fr; 
    gap: 0.4rem; 
  }
  
  .grid-3col { 
    display: grid; 
    grid-template-columns: repeat(3, 1fr); 
    gap: 0.4rem; 
  }
  
  .grid-10col { 
    display: grid; 
    grid-template-columns: repeat(10, 1fr); 
    gap: 0.4rem; 
  }
  
  .grid-btn {
    padding: 0.5rem;
    border: 1px solid #2d3e50;
    background: #0f1419;
    color: #a8c5e0;
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
  
  /* Acciones Modal */
  .modal-actions { 
    display: flex; 
    justify-content: flex-end; 
    gap: 0.75rem; 
    margin-top: 1.5rem; 
  }
  
  button { 
    padding: 0.6rem 1.25rem; 
    cursor: pointer; 
    border-radius: 6px; 
    font-family: 'Inter', sans-serif; 
    font-weight: 500; 
    transition: all 0.2s; 
  }
  
  button[type="submit"] { 
    background: linear-gradient(135deg, #5a8fc4 0%, #4a7ba7 100%); 
    color: #fff; 
    border: none; 
  }
  
  button[type="submit"]:hover { 
    background: linear-gradient(135deg, #6a9fd4 0%, #5a8bb7 100%); 
  }
  
  button[type="button"] { 
    background: transparent; 
    color: #8b9eb3; 
    border: 1px solid #2d3e50; 
  }
  
  button[type="button"]:hover { 
    background: #0f1419; 
  }
</style>
