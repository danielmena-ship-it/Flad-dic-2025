<script>
  import { createEventDispatcher } from 'svelte';
  import { updateRequerimiento } from '$lib/utils/db-helpers.js';
  import { calcularDiasMaximoPlazoAdicional } from '$lib/utils/calculos.js';

  export let requerimiento;

  const dispatch = createEventDispatcher();

  // Todos los campos editables
  let descripcion = requerimiento.descripcion || '';
  let observaciones = requerimiento.observaciones || '';
  let cantidad = requerimiento.cantidad || 1;
  let plazo = requerimiento.plazo || requerimiento.plazoDias || 0;
  let plazoAdicional = requerimiento.plazoAdicional || 0;
  
  let guardando = false;
  let mensaje = '';
  let dropdownPlazoAbierto = false;
  let dropdownPlazoAdicionalAbierto = false;
  let dropdownButton;
  let dropdownButtonAdicional;
  let dropdownTop = 0;
  let dropdownLeft = 0;
  let dropdownWidth = 0;
  let dropdownTopAdicional = 0;
  let dropdownLeftAdicional = 0;
  let dropdownWidthAdicional = 0;
  let dropdownFechaAbierto = null; // 'anio', 'mes', 'dia', o null
  
  // Parsear fechaInicio
  const fechaInicial = requerimiento.fechaInicio || new Date().toISOString().split('T')[0];
  const fechaPartes = fechaInicial.split('-');
  let anio = fechaPartes[0];
  let mes = fechaPartes[1];
  let dia = fechaPartes[2];
  let fechaInicio = fechaInicial;

  // Actualizar fecha cuando cambien los selectores
  $: {
    if (anio && mes && dia) {
      fechaInicio = `${anio}-${mes}-${String(dia).padStart(2, '0')}`;
    }
  }

  // Fecha actual para validaciones
  const hoy = new Date();
  const anioActual = hoy.getFullYear();
  const mesActual = String(hoy.getMonth() + 1).padStart(2, '0');
  const diaActual = hoy.getDate();
  
  // Generar años desde el año actual hasta 2030
  const anios = Array.from({length: 2031 - anioActual}, (_, i) => anioActual + i);
  
  // Meses
  const meses = [
    { num: '01', nombre: 'Ene' },
    { num: '02', nombre: 'Feb' },
    { num: '03', nombre: 'Mar' },
    { num: '04', nombre: 'Abr' },
    { num: '05', nombre: 'May' },
    { num: '06', nombre: 'Jun' },
    { num: '07', nombre: 'Jul' },
    { num: '08', nombre: 'Ago' },
    { num: '09', nombre: 'Sep' },
    { num: '10', nombre: 'Oct' },
    { num: '11', nombre: 'Nov' },
    { num: '12', nombre: 'Dic' }
  ];
  
  // Días disponibles según el mes y año
  $: diasEnMes = (() => {
    const fecha = new Date(parseInt(anio), parseInt(mes), 0);
    return fecha.getDate();
  })();
  
  $: dias = Array.from({length: diasEnMes}, (_, i) => i + 1);

  // Verificar si un año está deshabilitado (anterior al actual)
  function esAnioDeshabilitado(anioCheck) {
    return parseInt(anioCheck) < anioActual;
  }

  // Verificar si un mes está deshabilitado (anterior al mes actual en el año actual)
  function esMesDeshabilitado(mesCheck) {
    return parseInt(anio) === anioActual && parseInt(mesCheck) < parseInt(mesActual);
  }

  // Verificar si una fecha es anterior al día actual
  function esFechaAnterior(anioCheck, mesCheck, diaCheck) {
    const hoyDate = new Date();
    hoyDate.setHours(0, 0, 0, 0);
    const fechaCheck = new Date(parseInt(anioCheck), parseInt(mesCheck) - 1, parseInt(diaCheck));
    return fechaCheck < hoyDate;
  }

  function cerrar() {
    dispatch('cerrar');
  }

  function toggleDropdownFecha(tipo) {
    dropdownFechaAbierto = dropdownFechaAbierto === tipo ? null : tipo;
  }

  function seleccionarAnio(a) {
    anio = String(a);
    dropdownFechaAbierto = null;
  }

  function seleccionarMes(m) {
    mes = m;
    dropdownFechaAbierto = null;
  }

  function seleccionarDia(d) {
    dia = String(d);
    dropdownFechaAbierto = null;
  }

  function getMesNombre(num) {
    return meses.find(m => m.num === num)?.nombre || '';
  }

  function toggleDropdownPlazo() {
    dropdownPlazoAbierto = !dropdownPlazoAbierto;
  }

  function seleccionarPlazoBase(dias) {
    plazo = dias;
    dropdownPlazoAbierto = false;
  }

  function toggleDropdownPlazoAdicional() {
    dropdownPlazoAdicionalAbierto = !dropdownPlazoAdicionalAbierto;
  }

  function seleccionarPlazo(dias) {
    plazoAdicional = dias;
    dropdownPlazoAdicionalAbierto = false;
  }

  // Calcular días disponibles para plazo adicional (< 50% del plazo)
  $: diasDisponiblesPlazoAdicional = (() => {
    const maxDias = calcularDiasMaximoPlazoAdicional(plazo);
    return maxDias > 0 ? Array.from({length: maxDias}, (_, i) => i + 1) : [];
  })();

  async function guardar() {
    guardando = true;
    mensaje = '';

    try {
      const dataToUpdate = {
        descripcion,
        observaciones,
        cantidad: parseFloat(cantidad),
        fechaInicio: fechaInicio,
        plazoDias: parseInt(plazo, 10),
        plazoAdicional: parseInt(plazoAdicional, 10)
      };
      
      if (isNaN(dataToUpdate.plazoDias) || isNaN(dataToUpdate.plazoAdicional)) {
        throw new Error('Plazo inválido');
      }
      
      await updateRequerimiento(requerimiento.id, dataToUpdate);
      dispatch('actualizar');
      dispatch('cerrar');
    } catch (error) {
      mensaje = `❌ Error: ${error?.message || error?.toString() || 'Error desconocido'}`;
      guardando = false;
    }
  }

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget) {
      cerrar();
    }
  }
</script>

<div 
  class="overlay" 
  role="presentation"
  on:click={handleOverlayClick}
  on:keydown={(e) => e.key === 'Escape' && cerrar()}
>
  <div 
    class="modal"
    role="dialog"
    aria-modal="true"
  >
    <div class="header">
      <h2>Editar Requerimiento</h2>
      <button class="btn-cerrar" on:click={cerrar}>✕</button>
    </div>

    <div class="content">
      <div class="info-req">
        <p><strong>Recinto:</strong> {requerimiento.recinto}</p>
        <p><strong>Partida:</strong> {requerimiento.partidaItem} - {requerimiento.partidaNombre}</p>
      </div>

      <div class="form-group">
        <label for="descripcion">Descripción</label>
        <textarea 
          id="descripcion" 
          bind:value={descripcion}
          rows="3"
          placeholder="Descripción del requerimiento..."
        />
      </div>

      <div class="form-group">
        <label for="observaciones">Observaciones</label>
        <textarea 
          id="observaciones" 
          bind:value={observaciones}
          rows="3"
          placeholder="Ingrese observaciones adicionales..."
        />
      </div>

      <div class="form-group">
        <label for="cantidad">Cantidad</label>
        <input 
          type="number" 
          id="cantidad" 
          bind:value={cantidad}
          min="1"
          step="1"
        />
      </div>

      <div class="form-group">
        <label>Fecha Inicio</label>
        <div class="fecha-grid">
          <!-- Año -->
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger-fecha" on:click={() => toggleDropdownFecha('anio')}>
              {anio || 'Año'}
              <span class="arrow">{dropdownFechaAbierto === 'anio' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownFechaAbierto === 'anio'}
              <div class="dropdown-panel">
                <div class="grid-3col">
                  {#each anios as a}
                    {@const deshabilitado = esAnioDeshabilitado(a)}
                    <button 
                      type="button" 
                      class="grid-btn" 
                      class:selected={anio === String(a)}
                      class:disabled={deshabilitado}
                      disabled={deshabilitado}
                      on:click={() => seleccionarAnio(a)}
                    >
                      {a}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Mes -->
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger-fecha" on:click={() => toggleDropdownFecha('mes')}>
              {getMesNombre(mes) || 'Mes'}
              <span class="arrow">{dropdownFechaAbierto === 'mes' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownFechaAbierto === 'mes'}
              <div class="dropdown-panel">
                <div class="grid-4col">
                  {#each meses as m}
                    {@const deshabilitado = esMesDeshabilitado(m.num)}
                    <button 
                      type="button" 
                      class="grid-btn" 
                      class:selected={mes === m.num}
                      class:disabled={deshabilitado}
                      disabled={deshabilitado}
                      on:click={() => seleccionarMes(m.num)}
                    >
                      {m.nombre}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Día -->
          <div class="dropdown-wrapper">
            <button type="button" class="dropdown-trigger-fecha" on:click={() => toggleDropdownFecha('dia')}>
              {dia || 'Día'}
              <span class="arrow">{dropdownFechaAbierto === 'dia' ? '▲' : '▼'}</span>
            </button>
            {#if dropdownFechaAbierto === 'dia'}
              <div class="dropdown-panel dropdown-dias">
                <div class="grid-10col">
                  {#each dias as d}
                    {@const deshabilitado = esFechaAnterior(anio, mes, d)}
                    <button 
                      type="button" 
                      class="grid-btn" 
                      class:selected={dia === String(d)}
                      class:disabled={deshabilitado}
                      disabled={deshabilitado}
                      on:click={() => seleccionarDia(d)}
                    >
                      {d}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <div class="form-group">
        <label for="plazo">Plazo (días)</label>
        <div class="dropdown-wrapper">
          <button 
            type="button" 
            class="dropdown-trigger" 
            on:click={toggleDropdownPlazo}
            bind:this={dropdownButton}
          >
            {plazo} días
            <span class="arrow">{dropdownPlazoAbierto ? '▲' : '▼'}</span>
          </button>
          {#if dropdownPlazoAbierto}
            <div 
              class="dropdown-panel"
              style="top: {dropdownTop}px; left: {dropdownLeft}px;"
            >
              <div class="grid-10col">
                {#each Array.from({length: 37}, (_, i) => i + 1) as dia_plazo}
                  <button 
                    type="button" 
                    class="grid-btn" 
                    class:selected={plazo === dia_plazo} 
                    on:click={() => seleccionarPlazoBase(dia_plazo)}
                  >
                    {dia_plazo}
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>

      <div class="form-group">
        <label for="plazoAdicional">
          Plazo Adicional (días)
          <span class="hint">Días extra sobre el plazo original</span>
        </label>
        <div class="dropdown-wrapper">
          <button 
            type="button" 
            class="dropdown-trigger" 
            on:click={toggleDropdownPlazoAdicional}
            bind:this={dropdownButtonAdicional}
            disabled={diasDisponiblesPlazoAdicional.length === 0}
          >
            {plazoAdicional || 'Seleccionar días'}
            <span class="arrow">{dropdownPlazoAdicionalAbierto ? '▲' : '▼'}</span>
          </button>
          {#if dropdownPlazoAdicionalAbierto && diasDisponiblesPlazoAdicional.length > 0}
            <div 
              class="dropdown-panel"
              style="top: {dropdownTopAdicional}px; left: {dropdownLeftAdicional}px;"
            >
              <div class="grid-10col">
                <button 
                  type="button" 
                  class="grid-btn btn-eliminar" 
                  class:selected={plazoAdicional === 0}
                  on:click={() => seleccionarPlazo(0)}
                  title="Eliminar plazo adicional"
                >
                  0
                </button>
                {#each diasDisponiblesPlazoAdicional as dia_plazo}
                  <button 
                    type="button" 
                    class="grid-btn" 
                    class:selected={plazoAdicional === dia_plazo} 
                    on:click={() => seleccionarPlazo(dia_plazo)}
                  >
                    {dia_plazo}
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>

      {#if mensaje}
        <p class="mensaje" class:error={mensaje.includes('❌')}>{mensaje}</p>
      {/if}
    </div>

    <div class="footer">
      <button class="btn-secundario" on:click={cerrar} disabled={guardando}>
        Cancelar
      </button>
      <button class="btn-primario" on:click={guardar} disabled={guardando}>
        {guardando ? 'Guardando...' : 'Guardar'}
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
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal {
    background: #1a2332;
    border-radius: 12px;
    width: 90%;
    max-width: 600px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #2d3e50;
  }

  .header h2 {
    color: #a8c5e0;
    margin: 0;
    font-size: 1.5rem;
  }

  .btn-cerrar {
    background: transparent;
    border: none;
    color: #8b9eb3;
    font-size: 1.5rem;
    cursor: pointer;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .btn-cerrar:hover {
    background: #2d3e50;
    color: #e0e6ed;
  }

  .content {
    padding: 1.5rem;
    overflow-y: auto;
    overflow-x: visible;
    flex: 1;
  }

  .info-req {
    background: #0f1419;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1.5rem;
    border: 1px solid #2d3e50;
  }

  .info-req p {
    color: #a8c5e0;
    margin: 0.5rem 0;
    font-size: 0.9rem;
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  .form-group label {
    display: block;
    color: #a8c5e0;
    font-weight: 500;
    margin-bottom: 0.5rem;
    font-size: 0.95rem;
  }

  .hint {
    font-size: 0.8rem;
    color: #8b9eb3;
    font-weight: 400;
    margin-left: 0.5rem;
  }

  textarea, input, select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    background: #0f1419;
    color: #e0e6ed;
    font-family: 'Inter', sans-serif;
    font-size: 0.95rem;
    transition: border-color 0.2s;
  }

  textarea {
    resize: vertical;
    min-height: 60px;
  }

  textarea:focus, input:focus, select:focus {
    outline: none;
    border-color: #5a8fc4;
  }

  .fecha-selectores {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr;
    gap: 0.5rem;
  }

  .fecha-grid {
    display: grid;
    grid-template-columns: 1fr 1.5fr 1fr;
    gap: 0.5rem;
  }

  .dropdown-trigger-fecha {
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
    font-size: 0.95rem;
  }

  .dropdown-trigger-fecha:hover {
    border-color: #5a8fc4;
  }

  .dropdown-panel {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 0.25rem;
    background: #1a2332;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    padding: 0.3rem;
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  }
  
  .fecha-grid .dropdown-panel {
    width: auto !important;
    max-width: none !important;
  }

  .dropdown-panel.dropdown-up {
    top: auto;
    bottom: calc(100% + 4px);
    margin-top: 0;
  }

  .dropdown-panel.dropdown-dias {
    right: 0;
    left: auto;
  }

  .grid-1col {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.25rem;
  }

  .grid-3col {
    display: grid;
    grid-template-columns: repeat(3, auto);
    gap: 4px;
    justify-content: start;
  }

  .grid-4col {
    display: grid;
    grid-template-columns: repeat(4, auto);
    gap: 4px;
    justify-content: start;
  }

  .grid-5col {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
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
    color: #a8c5e0;
    cursor: pointer;
    border-radius: 4px;
    font-family: 'Inter', sans-serif;
    font-size: 0.85rem;
    transition: all 0.2s;
    white-space: nowrap;
  }

  /* Estilos específicos para botones de año/mes */
  .grid-3col .grid-btn,
  .grid-4col .grid-btn {
    padding: 0.5rem 0.65rem;
    min-width: 50px;
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

  .grid-btn.disabled,
  .grid-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    background: #0a0f14;
    color: #4a5460;
  }

  .grid-btn.disabled:hover,
  .grid-btn:disabled:hover {
    background: #0a0f14;
    border-color: #2d3e50;
  }

  .arrow {
    color: #a8c5e0;
    font-size: 0.7rem;
  }

  .mensaje {
    margin-top: 1rem;
    padding: 0.75rem;
    border-radius: 6px;
    background: #1e4d2b;
    color: #7dd695;
    font-size: 0.9rem;
  }

  .mensaje.error {
    background: #5c1f1f;
    color: #ff8080;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 1px solid #2d3e50;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
    font-size: 0.95rem;
  }

  .btn-primario {
    background: linear-gradient(135deg, #5a8fc4 0%, #4a7ba7 100%);
    color: white;
  }

  .btn-primario:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(90, 143, 196, 0.4);
  }

  .btn-primario:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-secundario {
    background: #2d3e50;
    color: #a8c5e0;
  }

  .btn-secundario:hover:not(:disabled) {
    background: #3d4e60;
  }

  .btn-secundario:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .dropdown-wrapper {
    position: relative;
    width: 100%;
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

  .grid-10col {
    display: grid;
    grid-template-columns: repeat(10, 32px);
    gap: 0.25rem;
    box-sizing: border-box;
  }

  .grid-btn {
    padding: 0.4rem 0.3rem;
    border: 1px solid #2d3e50;
    background: #0f1419;
    color: #a8c5e0;
    cursor: pointer;
    border-radius: 3px;
    font-family: 'Inter', sans-serif;
    font-size: 0.85rem;
    transition: all 0.2s;
    text-align: center;
    line-height: 1;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
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

  .btn-eliminar {
    background: #2d1f1f !important;
    color: #ff8080 !important;
    border-color: #5c3030 !important;
  }

  .btn-eliminar:hover {
    background: #3d2525 !important;
    border-color: #ff6b6b !important;
  }

  .btn-eliminar.selected {
    background: #5c1f1f !important;
    color: #ffa0a0 !important;
    border-color: #ff6b6b !important;
  }
</style>
