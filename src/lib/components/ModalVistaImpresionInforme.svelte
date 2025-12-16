<script>
  import { createEventDispatcher, onMount } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { configuracion } from '$lib/stores/configuracion';
  import { formatearNumero } from '$lib/utils/calculos.js';
  import { formatearFecha } from '$lib/utils/formatoFecha.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';
  import html2pdf from 'html2pdf.js';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';

  export let informe;
  export let requerimientos;

  const dispatch = createEventDispatcher();

  let jardinCompleto = null;
  let cargando = true;
  let mensajeGuardado = '';
  let requerimientosEnriquecidos = [];

  // ✅ Recargar datos cada vez que se abre el modal
  $: if (informe) {
    cargarDatos();
  }

  async function cargarDatos() {
    cargando = true;
    const jardines = await db.jardines.getAll();
    jardinCompleto = jardines.find(j => j.codigo === informe.jardinCodigo);
    requerimientosEnriquecidos = await enriquecerRequerimientos(requerimientos);
    cargando = false;
  }

  onMount(async () => {
    await cargarDatos();
  });

  function extraerNumeroZona(zona) {
    const match = zona.match(/^(\d+)/);
    return match ? match[1] : zona;
  }

  async function generarPDF() {
    const elemento = document.querySelector('.contenido-imprimible');
    
    const opciones = {
      margin: [5, 8],
      image: { type: 'jpeg', quality: 0.98 },
      html2canvas: { 
        scale: 2,
        useCORS: true,
        letterRendering: true
      },
      jsPDF: { 
        unit: 'mm', 
        format: 'legal', 
        orientation: 'landscape' 
      },
      pagebreak: { 
        mode: ['css', 'legacy']
      }
    };

    const pdfArrayBuffer = await html2pdf().set(opciones).from(elemento).output('arraybuffer');
    
    const filePath = await save({
      defaultPath: `${informe.codigo}.pdf`,
      filters: [{
        name: 'PDF',
        extensions: ['pdf']
      }]
    });

    if (filePath) {
      const pdfBytes = new Uint8Array(pdfArrayBuffer);
      await writeFile(filePath, pdfBytes);
      mensajeGuardado = '✅ PDF guardado exitosamente';
      setTimeout(() => mensajeGuardado = '', 3000);
    }
  }

  function cerrar() {
    dispatch('cerrar');
  }
</script>

<div class="overlay" on:click={cerrar} on:keydown={(e) => e.key === 'Escape' && cerrar()} role="button" tabindex="0" aria-label="Cerrar modal">
  <div class="modal-impresion" on:click|stopPropagation role="dialog" aria-modal="true">
    
    {#if cargando}
      <div class="cargando">
        <p>Cargando datos...</p>
      </div>
    {:else}
      <div class="acciones-modal no-print">
        <button class="btn-cerrar" on:click={cerrar}>✕ Cerrar</button>
        <button class="btn-pdf" on:click={generarPDF}>📄 Descargar PDF</button>
      </div>

      <div class="contenido-imprimible">
        
        <div class="encabezado">
          <h1>INFORME DE PAGO</h1>
          <div class="subtitulo">Servicio de Mantención y Reparación de Dependencias</div>
          <div class="subtitulo">JUNJI MAG</div>
          <div class="codigo-informe">{informe.codigo}</div>
        </div>

        <div class="seccion-info">
          <div class="info-row">
            <span class="label">Línea de Contratación:</span>
            <span class="valor">Linea 1</span>
          </div>
          <div class="info-row">
            <span class="label">Jardín:</span>
            <span class="valor">{jardinCompleto?.nombre || 'N/A'}</span>
          </div>
          <div class="info-row">
            <span class="label">Fecha Creación:</span>
            <span class="valor">{formatearFecha(informe.fechaCreacion)}</span>
          </div>
          <div class="info-row">
            <span class="label">Contratista:</span>
            <span class="valor">{$configuracion.contratista || 'Sin especificar'}</span>
          </div>
          <div class="info-row">
            <span class="label">ITO:</span>
            <span class="valor">{$configuracion.itoNombre || 'Sin especificar'}</span>
          </div>
        </div>

        <div class="seccion-requerimientos">
          <h2>Requerimientos a Pago</h2>
          <p class="nota-texto">Mediante el presente solicito proceder al pago, al contratista, de los siguientes servicios con recepción conforme:</p>
          
          {#if requerimientos.length === 0}
            <p class="sin-datos">No hay requerimientos asignados</p>
          {:else}
            <div class="tabla-requerimientos-wrapper">
              <table class="tabla-requerimientos">
                <thead>
                  <tr>
                    <th class="centrado" style="width: 5%;">Zona</th>
                    <th class="izquierda" style="width: 20%;">Item - Partida</th>
                    <th class="derecha" style="width: 9%;">Cant.- Un.</th>
                    <th class="derecha" style="width: 8%;">P. Unit.</th>
                    <th class="derecha" style="width: 9%;">P. Total</th>
                    <th class="centrado" style="width: 8%;">F. Inicio</th>
                    <th class="centrado" style="width: 6%;">T. Plazo</th>
                    <th class="centrado" style="width: 8%;">F. Límite</th>
                    <th class="centrado" style="width: 9%;">F. Recepción</th>
                    <th class="derecha" style="width: 6%;">D. Atraso</th>
                    <th class="derecha" style="width: 9%;">Multa</th>
                    <th class="derecha" style="width: 9%;">A Pago</th>
                  </tr>
                </thead>
                <tbody>
                  {#each requerimientosEnriquecidos as req}
                    <tr>
                      <td class="centrado">{extraerNumeroZona(req.recinto || '-')}</td>
                      <td class="izquierda">{req.partidaItem} - {req.partidaNombre}</td>
                      <td class="derecha">{formatearNumero(req.cantidad)} {req.partidaUnidad}</td>
                      <td class="derecha">${formatearNumero(req.precioUnitario)}</td>
                      <td class="derecha">${formatearNumero(req.precioTotal)}</td>
                      <td class="centrado">{formatearFecha(req.fechaInicio)}</td>
                      <td class="centrado">{req.plazoTotal || '-'}</td>
                      <td class="centrado">{formatearFecha(req.fechaLimite)}</td>
                      <td class="centrado">{formatearFecha(req.fechaRecepcion)}</td>
                      <td class="derecha">{Math.max(0, req.diasAtraso || 0)}</td>
                      <td class="derecha">${formatearNumero(req.multa || 0)}</td>
                      <td class="derecha destacado">${formatearNumero(req.aPago || 0)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>

        <div class="seccion-total">
          <div class="total-row">
            <span class="total-label">Neto:</span>
            <span class="total-valor">${formatearNumero(informe.neto || 0)}</span>
          </div>
          <div class="total-row">
            <span class="total-label">Utilidades ({Math.round(($configuracion?.porcentajeUtilidades || 0.25) * 100)}%):</span>
            <span class="total-valor">${formatearNumero(informe.utilidades || 0)}</span>
          </div>
          <div class="total-row">
            <span class="total-label">IVA (19%):</span>
            <span class="total-valor">${formatearNumero(informe.iva || 0)}</span>
          </div>
          <div class="total-row total-final">
            <span class="total-label">Total:</span>
            <span class="total-valor">${formatearNumero(informe.totalPagar || 0)}</span>
          </div>
        </div>

        <div class="seccion-firma">
          <div class="firma-box">
            {#if $configuracion.firmaBase64}
              <img src={`data:image/png;base64,${$configuracion.firmaBase64}`} alt="Firma ITO" class="firma-imagen" />
            {:else}
              <div class="firma-placeholder">Sin firma</div>
            {/if}
            <p class="firma-label">Firma ITO</p>
            <p class="firma-nombre">{$configuracion.itoNombre || 'Sin especificar'}</p>
          </div>
        </div>

        <div class="pie-pagina">
          <p>Documento generado el {formatearFecha(new Date())}</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    z-index: 2000;
    overflow-y: auto;
    padding: 2rem 1rem;
  }

  .modal-impresion {
    background: #ffffff;
    border-radius: 12px;
    max-width: 900px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.7);
    position: relative;
  }

  .cargando {
    padding: 3rem;
    text-align: center;
    color: #333;
  }

  .acciones-modal {
    position: sticky;
    top: 0;
    background: #f5f5f5;
    border-radius: 12px 12px 0 0;
    padding: 1rem 1.5rem;
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    border-bottom: 2px solid #ddd;
    z-index: 10;
  }

  .acciones-modal button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
  }

  .btn-cerrar { background: #666; color: white; }
  .btn-cerrar:hover { background: #555; }
  .btn-pdf { background: #4caf50; color: white; }
  .btn-pdf:hover { background: #45a049; }
  .btn-imprimir { background: #666; color: white; }
  .btn-imprimir:hover { background: #555; }

  .contenido-imprimible { padding: 2rem; }

  .encabezado {
    text-align: center;
    margin-bottom: 2rem;
    border-bottom: 3px solid #5a8fc4;
    padding-bottom: 1.5rem;
  }

  .encabezado h1 {
    color: #2c3e50;
    font-size: 2rem;
    margin: 0 0 0.5rem 0;
    font-weight: 700;
  }

  .subtitulo {
    color: #000;
    font-size: 1rem;
    margin: 0.25rem 0;
    font-weight: 400;
  }

  .codigo-informe {
    color: #5a8fc4;
    font-size: 1.5rem;
    font-weight: 600;
    margin-top: 0.5rem;
  }

  .seccion-info {
    background: #f5f5f5;
    padding: 0.75rem;
    border-radius: 8px;
    margin-bottom: 1.25rem;
    border: 1px solid #ddd;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    padding: 0.3rem 0;
    border-bottom: 1px solid #ddd;
  }

  .info-row:last-child { border-bottom: none; }
  .label { color: #000; font-weight: 600; font-size: 0.9rem; }
  .valor { color: #000; font-size: 0.9rem; font-weight: 500; }

  .seccion-requerimientos { 
    margin-bottom: 0.5rem; 
  }

  .seccion-requerimientos h2 {
    color: #000;
    font-size: 1.3rem;
    margin-bottom: 1.5rem;
    border-bottom: 2px solid #333;
    padding-bottom: 0.5rem;
  }

  .nota-texto {
    color: #000;
    font-size: 0.95rem;
    margin-bottom: 1rem;
    padding: 0.75rem;
    background: #f8f8f8;
    border-radius: 4px;
    line-height: 1.5;
    border: 1px solid #ddd;
  }

  .sin-datos {
    text-align: center;
    color: #666;
    padding: 2rem;
    font-style: italic;
  }

  .tabla-requerimientos-wrapper { 
    overflow-x: auto; 
    margin-top: 1rem;
  }

  .tabla-requerimientos {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
    page-break-inside: auto;
  }

  .tabla-requerimientos thead th {
    background: #2c3e50;
    color: white;
    padding: 0.75rem;
    font-weight: 600;
    border-bottom: 2px solid #5a8fc4;
    white-space: nowrap;
    border-left: none;
    border-right: none;
  }

  .tabla-requerimientos tbody td {
    padding: 0.75rem;
    color: #000;
    border-bottom: 1px solid #ddd;
    vertical-align: top;
    border-left: none;
    border-right: none;
  }

  .tabla-requerimientos tbody tr:hover { background: #f5f5f5; }
  .tabla-requerimientos tbody tr:last-child td { border-bottom: 2px solid #ccc; }

  .tabla-requerimientos .centrado { text-align: center; }
  .tabla-requerimientos .derecha { text-align: right; }
  .tabla-requerimientos .izquierda { text-align: left; }
  .tabla-requerimientos .destacado { color: #000 !important; font-weight: 600; }

  .descripcion-cell {
    max-width: 200px;
    word-wrap: break-word;
    white-space: normal;
    font-size: 0.85rem;
    line-height: 1.4;
  }

  .seccion-total {
    background: #e8f4f8;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: 2px solid #333;
    margin-bottom: 2rem;
  }

  .total-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.15rem 0;
  }

  .total-label { color: #000; font-weight: 700; font-size: 0.84rem; }
  .total-valor { color: #1a4d7a; font-weight: 700; font-size: 1.05rem; }

  .total-final {
    border-top: 2px solid #333;
    padding-top: 0.4rem !important;
    margin-top: 0.3rem;
  }

  .total-final .total-label { font-size: 0.91rem; }
  .total-final .total-valor { font-size: 1.12rem; color: #000; }

  .seccion-firma {
    margin: 3rem 0 0.25rem 0;
    display: flex;
    justify-content: flex-end;
  }

  .firma-box {
    width: 300px;
    text-align: center;
    position: relative;
    min-height: 80px;
  }

  .firma-imagen {
    position: absolute;
    bottom: 30px;
    left: 50%;
    transform: translateX(-50%);
    max-width: 260px;
    height: auto;
  }

  .firma-placeholder {
    position: absolute;
    bottom: 30px;
    left: 50%;
    transform: translateX(-50%);
    color: #999;
    font-style: italic;
  }

  .firma-linea {
    border-top: 2px solid #333;
    margin: 60px 0 0.5rem 0;
  }

  .firma-label,
  .firma-nombre {
    position: relative;
    z-index: 0;
  }

  .firma-label {
    color: #000;
    font-weight: 600;
    font-size: 0.9rem;
    margin: 0.25rem 0;
  }

  .firma-nombre {
    color: #000;
    font-size: 0.85rem;
    margin: 0.25rem 0;
  }

  .pie-pagina {
    text-align: center;
    padding-top: 0.5rem;
    border-top: 1px solid #ccc;
    color: #666;
    font-size: 0.9rem;
  }

  .pie-pagina p { margin: 0; }

  @page {
    size: legal landscape;
    margin: 0.5cm 0.8cm;
  }

  @media print {
    * {
      -webkit-print-color-adjust: exact !important;
      print-color-adjust: exact !important;
    }

    body * {
      visibility: hidden !important;
    }

    .overlay,
    .overlay *,
    .modal-impresion,
    .modal-impresion *,
    .contenido-imprimible,
    .contenido-imprimible * {
      visibility: visible !important;
    }

    .overlay { 
      position: absolute !important;
      left: 0 !important;
      top: 0 !important;
      background: white !important; 
      padding: 0 !important;
      margin: 0 !important;
      display: block !important;
      width: 100% !important;
    }
    
    .modal-impresion {
      position: static !important;
      box-shadow: none !important;
      background: white !important;
      max-width: 100% !important;
      width: 100% !important;
      max-height: none !important;
      overflow: visible !important;
      display: block !important;
      margin: 0 !important;
      padding: 0 !important;
    }

    .no-print { display: none !important; visibility: hidden !important; }
    
    .contenido-imprimible {
      padding: 0 !important;
      color: #000 !important;
      margin: 0 !important;
      width: 100% !important;
      max-width: 100% !important;
    }

    .encabezado {
      border-bottom: 1.5px solid #5a8fc4;
      margin-bottom: 0.01cm;
      padding: 0.01cm 0.04cm;
      page-break-after: avoid;
      background: #f8f8f8;
    }

    .encabezado h1 {
      font-size: 8pt;
      margin: 0;
      font-weight: 700;
      color: #2c3e50 !important;
      line-height: 0.95;
    }

    .subtitulo {
      font-size: 6pt;
      margin: 0;
      color: #000 !important;
      font-weight: 400;
      line-height: 0.95;
    }

    .codigo-informe {
      font-size: 7pt;
      font-weight: 600;
      color: #5a8fc4 !important;
      margin-top: 0.01cm;
      line-height: 0.95;
    }

    .seccion-info {
      background: #f5f5f5;
      border: none;
      padding: 0.01cm 0.04cm;
      margin-bottom: 0.01cm;
      page-break-inside: avoid;
      font-size: 6pt;
    }

    .info-row {
      display: flex;
      justify-content: space-between;
      padding: 0.008cm 0;
      border-bottom: 1px solid #999;
      font-size: 6pt;
      line-height: 0.95;
    }
    
    .info-row:last-child {
      border-bottom: none;
    }

    .label { color: #000 !important; font-weight: 600; }
    .valor { color: #000 !important; }

    .seccion-requerimientos { 
      margin-bottom: 0.02cm;
      page-break-inside: avoid;
      page-break-before: avoid !important;
    }

    .seccion-requerimientos h2 {
      font-size: 7pt;
      margin: 0 0 0.008cm 0;
      padding: 0.008cm 0.03cm;
      background: #e0e0e0;
      border-bottom: 2px solid #333;
      page-break-after: auto;
      color: #000 !important;
      font-weight: 700;
      line-height: 0.95;
    }

    .nota-texto {
      font-size: 5pt;
      margin: 0;
      padding: 0.008cm 0.015cm;
      background: #f5f5f5;
      color: #000 !important;
      line-height: 0.95;
      page-break-after: auto;
    }

    .tabla-requerimientos-wrapper {
      page-break-before: auto;
      page-break-inside: auto;
      width: 100%;
      margin-bottom: 0.1cm;
      margin-top: 0;
    }

    .tabla-requerimientos {
      width: 100%;
      border-collapse: collapse;
      font-size: 7pt;
      page-break-inside: auto;
      table-layout: fixed;
    }

    .tabla-requerimientos thead th:nth-child(1) { width: 3%; }
    .tabla-requerimientos thead th:nth-child(2) { width: 15%; }
    .tabla-requerimientos thead th:nth-child(3) { width: 10%; }
    .tabla-requerimientos thead th:nth-child(4) { width: 8%; }
    .tabla-requerimientos thead th:nth-child(5) { width: 8%; }
    .tabla-requerimientos thead th:nth-child(6) { width: 9%; }
    .tabla-requerimientos thead th:nth-child(7) { width: 6%; }
    .tabla-requerimientos thead th:nth-child(8) { width: 9%; }
    .tabla-requerimientos thead th:nth-child(9) { width: 9%; }
    .tabla-requerimientos thead th:nth-child(10) { width: 6%; }
    .tabla-requerimientos thead th:nth-child(11) { width: 8%; }
    .tabla-requerimientos thead th:nth-child(12) { width: 9%; }

    .tabla-requerimientos thead {
      display: table-header-group !important;
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      page-break-after: avoid !important;
      break-after: avoid-page !important;
    }

    .tabla-requerimientos thead th {
      background: #2c3e50 !important;
      color: white !important;
      border: none;
      padding: 0.04cm 0.04cm !important;
      font-size: 7pt;
      font-weight: 700;
      text-align: left;
      vertical-align: middle;
      line-height: 1;
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      -webkit-column-break-inside: avoid !important;
    }

    .tabla-requerimientos thead th:nth-child(3),
    .tabla-requerimientos thead th:nth-child(8) {
      border-right: 1px solid #333 !important;
    }

    .tabla-requerimientos tbody {
      display: table-row-group !important;
    }

    .tabla-requerimientos tbody td {
      color: #000 !important;
      border: none;
      border-bottom: 1px solid #ccc;
      padding: 0.03cm 0.03cm !important;
      font-size: 7pt;
      vertical-align: middle;
      line-height: 1;
      word-wrap: break-word;
      overflow-wrap: break-word;
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      -webkit-column-break-inside: avoid !important;
    }

    .tabla-requerimientos tbody td:nth-child(1),
    .tabla-requerimientos tbody td:nth-child(3),
    .tabla-requerimientos tbody td:nth-child(4),
    .tabla-requerimientos tbody td:nth-child(5),
    .tabla-requerimientos tbody td:nth-child(6),
    .tabla-requerimientos tbody td:nth-child(7),
    .tabla-requerimientos tbody td:nth-child(8),
    .tabla-requerimientos tbody td:nth-child(9),
    .tabla-requerimientos tbody td:nth-child(10),
    .tabla-requerimientos tbody td:nth-child(11),
    .tabla-requerimientos tbody td:nth-child(12) {
      white-space: nowrap;
    }

    .tabla-requerimientos tbody td:nth-child(2) {
      white-space: normal;
      word-wrap: break-word;
      overflow-wrap: break-word;
    }

    .tabla-requerimientos tbody td:nth-child(3),
    .tabla-requerimientos tbody td:nth-child(8) {
      border-right: 1px solid #333 !important;
    }

    .tabla-requerimientos tbody tr:nth-child(even) { background: #fafafa; }
    .tabla-requerimientos tbody tr:nth-child(odd) { background: white; }
    .tabla-requerimientos tbody tr { 
      display: table-row;
      page-break-inside: avoid !important; 
      break-inside: avoid-page !important;
      -webkit-column-break-inside: avoid !important;
      page-break-after: auto !important;
      page-break-before: auto !important;
      height: auto;
      vertical-align: middle;
    }
    
    .tabla-requerimientos tbody td {
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      -webkit-column-break-inside: avoid !important;
      vertical-align: top !important;
    }

    .centrado { text-align: center; }
    .derecha { text-align: right; }
    .destacado { color: #1a4d7a !important; font-weight: 600; }

    .descripcion-cell {
      font-size: 7.5pt;
      line-height: 1.3;
      word-wrap: break-word;
      overflow-wrap: break-word;
      hyphens: auto;
    }

    .seccion-total {
      page-break-inside: avoid;
      margin-top: 0.2cm;
      background: #e8f4f8;
      border: 2px solid #333;
      padding: 0.1cm 0.2cm;
    }

    .total-row {
      font-size: 8.5pt;
      display: flex;
      justify-content: space-between;
      padding: 0.05cm 0;
    }

    .total-label {
      color: #000 !important;
      font-weight: 700;
    }

    .total-valor {
      color: #1a4d7a !important;
      font-weight: 700;
      font-size: 9.5pt;
    }

    .total-final {
      border-top: 2px solid #333;
      padding-top: 0.3rem !important;
      margin-top: 0.2rem;
    }

    .seccion-firma {
      page-break-inside: avoid;
      margin-top: 0.4cm;
    }

    .firma-box {
      position: relative;
    }

    .firma-imagen {
      position: absolute !important;
      bottom: 30px !important;
      left: 50% !important;
      transform: translateX(-50%) !important;
      max-width: 3.9cm;
      height: auto;
    }

    .firma-label,
    .firma-nombre {
      position: relative;
      z-index: 0;
    }

    .firma-label,
    .firma-nombre {
      font-size: 7.5pt;
      color: #000 !important;
    }

    .pie-pagina {
      page-break-inside: avoid;
      font-size: 7pt;
      padding-top: 0.15cm;
      border-top: 1px solid #999;
      color: #000 !important;
      margin-top: 0.2cm;
    }

    .pie-pagina p {
      margin: 0;
    }

    .seccion-firma {
      page-break-before: auto;
    }
  }
</style>
