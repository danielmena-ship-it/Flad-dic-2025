<script>
  import { onMount } from 'svelte';
  import { partidas, cargarPartidas } from '$lib/stores/catalogos.js';

  export let value = '';
  export let nombre = '';
  export let unidad = '';
  export let precioUnitario = 0;
  export let error = '';

  onMount(async () => {
    await cargarPartidas();
  });

  function handleChange(event) {
    const selected = $partidas.find(p => p.item === event.target.value);
    if (selected) {
      nombre = selected.partida;
      unidad = selected.unidad || '';
      precioUnitario = selected.precioUnitario || 0;
    }
  }
</script>

<div class="form-group">
  <label for="partida">Partida <span class="requerido">*</span></label>
  <select id="partida" bind:value={value} on:change={handleChange}>
    <option value="">Seleccione una partida</option>
    {#each $partidas as partida}
      {@const esEncabezado = /^[A-Z]+$/.test(partida.item)}
      <option 
        value={partida.item} 
        disabled={esEncabezado}
        class:seccion={esEncabezado}
      >
        {partida.item} - {partida.partida}
      </option>
    {/each}
  </select>
  {#if error}<span class="error">{error}</span>{/if}
</div>

<style>
  .form-group {
    display: flex;
    flex-direction: column;
  }
  label {
    color: #a8c5e0;
    font-weight: 500;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
  }
  select {
    width: 100%;
    padding: 0.65rem;
    border: 1px solid #2d3e50;
    border-radius: 6px;
    background: #1a2332 !important;
    color: #e0e6ed !important;
    font-family: 'Inter', sans-serif;
    transition: border-color 0.2s;
    -webkit-appearance: none;
    appearance: none;
  }
  select:focus {
    outline: none;
    border-color: #5a8fc4;
  }
  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  option.seccion {
    color: #999;
    font-weight: 700;
    font-style: italic;
    background-color: #2a2a2a;
  }
  option:disabled {
    cursor: not-allowed;
  }
  .error {
    color: #ff6b6b;
    font-size: 0.825rem;
    margin-top: 0.25rem;
  }
  .requerido {
    color: #8b9eb3;
    margin-left: 0.25rem;
    font-size: 0.85em;
    opacity: 0.7;
  }
</style>
