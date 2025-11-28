import "./App.css";

function App() {
  return (
    <div className="app-root">
      <header className="app-header">
        <div className="app-title">
          <span className="app-badge">Orban Agent</span>
          <h1>Earnings Dashboard</h1>
        </div>
      </header>

      <main className="app-main">
        <section className="stat-grid">
          <div className="stat-card accent">
            <div className="stat-label">Total Earnings</div>
            <div className="stat-value">--</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Today</div>
            <div className="stat-value">--</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Pending</div>
            <div className="stat-value">--</div>
          </div>
        </section>

        <section className="panel gpu-panel">
          <div className="panel-header">
            <h2>GPU</h2>
            <span className="panel-subtitle">1 device</span>
          </div>
          <div className="gpu-list">
            <div className="gpu-card">
              <div className="gpu-icon">🎮</div>
              <div className="gpu-details">
                <div className="gpu-name">NVIDIA GeForce MX250</div>
                <div className="gpu-meta">
                  <span className="gpu-vendor">NVIDIA</span>
                  <span className="gpu-vram">VRAM: 2.0 GB</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="panel">
          <div className="panel-header">
            <h2>Agent Status</h2>
          </div>
          <div className="status-box">
            <div className="status-dot offline" />
            <div className="status-label">Disconnected</div>
          </div>
        </section>
      </main>
    </div>
  );
}

export default App;
