<script>
  import { onMount } from 'svelte';
  
  export let value = '';
  export let error = '';

  let año = '';
  let mes = '';
  let dia = '';
  let inicializado = false;
  let dropdownAbierto = null; // 'año', 'mes', 'dia', o null

  const fechaHoy = new Date();
  const añoActual = fechaHoy.getFullYear();
  const mesActual = fechaHoy.getMonth() + 1; // Los meses en JS van de 0-11
  const diaActual = fechaHoy.getDate();
  
  // Rango desde 2026 hasta 5 años hacia adelante desde hoy
  const añoInicio = 2026;
  const añoFin = añoActual + 5;
  const años = Array.from({ length: añoFin - añoInicio + 1 }, (_, i) => añoInicio + i);
  const meses = [
    { num: '01', nombre: 'Ene' }, { num: '02', nombre: 'Feb' }, { num: '03', nombre: 'Mar' },
    { num: '04', nombre: 'Abr' }, { num: '05', nombre: 'May' }, { num: '06', nombre: 'Jun' },
    { num: '07', nombre: 'Jul' }, { num: '08', nombre: 'Ago' }, { num: '09', nombre: 'Sep' },
    { num: '10', nombre: 'Oct' }, { num: '11', nombre: 'Nov' }, { num: '12', nombre: 'Dic' }
  ];

  onMount(() => {
    if (value && value.includes('-')) {
      const partes = value.split('-');
      año = partes[0];
      mes = partes[1];
      dia = partes[2];
    }
    inicializado = true;
  });

  $: diasEnMes = mes && año ? new Date(parseInt(año), parseInt(mes), 0).getDate() : 31;
  $: dias = Array.from({ length: diasEnMes }, (_, i) => i + 1);
  
  // Permitir todos los meses y días sin restricción
  $: mesesDisponibles = meses;
  $: diasDisponibles = dias;

  // Actualizar el valor cuando cambien año, mes o día
  $: {
    if (inicializado && año && mes && dia) {
      value = `${año}-${mes}-${String(dia).padStart(2, '0')}`;
    }
  }

  function toggleDropdown(tipo) {
    dropdownAbierto = dropdownAbierto === tipo ? null : tipo;
  }

  function seleccionarAño(a) {
    año = String(a);
    dropdownAbierto = null;
  }

  function seleccionarMes(m) {
    mes = m;
    dropdownAbierto = null;
  }

  function seleccionarDia(d) {
    dia = String(d);
    dropdownAbierto = null;
  }

  function getMesNombre(num) {
    return meses.find(m => m.num === num)?.nombre || '';
  }
</script>

<div class="form-group">
  <label for="fecha-año">Fecha Inicio <span class="requerido">*</span></label>
  <div class="fecha-grid">
    <!-- Año -->
    <div class="dropdown-wrapper">
      <button type="button" id="fecha-año" class="dropdown-trigger" on:click={() => toggleDropdown('año')}>
        {año || 'Año'}
        <span class="arrow">{dropdownAbierto === 'año' ? '▲' : '▼'}</span>
      </button>
      {#if dropdownAbierto === 'año'}
        <div class="dropdown-panel">
          <div class="grid-1col">
            {#each años as a}
              <button type="button" class="grid-btn" class:selected={año === String(a)} on:click={() => seleccionarAño(a)}>
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
        {getMesNombre(mes) || 'Mes'}
        <span class="arrow">{dropdownAbierto === 'mes' ? '▲' : '▼'}</span>
      </button>
      {#if dropdownAbierto === 'mes'}
        <div class="dropdown-panel">
          <div class="grid-3col">
            {#each mesesDisponibles as m}
              <button type="button" class="grid-btn" class:selected={mes === m.num} on:click={() => seleccionarMes(m.num)}>
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
        {dia || 'Día'}
        <span class="arrow">{dropdownAbierto === 'dia' ? '▲' : '▼'}</span>
      </button>
      {#if dropdownAbierto === 'dia'}
        <div class="dropdown-panel">
          <div class="grid-10col">
            {#each diasDisponibles as d}
              <button type="button" class="grid-btn" class:selected={dia === String(d)} on:click={() => seleccionarDia(d)}>
                {d}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
  {#if error}<span class="error">{error}</span>{/if}
</div>

<style>
  .form-group { margin-bottom: 1.25rem; }
  label { display: block; margin-bottom: 0.5rem; color: #a8c5e0; font-size: 0.9rem; font-weight: 500; }
  
  .fecha-grid {
    display: grid;
    grid-template-columns: 1fr 1.5fr 1fr;
    gap: 0.5rem;
  }

  .dropdown-wrapper { position: relative; }
  
  .dropdown-trigger {
    width: 100%;
    padding: 0.65rem;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    background: #1a2332;
    color: #e0e6ed;
    font-family: 'Inter', sans-serif;
    cursor: pointer;
    text-align: left;
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: border-color 0.2s;
  }
  
  .dropdown-trigger:hover { border-color: #5a8fc4; }
  .arrow { color: #a8c5e0; font-size: 0.7rem; }
  
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
  }
  
  .grid-1col { display: grid; grid-template-columns: 1fr; gap: 0.25rem; }
  .grid-3col { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.25rem; }
  .grid-10col { display: grid; grid-template-columns: repeat(10, 1fr); gap: 0.25rem; }
  
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
  
  .grid-btn:hover { background: #202b38; border-color: #5a8fc4; }
  .grid-btn.selected {
    background: linear-gradient(135deg, #5a8fc4 0%, #4a7ba7 100%);
    color: #fff;
    border-color: #5a8fc4;
  }
  
  .error { color: #ff6b6b; font-size: 0.825rem; margin-top: 0.25rem; display: block; }
  .requerido { color: #8b9eb3; margin-left: 0.25rem; font-size: 0.85em; opacity: 0.7; }
</style>
