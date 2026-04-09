pub const AXUM_FRONTEND_CSS: &str = r#"
body{margin:0;color:#e8edf2;font-family:Georgia,serif;background:linear-gradient(180deg,#061019 0%,#091722 40%,#0d1821 100%);min-height:100vh}
.shell{max-width:1320px;margin:0 auto;padding:2rem 1rem 4rem}
.hero{display:grid;grid-template-columns:minmax(0,1fr) 260px;gap:1.5rem;align-items:end;margin-bottom:2rem}
.breadcrumbs{display:flex;flex-wrap:wrap;gap:.45rem;margin-bottom:1rem;color:#a8bccc;font-family:system-ui,sans-serif;font-size:.82rem}
.breadcrumbs a{color:inherit;text-decoration:none}.hero h1{margin:0;font-size:clamp(2rem,5vw,4rem);line-height:.95;color:#f7f4ef}.lede{margin-top:.9rem;color:#acc0ce}
.hero-panel,.controls,.folder-card,.media-card{border:1px solid rgba(255,255,255,.08);border-radius:1rem;background:rgba(8,18,27,.84);box-shadow:0 16px 40px rgba(0,0,0,.18)}
.hero-panel{padding:1rem}.panel-label{display:block;color:#8fa8b8;font-size:.74rem;letter-spacing:.16em;text-transform:uppercase;font-family:system-ui,sans-serif}
.hero-panel strong{display:block;margin-top:.4rem;font-size:1.6rem;color:#fff4e6}.controls{padding:1rem;margin-bottom:1.5rem}.controls-form{display:flex;gap:.75rem;align-items:end;flex-wrap:wrap}
.controls-form label{display:flex;flex-direction:column;gap:.35rem;color:#a8bccc;font-size:.78rem;font-family:system-ui,sans-serif;text-transform:uppercase}
.controls-form select,.controls-form button,.refresh-link{padding:.55rem .8rem;border-radius:999px;border:1px solid rgba(255,255,255,.08);background:#0d1821;color:#f7f4ef;text-decoration:none}
.folder-section,.media-section{margin-top:2rem}.folder-grid,.media-grid{display:grid;gap:1rem}.folder-grid{grid-template-columns:repeat(auto-fill,minmax(220px,1fr))}
.folder-cover,.media-thumb{display:block;aspect-ratio:4/3;background:#081018;overflow:hidden}.folder-cover img,.media-thumb img{width:100%;height:100%;object-fit:cover;display:block}
.folder-placeholder{display:grid;place-items:center;height:100%;color:#f0c48c;background:rgba(255,255,255,.05)}.folder-meta,.media-meta{padding:.9rem 1rem 1rem}
.folder-meta h3,.media-meta h3{margin:0;color:#f7f4ef;font-size:1rem}.folder-meta p,.media-meta p,.hero-panel p,.empty-note,.sort-note{color:#89a0b2}
.media-actions{display:flex;gap:.7rem;margin-top:.7rem}.media-actions a{color:#f0c48c;text-decoration:none;font-family:system-ui,sans-serif;font-size:.8rem;text-transform:uppercase;letter-spacing:.08em}
.pager{display:flex;gap:1rem;margin-top:1rem}.pager a{padding:.55rem .85rem;border-radius:999px;background:#0d1821;color:#f7f4ef;text-decoration:none;border:1px solid rgba(255,255,255,.08)}
@media (max-width:720px){.hero{grid-template-columns:1fr}.controls-form{align-items:stretch}.controls-form label{width:100%}.controls-form select,.controls-form button,.refresh-link{width:100%;box-sizing:border-box}}
"#;
