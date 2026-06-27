import { useState, useEffect } from 'react';
import { Shield, Lock, Activity, Server, ArrowRight, Code, Globe, Cpu, Database, Network, Terminal, Zap, Box, CheckCircle, ChevronRight, Cloud, Layers } from 'lucide-react';
import './index.css';

const translations = {
  en: {
    features: "Features",
    architecture: "Architecture",
    docs: "Documentation",
    source: "Source (Coming Soon)",
    heroTitle1: "Next-Gen Security",
    heroTitle2: "Monitoring Platform",
    heroSub: "AegiSec provides unparalleled visibility into your infrastructure, detecting anomalies, managing access, and ensuring compliance with military-grade precision.",
    getStarted: "Get Started",
    readDocs: "Read the Docs",
    trustedBy: "Trusted by forward-thinking engineering teams",
    advantagesTitle: "Key Advantages",
    advCloud: "Cloud Native",
    advData: "Real-time Sync",
    advBlock: "Zero Trust",
    workflowTitle: "How it",
    workflowGradient: "Works",
    step1Title: "1. Deploy",
    step1Desc: "Install our lightweight agent in seconds.",
    step2Title: "2. Trace",
    step2Desc: "Monitor network and system events in real-time.",
    step3Title: "3. Secure",
    step3Desc: "Detect and block threats automatically.",
    coreCapabilities: "Core",
    capabilitiesGradient: "Capabilities",
    telemetryTitle: "Real-Time Telemetry",
    telemetryDesc: "Continuous monitoring of system resources, network traffic, and active processes across all your nodes with zero-latency agents.",
    threatTitle: "Threat Detection",
    threatDesc: "Advanced heuristics and signature-based scanning to identify malicious activity, unauthorized access, and vulnerabilities instantly.",
    dashboardTitle: "Centralized Dashboard",
    dashboardDesc: "Manage all your endpoints from a single, unified interface. Deploy policies, review logs, and mitigate risks in a few clicks.",
    techTitle: "System",
    techGradient: "Architecture",
    techRust: "Core Engine (Rust)",
    techRustDesc: "High-performance, memory-safe backend ensuring zero-latency packet analysis and telemetry.",
    techReact: "Dynamic Dashboard (React)",
    techReactDesc: "Vite-powered frontend delivering real-time metric visualization and policy management.",
    techEbpf: "Advanced Analytics",
    techEbpfDesc: "Deep system visibility with low overhead, intercepting critical system events continuously.",
    techDb: "Time-Series Data",
    techDbDesc: "Optimized storage for massive telemetry streams, enabling historical analysis and anomaly detection.",
    ctaTitle: "Ready to secure your infrastructure?",
    ctaSub: "Join the next generation of cloud security.",
    footer: "AegiSec. All rights reserved.",
    comingSoonTitle: "In Development",
    comingSoonDesc: "This section will be available soon.",
    goBack: "Go Back"
  },
  es: {
    features: "Características",
    architecture: "Arquitectura",
    docs: "Documentación",
    source: "Código (Próximamente)",
    heroTitle1: "Seguridad de Nueva Generación",
    heroTitle2: "Plataforma de Monitoreo",
    heroSub: "AegiSec proporciona una visibilidad inigualable de su infraestructura, detectando anomalías, gestionando accesos y garantizando el cumplimiento con precisión de grado militar.",
    getStarted: "Empezar Ahora",
    readDocs: "Leer Documentación",
    trustedBy: "Confiado por equipos de ingeniería visionarios",
    advantagesTitle: "Ventajas Clave",
    advCloud: "Cloud Native",
    advData: "Real-time Sync",
    advBlock: "Zero Trust",
    workflowTitle: "Cómo",
    workflowGradient: "Funciona",
    step1Title: "1. Despliega",
    step1Desc: "Instala nuestro agente ultraligero en segundos.",
    step2Title: "2. Rastrea",
    step2Desc: "Monitorea eventos de red y sistema en tiempo real.",
    step3Title: "3. Asegura",
    step3Desc: "Detecta y bloquea amenazas automáticamente.",
    coreCapabilities: "Capacidades",
    capabilitiesGradient: "Principales",
    telemetryTitle: "Telemetría en Tiempo Real",
    telemetryDesc: "Monitoreo continuo de recursos del sistema, tráfico de red y procesos activos en todos sus nodos con agentes de latencia cero.",
    threatTitle: "Detección de Amenazas",
    threatDesc: "Heurística avanzada y escaneo basado en firmas para identificar actividades maliciosas, accesos no autorizados y vulnerabilidades al instante.",
    dashboardTitle: "Panel Centralizado",
    dashboardDesc: "Gestione todos sus puntos finales desde una única interfaz unificada. Implemente políticas, revise registros y mitigue riesgos con pocos clics.",
    techTitle: "Arquitectura del",
    techGradient: "Sistema",
    techRust: "Motor Central (Rust)",
    techRustDesc: "Backend de alto rendimiento y seguro en memoria que garantiza análisis de paquetes y telemetría sin latencia.",
    techReact: "Panel Dinámico (React)",
    techReactDesc: "Frontend impulsado por Vite que ofrece visualización de métricas en tiempo real y gestión de políticas.",
    techEbpf: "Analítica Avanzada",
    techEbpfDesc: "Visibilidad profunda del sistema con baja sobrecarga, interceptando eventos críticos continuamente.",
    techDb: "Datos de Series Temporales",
    techDbDesc: "Almacenamiento optimizado para flujos masivos de telemetría, permitiendo análisis histórico y detección de anomalías.",
    ctaTitle: "¿Listo para asegurar tu infraestructura?",
    ctaSub: "Únete a la próxima generación de seguridad en la nube.",
    footer: "AegiSec. Todos los derechos reservados.",
    comingSoonTitle: "En Desarrollo",
    comingSoonDesc: "Esta sección estará disponible próximamente.",
    goBack: "Volver al Inicio"
  }
};

type Lang = 'en' | 'es';

function App() {
  const [lang, setLang] = useState<Lang>('es');
  const [showComingSoon, setShowComingSoon] = useState(false);
  const [terminalLines, setTerminalLines] = useState<string[]>([]);
  const t = translations[lang] as any;

  const toggleLanguage = () => {
    setLang(lang === 'en' ? 'es' : 'en');
  };

  useEffect(() => {
    const lines = [
      "> initializing aegi-agent v2.2.4...",
      "> connecting to command center... OK",
      "> applying zero-trust policies... OK",
      "> security monitor loaded successfully.",
      "> network scan complete... 0 anomalies found.",
      "> system secure."
    ];
    let currentLine = 0;
    const interval = setInterval(() => {
      if (currentLine < lines.length) {
        setTerminalLines(prev => [...prev, lines[currentLine]]);
        currentLine++;
      } else {
        clearInterval(interval);
      }
    }, 800);
    return () => clearInterval(interval);
  }, []);

  if (showComingSoon) {
    return (
      <div className="app-container">
        <div className="container" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh', textAlign: 'center' }}>
          <Shield className="logo-icon pulse" size={64} style={{ marginBottom: '2rem' }} />
          <h1 className="text-gradient" style={{ fontSize: '4rem', marginBottom: '1rem' }}>{t.comingSoonTitle}</h1>
          <p className="subtitle" style={{ marginBottom: '2rem' }}>{t.comingSoonDesc}</p>
          <button className="btn btn-secondary glass-btn" onClick={() => setShowComingSoon(false)}>
            {t.goBack}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <div className="container">
        {/* Navbar */}
        <nav className="navbar animate-fade-in-down">
          <div className="logo" style={{ cursor: 'pointer' }} onClick={() => setShowComingSoon(false)}>
            <Shield className="logo-icon pulse" size={28} />
            AegiSec
          </div>
          <div className="nav-links">
            <a href="#features" className="hover-underline">{t.features}</a>
            <a href="#architecture" className="hover-underline">{t.architecture}</a>
            <a href="#docs" className="hover-underline">{t.docs}</a>
          </div>
          <div className="nav-actions">
            <button className="btn btn-secondary glass-btn" onClick={toggleLanguage}>
              <Globe size={18} /> {lang.toUpperCase()}
            </button>
            <button className="btn btn-secondary glass-btn disabled-btn" title="Coming Soon">
              <Code size={18} /> {t.source}
            </button>
          </div>
        </nav>

        {/* Hero Section */}
        <header className="hero hero-split">
          <div className="hero-content">
            <h1 className="animate-slide-up">
              {t.heroTitle1} <br />
              <span className="text-gradient">{t.heroTitle2}</span>
            </h1>
            <p className="subtitle animate-slide-up delay-1">
              {t.heroSub}
            </p>
            <div className="hero-actions animate-slide-up delay-2">
              <button className="btn btn-primary glow-btn" onClick={() => setShowComingSoon(true)}>
                {t.getStarted} <ArrowRight size={18} />
              </button>
              <button className="btn btn-secondary glass-btn" onClick={() => setShowComingSoon(true)}>
                {t.readDocs}
              </button>
            </div>
          </div>
          
          <div className="hero-visual animate-slide-up delay-3">
            <div className="glass-panel terminal-mockup">
              <div className="terminal-header">
                <span className="dot red"></span>
                <span className="dot yellow"></span>
                <span className="dot green"></span>
                <span className="terminal-title">zsh - aegi-agent</span>
              </div>
              <div className="terminal-body">
                {terminalLines.map((line, i) => (
                  <div key={i} className="terminal-line typing-anim">{line}</div>
                ))}
                {terminalLines.length === 6 && <div className="terminal-line blink">_</div>}
              </div>
            </div>
          </div>
          
          {/* Animated Tech Orbs */}
          <div className="floating-orbs">
            <div className="orb orb-1"><Shield size={32} /></div>
            <div className="orb orb-2"><Terminal size={32} /></div>
            <div className="orb orb-3"><Activity size={32} /></div>
          </div>
        </header>

        {/* Advantages Section */}
        <section className="trusted-by animate-on-scroll">
          <p className="trusted-title">{t.advantagesTitle}</p>
          <div className="trusted-logos">
            <div className="trusted-logo"><Cloud size={32} /> <span>{t.advCloud}</span></div>
            <div className="trusted-logo"><Layers size={32} /> <span>{t.advData}</span></div>
            <div className="trusted-logo"><Box size={32} /> <span>{t.advBlock}</span></div>
          </div>
        </section>

        {/* Workflow Section */}
        <section className="workflow">
          <h2 className="section-title animate-on-scroll">{t.workflowTitle} <span className="text-gradient">{t.workflowGradient}</span></h2>
          <div className="workflow-steps">
            <div className="workflow-step animate-on-scroll delay-1">
              <div className="step-icon glow-icon"><Box size={32} /></div>
              <h3>{t.step1Title}</h3>
              <p>{t.step1Desc}</p>
            </div>
            <ChevronRight size={48} className="step-arrow hidden-mobile" />
            <div className="workflow-step animate-on-scroll delay-2">
              <div className="step-icon glow-icon"><Activity size={32} /></div>
              <h3>{t.step2Title}</h3>
              <p>{t.step2Desc}</p>
            </div>
            <ChevronRight size={48} className="step-arrow hidden-mobile" />
            <div className="workflow-step animate-on-scroll delay-3">
              <div className="step-icon glow-icon"><CheckCircle size={32} /></div>
              <h3>{t.step3Title}</h3>
              <p>{t.step3Desc}</p>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <section id="features" className="features">
          <h2 className="section-title animate-on-scroll">{t.coreCapabilities} <span className="text-gradient">{t.capabilitiesGradient}</span></h2>
          <div className="features-grid">
            
            <div className="glass-panel hover-card animate-on-scroll delay-1">
              <div className="feature-icon glow-icon">
                <Activity size={24} />
              </div>
              <h3 className="feature-title">{t.telemetryTitle}</h3>
              <p className="feature-desc">
                {t.telemetryDesc}
              </p>
            </div>

            <div className="glass-panel hover-card animate-on-scroll delay-2">
              <div className="feature-icon glow-icon">
                <Lock size={24} />
              </div>
              <h3 className="feature-title">{t.threatTitle}</h3>
              <p className="feature-desc">
                {t.threatDesc}
              </p>
            </div>

            <div className="glass-panel hover-card animate-on-scroll delay-3">
              <div className="feature-icon glow-icon">
                <Server size={24} />
              </div>
              <h3 className="feature-title">{t.dashboardTitle}</h3>
              <p className="feature-desc">
                {t.dashboardDesc}
              </p>
            </div>

          </div>
        </section>

        {/* Architecture Section */}
        <section id="architecture" className="features architecture-section">
          <h2 className="section-title animate-on-scroll">{t.techTitle} <span className="text-gradient">{t.techGradient}</span></h2>
          <div className="features-grid arch-grid">
            
            <div className="glass-panel tech-panel hover-card animate-on-scroll delay-1">
              <div className="tech-header">
                <Cpu size={32} className="tech-icon text-gradient" />
                <h3 className="tech-title">{t.techRust}</h3>
              </div>
              <p className="feature-desc">{t.techRustDesc}</p>
            </div>

            <div className="glass-panel tech-panel hover-card animate-on-scroll delay-2">
              <div className="tech-header">
                <Network size={32} className="tech-icon text-gradient" />
                <h3 className="tech-title">{t.techEbpf}</h3>
              </div>
              <p className="feature-desc">{t.techEbpfDesc}</p>
            </div>

            <div className="glass-panel tech-panel hover-card animate-on-scroll delay-3">
              <div className="tech-header">
                <Database size={32} className="tech-icon text-gradient" />
                <h3 className="tech-title">{t.techDb}</h3>
              </div>
              <p className="feature-desc">{t.techDbDesc}</p>
            </div>

            <div className="glass-panel tech-panel hover-card animate-on-scroll delay-4">
              <div className="tech-header">
                <Zap size={32} className="tech-icon text-gradient" />
                <h3 className="tech-title">{t.techReact}</h3>
              </div>
              <p className="feature-desc">{t.techReactDesc}</p>
            </div>

          </div>
        </section>

        {/* CTA Section */}
        <section className="cta-section animate-on-scroll">
          <div className="glass-panel cta-panel">
            <h2>{t.ctaTitle}</h2>
            <p className="subtitle">{t.ctaSub}</p>
            <button className="btn btn-primary glow-btn cta-btn" onClick={() => setShowComingSoon(true)}>
              {t.getStarted} <ArrowRight size={18} />
            </button>
          </div>
        </section>

        {/* Footer */}
        <footer className="footer">
          <div className="footer-content">
            <Shield className="logo-icon footer-logo pulse" size={24} />
            <p>© {new Date().getFullYear()} {t.footer}</p>
          </div>
        </footer>
      </div>
    </div>
  );
}

export default App;
