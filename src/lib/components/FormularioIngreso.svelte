<script>
  import { onMount } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { jardines, cargarJardines } from '$lib/stores/catalogos';
  import { configuracion } from '$lib/stores/configuracion';
  import { calcularPrecioTotal, calcularLineaRequerimiento, formatearNumero, calcularFechaLimite, calcularPlazoTotal } from '$lib/utils/calculos.js';
  import { validarRequerimiento } from '$lib/utils/validaciones.js';
  import SelectRecinto from './SelectRecinto.svelte';
  import SelectPartida from './SelectPartida.svelte';
  import SelectorPlazo from './SelectorPlazo.svelte';
  import SelectorFecha from './SelectorFecha.svelte';

  let jardinSeleccionado = '';
  let mensaje = '';

  let formData = {
    jardinCodigo: '',
    recinto: '',
    item: '',
    partida: '',
    unidad: '',
    cantidad: null,
    plazoDias: null,
    plazoObservacion: 0,
    descripcion: '',
    observaciones: '',
    precioUnitario: 0,
    precioTotal: 0,
    fechaInicio: ''
  };

  let errores = {};
  let cargando = false;
  let formKey = 0;

  onMount(async () => {
    await cargarJardines();
  });

  function seleccionarJardin() {
    if (jardinSeleccionado) {
      const jardin = $jardines.find(j => j.codigo === jardinSeleccionado);
      formData.jardinCodigo = jardin.codigo;
    } else {
      formData.jardinCodigo = '';
    }
  }

  // Normalizar entrada de cantidad (máximo 2 decimales)
  let cantidadDisplay = '';
  
  function handleCantidadInput(e) {
    let valor = e.target.value;
    
    // Permitir solo números, coma y punto
    valor = valor.replace(/[^0-9,\.]/g, '');
    
    // Reemplazar punto por coma para normalización
    valor = valor.replace(/\./g, ',');
    
    // Limitar a máximo 2 decimales
    const partes = valor.split(',');
    if (partes.length > 2) {
      // Si hay más de una coma, mantener solo la primera
      valor = partes[0] + ',' + partes.slice(1).join('');
      partes.length = 2;
      partes[1] = valor.split(',')[1];
    }
    if (partes.length > 1 && partes[1].length > 2) {
      // Limitar decimales a 2
      valor = partes[0] + ',' + partes[1].substring(0, 2);
    }
    
    cantidadDisplay = valor;
    
    // Convertir y redondear a 2 decimales, o null si vacío
    if (!valor || valor === '') {
      formData.cantidad = null;
    } else {
      const num = parseFloat(valor.replace(',', '.'));
      formData.cantidad = isNaN(num) ? null : Math.round(num * 100) / 100;
    }
  }

  // Calcular precio total solo si hay cantidad válida
  $: formData.precioTotal = (formData.cantidad && formData.cantidad > 0) 
    ? calcularPrecioTotal(formData.cantidad, formData.precioUnitario || 0) 
    : 0;

  // Calcular valor final (con sobre_costo, utilidades e IVA)
  $: valorFinal = (() => {
    if (formData.precioTotal > 0 && jardinSeleccionado) {
      const jardin = $jardines.find(j => j.codigo === jardinSeleccionado);
      const porcentajeSobreCosto = jardin?.sobreCosto ? jardin.sobreCosto / 100 : 0;
      
      const { totalLinea } = calcularLineaRequerimiento(
        formData.precioTotal, 
        $configuracion.porcentajeUtilidades || 0.25,
        porcentajeSobreCosto
      );
      return totalLinea;
    }
    return 0;
  })();

  $: fechaLimite = (() => {
    if (!formData.fechaInicio || !formData.plazoDias) return '-';
    const plazoTotal = calcularPlazoTotal(formData.plazoDias, formData.plazoObservacion);
    const fecha = calcularFechaLimite(formData.fechaInicio, plazoTotal);
    if (!fecha) return '-';
    const [año, mes, dia] = fecha.split('-').map(Number);
    return new Date(año, mes - 1, dia).toLocaleDateString('es-CL');
  })();

  async function handleSubmit() {
    errores = validarRequerimiento(formData) || {};
    
    if (Object.keys(errores).length > 0) {
      const camposFaltantes = Object.values(errores).join(', ');
      mensaje = `⚠️ Campos obligatorios: ${camposFaltantes}`;
      return;
    }

    cargando = true;
    mensaje = '';
    try {
      await db.requerimientos.add({
        jardinCodigo: formData.jardinCodigo,
        recinto: formData.recinto,
        partidaItem: formData.item,
        cantidad: formData.cantidad,
        precioUnitario: formData.precioUnitario,
        fechaInicio: formData.fechaInicio,
        fechaRegistro: new Date().toISOString().split('T')[0],
        plazoDias: formData.plazoDias,
        descripcion: formData.descripcion || null
      });
      
      resetForm();
      mensaje = '✅ Requerimiento registrado exitosamente';
      setTimeout(() => mensaje = '', 3000);
    } catch (error) {
      console.error('Error guardando:', error);
      mensaje = '❌ Error al guardar: ' + (error.message || error);
    } finally {
      cargando = false;
    }
  }

  function resetForm() {
    formData = {
      jardin_codigo: jardinSeleccionado ? formData.jardinCodigo : '',
      recinto: '',
      item: '',
      partida: '',
      unidad: '',
      cantidad: null,
      plazo: null,
      plazoAdicional: 0,
      descripcion: '',
      observaciones: '',
      precioUnitario: 0,
      precioTotal: 0,
      fechaInicio: ''
    };
    cantidadDisplay = '';
    errores = {};
    formKey++;
  }
</script>

<div class="container">
  <h2>Ingresar Requerimiento</h2>
  
  <div class="selector-jardin">
    <div class="form-group">
      <label for="jardin">Jardín <span class="requerido">*</span></label>
      <select id="jardin" bind:value={jardinSeleccionado} on:change={seleccionarJardin}>
        <option value="">Seleccionar jardín...</option>
        {#each $jardines as jardin}
          <option value={jardin.codigo}>{jardin.codigo} - {jardin.nombre}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if !jardinSeleccionado}
    <p class="placeholder">Selecciona un jardín para ingresar requerimientos</p>
  {:else}
    <form on:submit|preventDefault={handleSubmit} class="formulario">
  
  {#key formKey}
  <SelectRecinto jardinCodigo={formData.jardinCodigo} bind:value={formData.recinto} error={errores.recinto} />
  
  <SelectPartida bind:value={formData.item} bind:nombre={formData.partida} 
                 bind:unidad={formData.unidad} bind:precioUnitario={formData.precioUnitario} error={errores.partida} />

  <div class="form-group">
    <label for="descripcion">Descripción</label>
    <textarea id="descripcion" bind:value={formData.descripcion} rows="3"></textarea>
    {#if errores.descripcion}<span class="error">{errores.descripcion}</span>{/if}
  </div>

  <div class="form-group">
    <label for="cantidad">Cantidad <span class="requerido">*</span></label>
    <input 
      type="text" 
      id="cantidad" 
      value={cantidadDisplay}
      on:input={handleCantidadInput}
      placeholder="Ej: 1 o 1,50"
    />
    {#if errores.cantidad}<span class="error">{errores.cantidad}</span>{/if}
  </div>

  <SelectorFecha bind:value={formData.fechaInicio} error={errores.fechaInicio} />

  <SelectorPlazo bind:value={formData.plazoDias} error={errores.plazo} />
  {/key}

  {#if formData.cantidad > 0 && formData.precioUnitario > 0}
  <div class="form-group">
    <span class="info-label" title="Incluye sobre costo, utilidades e IVA">Valor Total: <strong>${formatearNumero(valorFinal)}</strong></span>
  </div>
  {/if}

  <div class="form-group">
    <span class="info-label">Fecha Límite: <strong>{fechaLimite}</strong></span>
  </div>

  {#if mensaje}
    <div class="mensaje {mensaje.includes('✅') ? 'exito' : 'error'}">{mensaje}</div>
  {/if}

  <button type="submit" disabled={cargando}>
    {cargando ? 'Guardando...' : 'Registrar Requerimiento'}
  </button>
</form>
  {/if}
</div>

<style>
  .container { padding: 0 2rem 2rem 2rem; }
  h2 { color: #a8c5e0; margin: 0 0 1.5rem 0; text-align: center; }
  .selector-jardin { max-width: 400px; margin: 0 auto 1.5rem auto; }
  .placeholder { text-align: center; color: #6b7d8f; padding: 4rem 2rem; background: #1a2332; border: 2px dashed #2d3e50; border-radius: 8px; font-size: 1.05rem; max-width: 400px; margin: 0 auto; }
  .formulario { max-width: 400px; margin: 0 auto; }
  .form-group { margin-bottom: 1.25rem; display: flex; flex-direction: column; }
  label { display: block; margin-bottom: 0.5rem; font-weight: 500; color: #a8c5e0; font-size: 0.9rem; }
  .requerido { color: #8b9eb3; margin-left: 0.25rem; font-size: 0.85em; opacity: 0.7; }
  .info-label { display: block; margin-bottom: 0.5rem; font-weight: 500; color: #a8c5e0; font-size: 0.9rem; }
  input, select, textarea { width: 100%; padding: 0.65rem; border: 1px solid #2d3e50; border-radius: 6px; background: #1a2332 !important; color: #e0e6ed !important; font-family: 'Inter', sans-serif; transition: border-color 0.2s; -webkit-appearance: none; appearance: none; }
  input:focus, select:focus, textarea:focus { outline: none; border-color: #5a8fc4; }
  input:disabled, select:disabled { opacity: 0.5; cursor: not-allowed; }
  .error { color: #ff6b6b; font-size: 0.825rem; margin-top: 0.25rem; display: block; }
  .mensaje { padding: 1rem; border-radius: 6px; margin-bottom: 1rem; font-weight: 500; }
  .mensaje.exito { background: rgba(76, 175, 80, 0.2); color: #81c784; border: 1px solid #4caf50; }
  .mensaje.error { background: rgba(244, 67, 54, 0.2); color: #e57373; border: 1px solid #f44336; }
  .mensaje:not(.exito):not(.error) { background: rgba(255, 193, 7, 0.15); color: #ffd54f; border: 1px solid rgba(255, 193, 7, 0.3); }
  button { padding: 0.75rem 1.5rem; background: linear-gradient(135deg, #5a8fc4 0%, #4a7ba7 100%); color: #fff; border: none; border-radius: 6px; cursor: pointer; font-family: 'Inter', sans-serif; font-weight: 600; transition: all 0.2s; }
  button:hover { background: linear-gradient(135deg, #6a9fd4 0%, #5a8bb7 100%); transform: translateY(-1px); }
  button:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }
  strong { color: #a8c5e0; }
</style>
