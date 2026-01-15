<script>
  export let value = null;
  export let error = '';

  let abierto = false;
  const filas = 4;
  const columnas = 10;

  function seleccionar(plazo) {
    value = plazo;
    abierto = false;
  }

  function toggleDropdown() {
    abierto = !abierto;
  }
</script>

<div class="form-group">
  <label for="plazo-selector">Plazo (días) <span class="requerido">*</span></label>
  <div class="dropdown-wrapper">
    <button type="button" id="plazo-selector" class="dropdown-trigger" on:click={toggleDropdown}>
      {value || 'Seleccione plazo'}
      <span class="arrow">{abierto ? '▲' : '▼'}</span>
    </button>
    
    {#if abierto}
      <div class="dropdown-panel">
        <div class="plazo-grid">
          {#each Array(filas) as _, fila}
            <div class="fila">
              {#each Array(columnas) as _, col}
                {@const plazo = fila * columnas + col + 1}
                {#if plazo <= 37}
                  <button
                    type="button"
                    class="plazo-btn"
                    class:selected={value === plazo}
                    on:click={() => seleccionar(plazo)}
                  >
                    {plazo}
                  </button>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
  {#if error}<span class="error">{error}</span>{/if}
</div>

<style>
  .form-group { margin-bottom: 1.25rem; position: relative; }
  label { display: block; margin-bottom: 0.5rem; color: #a8c5e0; font-size: 0.9rem; font-weight: 500; }
  
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
    padding: 0.4rem;
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    width: fit-content;
    box-sizing: border-box;
  }
  
  .plazo-grid { 
    display: grid; 
    grid-template-columns: repeat(10, 32px); 
    gap: 0.25rem;
  }
  
  .fila { display: contents; }
  
  .plazo-btn {
    padding: 0.5rem;
    border: 1px solid #2d3e50;
    background: #0f1419;
    color: #a8c5e0;
    cursor: pointer;
    border-radius: 4px;
    font-family: 'Inter', sans-serif;
    font-size: 0.85rem;
    transition: all 0.2s;
  }
  
  .plazo-btn:hover { background: #202b38; border-color: #5a8fc4; }
  .plazo-btn.selected {
    background: linear-gradient(135deg, #5a8fc4 0%, #4a7ba7 100%);
    color: #fff;
    border-color: #5a8fc4;
  }
  .requerido { color: #8b9eb3; margin-left: 0.25rem; font-size: 0.85em; opacity: 0.7; }
  
  .error { color: #ff6b6b; font-size: 0.825rem; margin-top: 0.25rem; display: block; }
</style>
