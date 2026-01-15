<script>
  export let onContinue = () => {};
</script>

<div class="bienvenida-container">
  <div class="bienvenida-card">
    <div class="logo-section">
      <div class="logo-circle">
        <span class="logo-text">FLAD</span>
      </div>
      <h1 class="sistema-titulo">Flujo Administrativo de Contratos de Mantención</h1>
    </div>

    <div class="descripcion-section">
      <p>
        FLAD gestiona el ciclo completo de contratos de mantención mediante cuatro fases integradas. 
        El sistema controla desde el registro inicial hasta el pago final, asegurando trazabilidad 
        y cumplimiento de plazos contractuales.
      </p>
      
      <p>
        Las fases incluyen: Requerimientos (ingreso y seguimiento), Órdenes de Trabajo (creación y gestión), 
        Recepción (validación e informes), y Pagos (procesamiento y registro). Cada fase cuenta con dos módulos 
        especializados que garantizan control y eficiencia operativa.
      </p>
      
      <p>
        El flujo automatiza cálculos de plazos, multas y montos a pagar, manteniendo sincronización 
        en tiempo real entre módulos. Los ocho módulos operan integrados para optimizar la administración 
        contractual de mantención en jardines infantiles JUNJI.
      </p>

      <div class="fases-grid">
        <div class="fase-card">
          <div class="fase-numero">1</div>
          <div class="fase-info">
            <h3>Requerimientos</h3>
            <p>Ingreso y Listado</p>
          </div>
        </div>
        
        <div class="fase-card">
          <div class="fase-numero">2</div>
          <div class="fase-info">
            <h3>Órdenes de Trabajo</h3>
            <p>Creación y Gestión</p>
          </div>
        </div>
        
        <div class="fase-card">
          <div class="fase-numero">3</div>
          <div class="fase-info">
            <h3>Recepción</h3>
            <p>Validación e Informes</p>
          </div>
        </div>
        
        <div class="fase-card">
          <div class="fase-numero">4</div>
          <div class="fase-info">
            <h3>Pagos</h3>
            <p>Procesamiento y Registro</p>
          </div>
        </div>
      </div>
    </div>

    <button class="btn-continuar" on:click={onContinue}>
      Comenzar
    </button>

    <div class="creditos">
      <p>Créditos: Daniel Mena Flores, 2025.</p>
      <p>Desarrollado con Claude Sonnet 4.5</p>
      <p class="version">Versión 2.0.0</p>
    </div>
  </div>
</div>

<style>
  .bienvenida-container {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .bienvenida-card {
    background: #1a2332;
    border: 1px solid #2d3e50;
    border-radius: 20px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.8);
    max-width: 900px;
    width: 100%;
    padding: 3rem;
    animation: fadeInUp 0.6s ease-out;
  }

  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(30px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .logo-section {
    text-align: center;
    margin-bottom: 2.5rem;
  }

  .logo-circle {
    width: 120px;
    height: 120px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1.5rem;
    box-shadow: 0 10px 30px rgba(102, 126, 234, 0.4);
  }

  .logo-text {
    font-size: 2.5rem;
    font-weight: 800;
    color: white;
    letter-spacing: 2px;
  }

  .sistema-titulo {
    font-size: 1.8rem;
    font-weight: 700;
    color: #e0e6ed;
    margin: 0;
    line-height: 1.3;
  }

  .descripcion-section {
    margin-bottom: 2.5rem;
  }

  .descripcion-section p {
    font-size: 1rem;
    line-height: 1.7;
    color: #a8b5c7;
    margin-bottom: 1rem;
    text-align: justify;
  }

  .fases-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-top: 2rem;
  }

  .fase-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: #0f1419;
    border-radius: 12px;
    border: 2px solid #2d3e50;
    transition: all 0.3s ease;
  }

  .fase-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(90, 143, 196, 0.3);
    border-color: #5a8fc4;
  }

  .fase-numero {
    width: 40px;
    height: 40px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
    font-weight: 700;
    flex-shrink: 0;
  }

  .fase-info h3 {
    font-size: 0.95rem;
    font-weight: 600;
    color: #e0e6ed;
    margin: 0 0 0.25rem 0;
  }

  .fase-info p {
    font-size: 0.8rem;
    color: #8b9eb3;
    margin: 0;
    text-align: left;
  }

  .btn-continuar {
    width: 100%;
    padding: 1rem 2rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 12px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
  }

  .btn-continuar:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
  }

  .btn-continuar:active {
    transform: translateY(0);
  }

  .creditos {
    text-align: center;
    margin-top: 1.5rem;
  }

  .creditos p {
    font-size: 0.75rem;
    color: #6b7d8f;
    margin: 0.25rem 0;
    line-height: 1.4;
  }

  .creditos .version {
    color: #5a8fc4;
    font-weight: 600;
    margin-top: 0.5rem;
  }

  @media (max-width: 768px) {
    .bienvenida-card {
      padding: 2rem 1.5rem;
    }

    .sistema-titulo {
      font-size: 1.4rem;
    }

    .fases-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
