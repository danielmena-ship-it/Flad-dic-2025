<script>
  import { cargarRecintos } from '$lib/stores/catalogos.js';

  export let jardinCodigo = '';
  export let value = '';
  export let error = '';

  let recintos = [];

  $: if (jardinCodigo) {
    cargarRecintosAsync();
  } else {
    recintos = [];
    value = '';
  }

  async function cargarRecintosAsync() {
    recintos = await cargarRecintos(jardinCodigo);
  }
</script>

<div class="form-group">
  <label for="recinto">Zona <span class="requerido">*</span></label>
  <select id="recinto" bind:value={value} disabled={!jardinCodigo}>
    <option value="">Seleccione una zona</option>
    {#each recintos as recinto}
      <option value={recinto.nombre}>{recinto.nombre}</option>
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
